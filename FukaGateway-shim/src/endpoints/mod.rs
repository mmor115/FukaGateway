use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use uuid::Uuid;
use FukaGateway_lib::endpoints::error::EndpointError;
use FukaGateway_lib::WORKER_PORT;

#[post("jobs")]
async fn submit_info_file(req_body: String) -> Result<impl Responder, EndpointError> {
    let client = reqwest::Client::new();
    let resp = client.post(format!("http://localhost:{}/jobs", WORKER_PORT)).body(req_body).send().await?;

    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap();
    let body = resp.text().await?;

    Ok(
        HttpResponse::build(status)
                     .insert_header(ContentType::json())
                     .body(body)
    )
}

#[get("jobs/{job_id}/status")]
async fn poll_job(path: web::Path<Uuid>) -> Result<impl Responder, EndpointError> {
    let client = reqwest::Client::new();
    let job_id = path.into_inner();
    
    let resp = client.get(format!("http://localhost:{}/jobs/{}/status", WORKER_PORT, job_id)).send().await?;

    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap();
    let body = resp.text().await?;

    Ok(
        HttpResponse::build(status)
                     .insert_header(ContentType::json())
                     .body(body)
    )
}

#[get("jobs/{job_id}/info")]
async fn get_job_info(path: web::Path<Uuid>) -> Result<impl Responder, EndpointError> {
    let client = reqwest::Client::new();
    let job_id = path.into_inner();

    let resp = client.get(format!("http://localhost:{}/jobs/{}/info", WORKER_PORT, job_id)).send().await?;

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

#[get("jobs/{job_id}/result")]
async fn download_job(path: web::Path<Uuid>) -> Result<impl Responder, EndpointError> {
    let client = reqwest::Client::new();
    let job_id = path.into_inner();

    let resp = client.get(format!("http://localhost:{}/jobs/{}/result", WORKER_PORT, job_id)).send().await?;
    
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