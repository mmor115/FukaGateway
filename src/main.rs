mod endpoints;
mod info_file_parser;
mod database;
mod job;

use actix_web::{App, HttpServer};
use std::io;
use landlock::{path_beneath_rules, Access, AccessFs, AccessNet, NetPort, RulesetAttr, RulesetCreatedAttr, RulesetError, RulesetStatus};
use crate::job::{get_jobs_dir, get_scripts_dir};

const PORT: u16 = 8080;

#[actix_web::main]
async fn main() -> io::Result<()> {
    if cfg!(target_os = "linux") {
        let landlock_status = do_landlock().expect("Failed to enforce Landlock ruleset");
        println!("Landlock status: {:?}", landlock_status);
    }

    HttpServer::new(|| {
        App::new()
            .service(endpoints::echo)
            .service(endpoints::lex_info_file)
            .service(endpoints::parse_info_file)
            .service(endpoints::submit_info_file)
            .service(endpoints::poll_job)
            .service(endpoints::download_job)
    })
    .bind(("127.0.0.1", PORT))?
    .run()
    .await
}

#[cfg(target_os = "linux")]
fn do_landlock() -> Result<RulesetStatus, RulesetError> {
    let abi = landlock::ABI::V6;
    let ruleset =
        landlock::Ruleset::default()
                          .handle_access(AccessFs::from_all(abi))?
                          .handle_access(AccessNet::BindTcp)?
                          .handle_access(AccessNet::ConnectTcp)?;

    let status =
        ruleset.create()?.add_rules(path_beneath_rules(&[get_jobs_dir(), get_scripts_dir()], AccessFs::from_all(abi)))?
                         .add_rules(path_beneath_rules(&[get_scripts_dir()], AccessFs::Execute))?
                         .add_rules(path_beneath_rules(&["/usr/bin/sbatch", "/usr/bin/squeue"], AccessFs::Execute))?
                         .add_rule(NetPort::new(PORT, AccessNet::BindTcp))?
                         .restrict_self()?;

    Ok(status.ruleset)
}