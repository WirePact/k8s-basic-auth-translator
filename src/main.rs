mod auth;

use clap::{ArgEnum, Parser};
use log::info;

#[derive(Clone, Debug, ArgEnum)]
enum Mode {
    Csv,
    Kubernetes,
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Cli {
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
    k8s_secret_name: String,

    /// If `mode` is set to `csv`, this is the path to the csv file
    /// that is used to translate user credentials.
    #[clap(short, long, env)]
    csv_path: String,

    /// If set, debug log messages are printed as well.
    #[clap(short, long, env)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    env_logger::builder()
        .filter_level(match cli.debug {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Info,
        })
        .init();

    Ok(())
}
