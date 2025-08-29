pub mod error;

use crate::database::error::DatabaseError;
use crate::info_file_parser::flat_property_map::PropMap;
use crate::job::{get_jobs_dir, JobStatus};
use polodb_core::bson::{doc, to_bson, Document};
use polodb_core::results::{InsertOneResult, UpdateResult};
use polodb_core::{Collection, CollectionT, Database, Transaction, TransactionalCollection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct JobEntry {
    pub info_parameters: PropMap,
    pub num_info_parameters: usize,
    pub job_status: JobStatus,
    pub id: Uuid
}

pub trait JobDbOperations {
    fn exact_parameter_search(&self, job: &JobEntry) -> Result<Option<JobEntry>, DatabaseError>;
    fn id_search(&self, id: Uuid) -> Result<Option<JobEntry>, DatabaseError>;
    fn insert_job(&mut self, job: &JobEntry) -> Result<InsertOneResult, DatabaseError>;
    fn update_job_status(&mut self, id: Uuid, status: JobStatus) -> Result<UpdateResult, DatabaseError>;
}

impl<C: CollectionT<JobEntry>> JobDbOperations for C {
    fn exact_parameter_search(&self, job: &JobEntry) -> Result<Option<JobEntry>, DatabaseError> {
        let mut doc = Document::from_iter(
            job.info_parameters.iter()
                               .map(|(k, v)| (format!("info_parameters.{k}"), to_bson(v).unwrap()))
        );

        doc.insert("num_info_parameters", to_bson(&job.num_info_parameters).unwrap());

        self.find_one(doc)
            .map_err(|e| e.into())
    }

    fn id_search(&self, id: Uuid) -> Result<Option<JobEntry>, DatabaseError> {
        self.find_one(doc! {
            "id": to_bson(&id).unwrap()
        }).map_err(|e| e.into())
    }

    fn insert_job(&mut self, job: &JobEntry) -> Result<InsertOneResult, DatabaseError> {
        self.insert_one(job)
            .map_err(|e| e.into())
    }

    fn update_job_status(&mut self, id: Uuid, status: JobStatus) -> Result<UpdateResult, DatabaseError> {
        self.update_one(doc! {
            "id": to_bson(&id).unwrap()
        }, doc! {
            "$set": {
                "job_status": to_bson(&status).unwrap()
            }
        })
        .map_err(|e| e.into())
    }
}

pub struct JobDb {
    db: Database,
    collection: Collection<JobEntry>
}

impl JobDbOperations for JobDb {
    fn exact_parameter_search(&self, job: &JobEntry) -> Result<Option<JobEntry>, DatabaseError> {
        self.collection.exact_parameter_search(job)
    }

    fn id_search(&self, id: Uuid) -> Result<Option<JobEntry>, DatabaseError> {
        self.collection.id_search(id)
    }

    fn insert_job(&mut self, job: &JobEntry) -> Result<InsertOneResult, DatabaseError> {
        self.collection.insert_job(job)
    }

    fn update_job_status(&mut self, id: Uuid, status: JobStatus) -> Result<UpdateResult, DatabaseError> {
        self.collection.update_job_status(id, status)
    }
}

pub struct JobTransaction {
    db: Database,
    txn: Transaction,
    collection: TransactionalCollection<JobEntry>
}

impl JobTransaction {
    pub fn commit(&mut self) -> Result<(), DatabaseError> {
        self.txn.commit().map_err(|e| e.into())
    }

    pub fn rollback(&mut self) -> Result<(), DatabaseError> {
        self.txn.rollback().map_err(|e| e.into())
    }
}

impl JobDbOperations for JobTransaction {
    fn exact_parameter_search(&self, job: &JobEntry) -> Result<Option<JobEntry>, DatabaseError> {
        self.collection.exact_parameter_search(job)
    }

    fn id_search(&self, id: Uuid) -> Result<Option<JobEntry>, DatabaseError> {
        self.collection.id_search(id)
    }

    fn insert_job(&mut self, job: &JobEntry) -> Result<InsertOneResult, DatabaseError> {
        self.collection.insert_job(job)
    }

    fn update_job_status(&mut self, id: Uuid, status: JobStatus) -> Result<UpdateResult, DatabaseError> {
        self.collection.update_job_status(id, status)
    }
}

impl JobEntry {
    pub fn new(info_parameters: PropMap) -> Self {
        Self {
            num_info_parameters: info_parameters.len(),
            info_parameters,
            job_status: JobStatus::NotStarted,
            id: Uuid::new_v4(),
        }
    }
}

pub fn open_job_db() -> Result<JobDb, DatabaseError> {
    Database::open_path(Path::new(&get_jobs_dir()).join("database"))
             .map(|db| JobDb { collection: db.collection::<JobEntry>("jobs"), db })
             .map_err(|e| e.into())
}

pub fn open_job_transaction() -> Result<JobTransaction, DatabaseError> {
    Database::open_path(Path::new(&get_jobs_dir()).join("database"))
             .and_then(|db| db.start_transaction().map(|txn| (txn, db)))
             .map(|(txn, db)| JobTransaction { db, collection: txn.collection::<JobEntry>("jobs"), txn })
             .map_err(|e| e.into())
}