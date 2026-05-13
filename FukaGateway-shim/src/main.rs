// Copyright (C) 2026 Max Morris.
// 
// This file is part of FukaGateway.
// 
// FukaGateway is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// FukaGateway is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod endpoints;

use actix_web::{App, HttpServer};
use std::io;
use landlock::{path_beneath_rules, Access, AccessFs, AccessNet, NetPort, RulesetAttr, RulesetCreatedAttr, RulesetError, RulesetStatus};
use FukaGateway_lib::{SHIM_PORT, WORKER_PORT};
use seccompiler::{
    apply_filter, BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp,
    SeccompCondition, SeccompFilter, SeccompRule,
};
use std::{collections::BTreeMap, convert::TryInto};

#[actix_web::main]
async fn main() -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let landlock_status = do_landlock().expect("Failed to enforce Landlock ruleset");
        println!("Landlock status: {:?}", landlock_status);

        do_seccomp().expect("Failed to enforce seccomp filter");
        println!("Seccomp filter applied");
    }

    HttpServer::new(|| {
        App::new()
            .service(endpoints::submit_info_file)
            .service(endpoints::poll_job)
            .service(endpoints::download_job)
            .service(endpoints::get_job_info)
            .service(endpoints::list_jobs)
    })
    .bind(("0.0.0.0", SHIM_PORT))?
    .run()
    .await
}

const ALLOWED_PATHS: &[&str] = &[
    "/proc/self/cgroup",
    "/sys/fs/cgroup",
    "/sys/devices/system/cpu/online",
    "/proc/stat",
    "/etc/hosts",
    "/etc/resolv.conf",
    "/etc/host.conf",
    "/etc/nsswitch.conf"
];

#[cfg(target_os = "linux")]
fn do_landlock() -> Result<RulesetStatus, RulesetError> {
    let abi = landlock::ABI::V6;
    let ruleset =
        landlock::Ruleset::default()
                          .handle_access(AccessFs::from_all(abi))?
                          .handle_access(AccessNet::BindTcp)?
                          .handle_access(AccessNet::ConnectTcp)?;

    let status =
        ruleset.create()?.add_rule(NetPort::new(SHIM_PORT, AccessNet::BindTcp))?
                         .add_rule(NetPort::new(WORKER_PORT, AccessNet::ConnectTcp))?
                         .add_rules(path_beneath_rules(ALLOWED_PATHS, AccessFs::from_read(abi)))?
                         .restrict_self()?;

    Ok(status.ruleset)
}

#[cfg(target_os = "linux")]
fn do_seccomp() -> seccompiler::Result<()> {
    // Linux encodes the socket type in the low nibble of the second socket() argument.
    // This lets us match SOCK_DGRAM even when flags (SOCK_CLOEXEC/SOCK_NONBLOCK) are ORed in.
    const SOCKET_TYPE_MASK: u64 = 0x0f;

    // Reusable closure for creating rules matching socket(<family>, SOCK_DGRAM, <protocol>) syscalls
    let udp_socket_rule = |family: i64, protocol: i64| -> seccompiler::Result<SeccompRule> {
        Ok(SeccompRule::new(vec![
            SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, family as u64)?,
            SeccompCondition::new(
                1,
                SeccompCmpArgLen::Dword,
                SeccompCmpOp::MaskedEq(SOCKET_TYPE_MASK),
                libc::SOCK_DGRAM as u64,
            )?,
            SeccompCondition::new(
                2,
                SeccompCmpArgLen::Dword,
                SeccompCmpOp::Eq,
                protocol as u64,
            )?,
        ])?)
    };

    // Map syscall numbers to rules.
    let rules: BTreeMap<_, _> = vec![
        (
            libc::SYS_socket as i64,
            vec![
                // kernel behavior: protocol 0 means "default for this type/family" (UDP for dgram)
                // If any of these rules match, the syscall will match.
                udp_socket_rule(libc::AF_INET as i64, 0)?,
                udp_socket_rule(libc::AF_INET as i64, libc::IPPROTO_UDP as i64)?,
                udp_socket_rule(libc::AF_INET6 as i64, 0)?,
                udp_socket_rule(libc::AF_INET6 as i64, libc::IPPROTO_UDP as i64)?,
            ],
        ),
        // Empty rule vector means unconditional match for this syscall number.
        (libc::SYS_mount as i64, vec![]),
        (libc::SYS_umount2 as i64, vec![]),
    ]
    .into_iter()
    .collect();

    // Create the seccomp filter with our ruleset.
    let filter: BpfProgram = SeccompFilter::new(
        rules,
        SeccompAction::Allow, // mismatch_action; i.e., allow all syscalls that don't match any rule
        SeccompAction::Errno(libc::EPERM as u32), // block and return EPERM for syscalls matching a rule
        std::env::consts::ARCH.try_into()?,
    )?
    .try_into()?;

    apply_filter(&filter)
}
