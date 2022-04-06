mod kubernetes;
mod repository;

use std::sync::Arc;

use clap::{ArgEnum, Parser};
use log::{debug, error, info, warn};

use crate::repository::{LocalCsvRepository, Repository};
use wirepact_translator::{
    run_translator, CheckRequest, EgressResult, IngressResult, Status, Translator, TranslatorConfig,
};

#[derive(Clone, Debug, ArgEnum)]
enum Mode {
    Csv,
    Kubernetes,
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// The address of the WirePact PKI.
    #[clap(short, long, env)]
    pki_address: String,

    /// The name of the translator. This is used as common name
    /// when requesting a certificate from the PKI.
    #[clap(short, long, env, default_value = "k8s basic auth translator")]
    name: String,

    /// The port that the server will listen for
    /// ingress communication (incoming connections) on.
    #[clap(short, long, env, default_value = "50051")]
    ingress_port: u16,

    /// The port that the server will listen for
    /// egress communication (outgoing connections) on.
    #[clap(short, long, env, default_value = "50052")]
    egress_port: u16,

    /// Defines the mode that the translator runs in.
    /// The following modes are possible:
    /// - `csv`: Use a csv file with a certain structure to
    ///   translate user credentials into a user ID and vice versa.
    /// - `kubernetes`: Use a Kubernetes secret to translate
    ///   user credentials into a user ID and vice versa.
    #[clap(arg_enum, short, long, env, default_value = "csv")]
    mode: Mode,

    /// If `mode` is set to `kubernetes`, this is the name of the
    /// Kubernetes secret that is used to translate user credentials.
    #[clap(short, long, env)]
    k8s_secret_name: Option<String>,

    /// If `mode` is set to `csv`, this is the path to the csv file
    /// that is used to translate user credentials.
    #[clap(short, long, env)]
    csv_path: Option<String>,

    /// If set, debug log messages are printed as well.
    #[clap(short, long, env)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let level = match cli.debug {
        true => log::LevelFilter::Debug,
        false => log::LevelFilter::Info,
    };

    env_logger::builder()
        .filter_module("k8s_basic_auth_translator", level)
        .filter_module("wirepact_translator", level)
        .init();

    info!(
        "Starting basic auth translator '{}' in {:?} mode.",
        cli.name, cli.mode
    );
    debug!("Debug logging is enabled.");

    let repository = match cli.mode {
        Mode::Csv => {
            if cli.csv_path.is_none() {
                error!("No CSV path provided.");
                return Err("No CSV path provided.".into());
            }

            LocalCsvRepository::new(&cli.csv_path.unwrap())
        }
        Mode::Kubernetes => Err("Not implemented yet.".into()),
    }?;

    run_translator(&TranslatorConfig {
        pki_address: cli.pki_address,
        common_name: cli.name,
        ingress_port: cli.ingress_port,
        egress_port: cli.egress_port,
        translator: Arc::new(BasicAuthTranslator {
            repository: Arc::new(repository),
        }),
    })
    .await?;

    Ok(())
}

struct BasicAuthTranslator {
    repository: Arc<dyn Repository>,
}

#[wirepact_translator::async_trait]
impl Translator for BasicAuthTranslator {
    async fn ingress(
        &self,
        subject_id: &str,
        request: &CheckRequest,
    ) -> Result<IngressResult, Status> {
        let user_information = self.repository.lookup_user(subject_id).await;
        if user_information.is_none() {
            warn!("No user information found for subject ID '{}'.", subject_id);
            return Ok(IngressResult::forbidden(
                "No user information found.".to_string(),
            ));
        }

        let (username, password) = user_information.unwrap();
        debug!(
            "Found user information for subject ID '{}': '{}'.",
            subject_id, username
        );

        Ok(IngressResult::allowed(
            Some(vec![(
                wirepact_translator::HTTP_AUTHORIZATION_HEADER.to_string(),
                format!(
                    "Basic {}",
                    base64::encode(&format!("{}:{}", username, password)),
                ),
            )]),
            None,
        ))
    }

    async fn egress(&self, request: &CheckRequest) -> Result<EgressResult, Status> {
        let auth_header =
            self.get_header(request, wirepact_translator::HTTP_AUTHORIZATION_HEADER)?;

        if auth_header.is_none() {
            debug!("No authorization header found. Skip request.");
            return Ok(EgressResult::skip());
        }

        let auth_header = auth_header.unwrap();
        if !auth_header.starts_with("Basic ") {
            debug!("Authorization header does not start with 'Basic'. Skip Request.");
            return Ok(EgressResult::skip());
        }

        debug!("Request contains Basic Auth Header. Create JWT token.");

        let payload = base64::decode(auth_header.replace("Basic ", ""))?.to_str()?;
        let auth_pair = payload.split(':').collect::<Vec<&str>>();

        if auth_pair.len() != 2 {
            warn!("Authorization header does not contain a valid username and password pair. Received: {}.", payload);
            return Ok(EgressResult::forbidden(
                "Basic Auth data is corrupted.".to_string(),
            ));
        }

        if let Some(user_id) = self.repository.lookup_id(auth_pair[0], auth_pair[1]).await {
            debug!("Found user ID '{}' for basic auth credentials.", user_id);
            return Ok(EgressResult::allowed(
                user_id,
                Some(vec![
                    wirepact_translator::HTTP_AUTHORIZATION_HEADER.to_string()
                ]),
            ));
        }

        warn!("No user found for Basic Auth credentials.");
        Ok(EgressResult::no_user_id())
    }
}
