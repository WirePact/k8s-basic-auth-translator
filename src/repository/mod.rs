use std::error::Error;

pub(crate) use kubernetes_secret::KubernetesSecretRepository;
pub(crate) use local_csv::LocalCsvRepository;

mod kubernetes_secret;
mod local_csv;

#[wirepact_translator::async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn lookup_id(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<String>, Box<dyn Error>>;
    async fn lookup_user(&self, user_id: &str) -> Result<Option<(String, String)>, Box<dyn Error>>;
}
