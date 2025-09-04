mod endpoints;

use actix_web::{App, HttpServer};
use std::io;
use landlock::{path_beneath_rules, Access, AccessFs, AccessNet, NetPort, RulesetAttr, RulesetCreatedAttr, RulesetError, RulesetStatus};
use FukaGateway_lib::{SHIM_PORT, WORKER_PORT};

#[actix_web::main]
async fn main() -> io::Result<()> {
    if cfg!(target_os = "linux") {
        let landlock_status = do_landlock().expect("Failed to enforce Landlock ruleset");
        println!("Landlock status: {:?}", landlock_status);
    }

    HttpServer::new(|| {
        App::new()
            .service(endpoints::submit_info_file)
            .service(endpoints::poll_job)
            .service(endpoints::download_job)
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