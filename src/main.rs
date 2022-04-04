use std::sync::Arc;

use clap::{ArgEnum, Parser};
use log::{debug, info};

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

    run_translator(&TranslatorConfig {
        pki_address: cli.pki_address,
        common_name: cli.name,
        ingress_port: cli.ingress_port,
        egress_port: cli.egress_port,
        translator: Arc::new(BasicAuthTranslator {}),
    })
    .await?;

    Ok(())
}

struct BasicAuthTranslator {}

#[wirepact_translator::async_trait]
impl Translator for BasicAuthTranslator {
    async fn ingress(
        &self,
        _subject_id: &str,
        _request: &CheckRequest,
    ) -> Result<IngressResult, Status> {
        Ok(IngressResult::skip())
    }

    async fn egress(&self, _request: &CheckRequest) -> Result<EgressResult, Status> {
        Ok(EgressResult::allowed("1".to_string(), vec![]))
    }
}
