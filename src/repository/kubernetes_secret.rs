use std::env;
use std::error::Error;
use std::path::Path;

use k8s_openapi::api::core::v1::Secret;
use kube::config::Kubeconfig;
use kube::{Api, Client};
use log::debug;
use tokio::fs::read_to_string;

use crate::Repository;

const DEFAULT_NAMESPACE: &str = "default";
const DOWNWARD_API_ENV: &str = "POD_NAMESPACE";
const DOWNWARD_API_FILE: &str = "/var/run/secrets/kubernetes.io/serviceaccount/namespace";

struct KubernetesEntry {
    id: String,
    username: String,
    password: String,
}

pub(crate) struct KubernetesSecretRepository {
    secret_name: String,
}

impl KubernetesSecretRepository {
    pub(crate) async fn new(secret_name: &str) -> Result<Self, Box<dyn Error>> {
        debug!(
            "Creating KubernetesSecretRepository with secret_name: {}.",
            secret_name
        );

        let result = Self {
            secret_name: secret_name.to_string(),
        };
        result.load_entries().await?;

        Ok(result)
    }

    async fn current_namespace() -> Result<String, Box<dyn Error>> {
        if let Ok(config) = Kubeconfig::read() {
            let default_context = "".to_string();
            let current_context_name = config.current_context.as_ref().unwrap_or(&default_context);
            let current_namespace = config
                .contexts
                .iter()
                .find(|&ctx| ctx.name == *current_context_name)
                .expect("No context with name found.")
                .clone()
                .context
                .namespace
                .unwrap_or_else(|| "".to_string());

            if !current_namespace.is_empty() {
                return Ok(current_namespace);
            }
        }

        if let Ok(value) = env::var(DOWNWARD_API_ENV) {
            return Ok(value);
        }

        let path = Path::new(DOWNWARD_API_FILE);
        if path.exists() {
            let content = read_to_string(path).await?;
            return Ok(content.trim().to_string());
        }

        Ok(DEFAULT_NAMESPACE.to_string())
    }

    async fn load_entries(&self) -> Result<Vec<KubernetesEntry>, Box<dyn Error>> {
        let client = Client::try_default().await?;
        let secrets = Api::namespaced(
            client,
            &KubernetesSecretRepository::current_namespace().await?,
        );
        let secret: Secret = secrets.get(&self.secret_name).await?;
        let entries = secret
            .data
            .as_ref()
            .unwrap()
            .iter()
            .map::<Result<KubernetesEntry, Box<dyn Error>>, _>(|(key, value)| {
                let id = key.to_string();
                let payload = String::from_utf8(value.clone().0)?;
                let auth_pair = payload.split(':').collect::<Vec<&str>>();

                Ok(KubernetesEntry {
                    id,
                    username: auth_pair[0].to_string(),
                    password: auth_pair[1].to_string(),
                })
            })
            .collect::<Result<Vec<KubernetesEntry>, _>>()?;

        Ok(entries)
    }
}

#[wirepact_translator::async_trait]
impl Repository for KubernetesSecretRepository {
    async fn lookup_id(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let entries = self.load_entries().await?;
        for entry in entries {
            if entry.username == username && entry.password == password {
                return Ok(Some(entry.id));
            }
        }

        Ok(None)
    }

    async fn lookup_user(&self, user_id: &str) -> Result<Option<(String, String)>, Box<dyn Error>> {
        let entries = self.load_entries().await?;
        for entry in entries {
            if entry.id == user_id {
                return Ok(Some((entry.username, entry.password)));
            }
        }

        Ok(None)
    }
}
