mod endpoints;
mod info_file_parser;
mod database;
mod job;

use actix_web::{App, HttpServer};
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(endpoints::echo)
            .service(endpoints::lex_info_file)
            .service(endpoints::parse_info_file)
            .service(endpoints::submit_info_file)
            .service(endpoints::poll_job)
            .service(endpoints::download_job)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}