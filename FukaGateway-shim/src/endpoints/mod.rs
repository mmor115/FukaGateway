use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use uuid::Uuid;
use FukaGateway_lib::endpoints::error::EndpointError;
use FukaGateway_lib::WORKER_PORT;

#[post("submit")]
async fn submit_info_file(req_body: String) -> Result<impl Responder, EndpointError> {
    let client = reqwest::Client::new();
    let resp = client.post(format!("http://localhost:{}/submit", WORKER_PORT)).body(req_body).send().await?;

    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap();
    let body = resp.text().await?;

    Ok(
        HttpResponse::build(status)
                     .insert_header(ContentType::json())
                     .body(body)
    )
}

#[get("poll/{job_id}")]
async fn poll_job(path: web::Path<Uuid>) -> Result<impl Responder, EndpointError> {
    let client = reqwest::Client::new();
    let job_id = path.into_inner();
    
    let resp = client.get(format!("http://localhost:{}/poll/{}", WORKER_PORT, job_id)).send().await?;

    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap();
    let body = resp.text().await?;

    Ok(
        HttpResponse::build(status)
                     .insert_header(ContentType::json())
                     .body(body)
    )
}

#[get("download/{job_id}")]
async fn download_job(path: web::Path<Uuid>) -> Result<impl Responder, EndpointError> {
    let client = reqwest::Client::new();
    let job_id = path.into_inner();

    let resp = client.get(format!("http://localhost:{}/download/{}", WORKER_PORT, job_id)).send().await?;
    
    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap();
    if !status.is_success() {
        return Ok(
            HttpResponse::build(status)
                         .insert_header(ContentType::json())
                         .body(resp.text().await?)
        );
    }

    Ok(
        HttpResponse::build(status)
                     .append_header(("Content-Type", resp.headers().get("content-type").unwrap().to_str().unwrap()))
                     .append_header(("Content-Disposition", resp.headers().get("content-disposition").unwrap().to_str().unwrap()))
                     .body(resp.bytes().await?)
    )
}