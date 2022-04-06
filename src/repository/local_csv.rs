use std::error::Error;

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
    async fn lookup_id(&self, username: &str, password: &str) -> Result<Option<String>, dyn Error> {
        todo!()
    }

    async fn lookup_user(&self, user_id: &str) -> Result<Option<(String, String)>, dyn Error> {
        todo!()
    }
}
