mod endpoints;

use actix_web::{App, HttpServer};
use std::io;
use FukaGateway_lib::WORKER_PORT;

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(endpoints::submit_info_file)
            .service(endpoints::poll_job)
            .service(endpoints::download_job)
            .service(endpoints::get_job_info)
            .service(endpoints::list_jobs)
    })
    .bind(("127.0.0.1", WORKER_PORT))?
    .run()
    .await
}