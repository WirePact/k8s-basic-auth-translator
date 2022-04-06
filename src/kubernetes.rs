use kube::config::Kubeconfig;
use std::env;
use std::error::Error;
use std::path::Path;
use tokio::fs::read_to_string;

const DEFAULT_NAMESPACE: &str = "default";
const DOWNWARD_API_ENV: &str = "POD_NAMESPACE";
const DOWNWARD_API_FILE: &str = "/var/run/secrets/kubernetes.io/serviceaccount/namespace";

pub(crate) async fn current_namespace() -> Result<String, Box<dyn Error>> {
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
