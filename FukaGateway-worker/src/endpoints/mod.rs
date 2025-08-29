mod error;
use FukaGateway_lib::database::{open_job_db, JobDbOperations, JobEntry};

use crate::endpoints::error::EndpointError;
use FukaGateway_lib::info_file_parser::flat_property_map::{InfoFileToFlatPropertyMapVisitor, PropMap};
use FukaGateway_lib::info_file_parser::lexer::InfoFileLexer;
use FukaGateway_lib::info_file_parser::parser::InfoFileParser;
use FukaGateway_lib::job::{get_info_file_path, get_job_dir, get_job_status, get_result_file_path, submit_job, JobStatus};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Serialize;
use std::io::ErrorKind::AlreadyExists;
use uuid::Uuid;
use crate::endpoints::error::EndpointError::NoSuchResource;
use futures_lite::io::AsyncWriteExt;

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    println!("{}", req_body);
    HttpResponse::Ok().body(req_body)
}

#[post("/lex")]
async fn lex_info_file(req_body: String) -> Result<impl Responder, EndpointError> {
    let tokens = InfoFileLexer::new(&req_body).lex()?;
    let mut output = String::new();

    for token in tokens {
        output.push_str(&token.to_string());
        output.push(' ');
    }

    Ok(HttpResponse::Ok().body(output))
}

#[post("/parse")]
async fn parse_info_file(req_body: String) -> Result<impl Responder, EndpointError> {
    let ast = {
        let tokens = InfoFileLexer::new(&req_body).lex()?;
        let mut par = InfoFileParser::new(tokens);
        par.parse()?
    };

    let properties = InfoFileToFlatPropertyMapVisitor::new(&ast).visit();

    let mut output = String::new();

    for (key, value) in properties {
        output.push_str(format!("{} -> {}\n", key, value).as_str());
    }

    Ok(HttpResponse::Ok().body(output))
}

#[derive(Serialize, Debug)]
#[serde(tag = "job_status")]
enum SubmitInfoFileResponse {
    Submitted { job_id: Uuid },
    AlreadySubmitted { job_id: Uuid },
    AlreadyRunning { job_id: Uuid },
    AlreadyFinished { job_id: Uuid },
    AlreadyFailed { job_id: Uuid }
}

