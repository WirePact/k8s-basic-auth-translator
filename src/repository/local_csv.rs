use std::error::Error;

use log::debug;
use serde::Deserialize;

use crate::repository::Repository;

#[derive(Debug, Deserialize)]
struct CsvEntry {
    id: String,
    username: String,
    password: String,
}

pub(crate) struct LocalCsvRepository {
    entries: Vec<CsvEntry>,
}

impl LocalCsvRepository {
    pub(crate) fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        debug!("Loading local CSV repository from {}.", path);
        let mut reader = csv::Reader::from_path(path)?;
        let mut entries = Vec::new();
        for record in reader.deserialize() {
            let entry: CsvEntry = record?;
            entries.push(entry);
        }

        Ok(Self { entries })
    }
}

#[wirepact_translator::async_trait]
impl Repository for LocalCsvRepository {
    async fn lookup_id(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        for entry in &self.entries {
            if entry.username == username && entry.password == password {
                return Ok(Some(entry.id.clone()));
            }
        }

        Ok(None)
    }

    async fn lookup_user(&self, user_id: &str) -> Result<Option<(String, String)>, Box<dyn Error>> {
        for entry in &self.entries {
            if entry.id == user_id {
                return Ok(Some((entry.username.clone(), entry.password.clone())));
            }
        }

        Ok(None)
    }
}
