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