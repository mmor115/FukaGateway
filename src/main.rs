mod endpoints;

use actix_web::{App, HttpServer};
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(endpoints::echo)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}