#[post("submit")]
async fn submit_info_file(req_body: String) -> Result<impl Responder, EndpointError> {
    let (properties, req_body) = web::block(move || {
        do_parse_info_file(&req_body).map(|p| (p, req_body))
    }).await??;

    let job_entry = JobEntry::new(properties);
    let db = open_job_db()?;

    let (mut job_entry, mut db) = web::block(move || {
        db.exact_parameter_search(&job_entry)
          .map(|o| o.unwrap_or(job_entry))
          .map(|j| (j, db))
    }).await??;

    if let JobStatus::Pending { .. } | JobStatus::Running { .. } = job_entry.job_status &&
       let Some(job_status) = get_job_status(job_entry.id).await? {
            if let JobStatus::NotStarted = job_status {
                job_entry.job_status = JobStatus::Failed;
            } else {
                job_entry.job_status = job_status;
            }
    }

    if !matches!(job_entry.job_status, JobStatus::NotStarted) {
        db = web::block(move || {
            db.update_job_status(job_entry.id, job_entry.job_status).map(|_| db)
        }).await??;
    }

    match job_entry.job_status {
        JobStatus::NotStarted => {
            while let Err(e) = async_fs::create_dir(get_job_dir(job_entry.id)).await {
                if e.kind() != AlreadyExists {
                    return Err(e.into());
                }

                job_entry.id = Uuid::new_v4();
            }

            //async_fs::write(get_info_file_path(job_entry.id), req_body).await?;
            {
                let mut info_file = async_fs::File::create(get_info_file_path(job_entry.id)).await?;
                info_file.write_all(req_body.as_bytes()).await?;
                info_file.flush().await?;
            }

            let slurm_job_id = submit_job(job_entry.id).await?;
            job_entry.job_status = JobStatus::Pending { slurm_job_id };

            let job_id = web::block(move || {
                db.insert_job(&job_entry).map(|_| job_entry.id)
            }).await??;

            Ok(web::Json(SubmitInfoFileResponse::Submitted {
                job_id
            }))
        },
        JobStatus::Pending { .. } => {
            Ok(web::Json(SubmitInfoFileResponse::AlreadySubmitted {
                job_id: job_entry.id
            }))
        }
        JobStatus::Running { .. } => {
            Ok(web::Json(SubmitInfoFileResponse::AlreadyRunning {
                job_id: job_entry.id
            }))
        },
        JobStatus::Finished => {
            Ok(web::Json(SubmitInfoFileResponse::AlreadyFinished {
                job_id: job_entry.id
            }))
        },
        JobStatus::Failed => {
            Ok(web::Json(SubmitInfoFileResponse::AlreadyFailed {
                job_id: job_entry.id
            }))
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "job_status")]
enum JobPollResponse {
    NotFound,
    Pending,
    Running,
    Finished,
    Failed
}

#[get("poll/{job_id}")]
async fn poll_job(path: web::Path<Uuid>) -> Result<impl Responder, EndpointError> {
    let job_id = path.into_inner();
    let db = open_job_db()?;

    let (job_entry, mut db) = web::block(move || {
        db.id_search(job_id)
          .map(|j| (j, db))
    }).await??;

    let mut job_entry = match job_entry {
        Some(j) => j,
        None => {
            return Ok(web::Json(JobPollResponse::NotFound));
        }
    };

    if let JobStatus::Pending { .. } | JobStatus::Running { .. } = job_entry.job_status &&
        let Some(job_status) = get_job_status(job_entry.id).await? {
        if let JobStatus::NotStarted = job_status {
            job_entry.job_status = JobStatus::Failed;
        } else {
            job_entry.job_status = job_status;
        }
    }

    web::block(move || {
        db.update_job_status(job_entry.id, job_entry.job_status).map(|_| db)
    }).await??;

    Ok(web::Json(
        match job_entry.job_status {
            JobStatus::Pending { .. } => JobPollResponse::Pending,
            JobStatus::Running { .. } => JobPollResponse::Running,
            JobStatus::Finished => JobPollResponse::Finished,
            JobStatus::Failed => JobPollResponse::Failed,
            JobStatus::NotStarted => unreachable!()
        }
    ))
}

#[get("download/{job_id}")]
async fn download_job(path: web::Path<Uuid>) -> Result<impl Responder, EndpointError> {
    let job_id = path.into_inner();
    let db = open_job_db()?;

    let (job_entry, mut db) = web::block(move || {
        db.id_search(job_id)
          .map(|j| (j, db))
    }).await??;

    let mut job_entry = match job_entry {
        Some(j) => j,
        None => {
            return Err(NoSuchResource);
        }
    };

    if let JobStatus::Pending { .. } | JobStatus::Running { .. } = job_entry.job_status &&
       let Some(job_status) = get_job_status(job_entry.id).await? {
        if let JobStatus::NotStarted = job_status {
            job_entry.job_status = JobStatus::Failed;
        } else {
            job_entry.job_status = job_status;
        }
    }

    web::block(move || {
        db.update_job_status(job_entry.id, job_entry.job_status).map(|_| db)
    }).await??;

    match job_entry.job_status {
        JobStatus::Finished => {
            Ok(actix_files::NamedFile::open(
                get_result_file_path(job_entry.id)
            )?)
        },
        _ => Err(NoSuchResource)
    }
}

fn do_parse_info_file(req_body: &str) -> Result<PropMap, EndpointError> {
    let tokens = InfoFileLexer::new(req_body).lex()?;
    let mut par = InfoFileParser::new(tokens);
    let ast = par.parse()?;
    Ok(InfoFileToFlatPropertyMapVisitor::new(&ast).visit())
}