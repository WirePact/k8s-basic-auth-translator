extern crate core;

use std::error::Error;
use std::sync::Arc;

use log::{debug, info};
use tokio::signal::ctrl_c;
use tokio::try_join;
use tonic::transport::Server;
pub use tonic::{async_trait, Status};

pub use grpc::envoy::service::auth::v3::CheckRequest;
pub use translator::{
    EgressResult, IngressResult, Translator, HTTP_AUTHORIZATION_HEADER, WIREPACT_IDENTITY_HEADER,
};

use crate::pki::Pki;
use crate::translator::egress::EgressServer;
use crate::translator::ingress::IngressServer;

mod grpc;
mod pki;
mod translator;

pub struct TranslatorConfig {
    pub pki_address: String,
    pub common_name: String,
    pub ingress_port: u16,
    pub egress_port: u16,
    pub translator: Arc<dyn Translator>,
}

pub async fn run_translator(config: &TranslatorConfig) -> Result<(), Box<dyn Error>> {
    debug!("Initializing PKI.");
    let pki = Arc::new(Pki::new(&config.pki_address, &config.common_name).await?);

    let ingress_address = format!("0.0.0.0:{}", config.ingress_port);
    info!("Creating and starting ingress server @ {}", ingress_address);
    let ingress = Server::builder().add_service(
        grpc::envoy::service::auth::v3::authorization_server::AuthorizationServer::new(
            IngressServer::new(config.translator.clone(), pki.clone()),
        ),
    );

    let egress_address = format!("0.0.0.0:{}", config.egress_port);
    info!("Creating and starting egress server @ {}", egress_address);
    let egress = Server::builder().add_service(
        grpc::envoy::service::auth::v3::authorization_server::AuthorizationServer::new(
            EgressServer::new(config.translator.clone(), pki.clone()),
        ),
    );

    try_join!(
        ingress.serve_with_shutdown(ingress_address.parse()?, async {
            let _ = ctrl_c().await;
            info!("Received signal. Shutting down server.");
            {}
        }),
        egress.serve_with_shutdown(egress_address.parse()?, async {
            let _ = ctrl_c().await;
            {}
        }),
    )?;

    Ok(())
}
