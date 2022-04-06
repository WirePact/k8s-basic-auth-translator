mod kubernetes_secret;
mod local_csv;

pub(crate) use local_csv::LocalCsvRepository;
use std::error::Error;

#[wirepact_translator::async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn lookup_id(&self, username: &str, password: &str) -> Result<Option<String>, dyn Error>;
    async fn lookup_user(&self, user_id: &str) -> Result<Option<(String, String)>, dyn Error>;
}
