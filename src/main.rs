mod endpoints;
mod info_file_parser;

use actix_web::{App, HttpServer};
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(endpoints::echo)
            .service(endpoints::lex_info_file)
            .service(endpoints::parse_info_file)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}