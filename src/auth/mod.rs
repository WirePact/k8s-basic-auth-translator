use std::error::Error;
use std::sync::Arc;

use log::info;
use tokio::try_join;
use tonic::transport::Server;

pub use grpc::envoy::service::auth::v3::{CheckRequest, CheckResponse};
pub use server::{EgressResult, IngressResult, Translator};

use crate::auth::server::{EgressServer, IngressServer};

mod grpc;
pub mod responses;
mod server;

pub async fn create_servers(
    ingress_port: u16,
    egress_port: u16,
    translator: Arc<dyn Translator>,
) -> Result<(), Box<dyn Error>> {
    let ingress_address = format!("0.0.0.0:{}", ingress_port);
    info!("Creating and starting ingress server @ {}", ingress_address);
    let ingress = Server::builder().add_service(
        grpc::envoy::service::auth::v3::authorization_server::AuthorizationServer::new(
            IngressServer::new(translator.clone()),
        ),
    );

    let egress_address = format!("0.0.0.0:{}", egress_port);
    info!("Creating and starting egress server @ {}", egress_address);
    let egress = Server::builder().add_service(
        grpc::envoy::service::auth::v3::authorization_server::AuthorizationServer::new(
            EgressServer::new(translator.clone()),
        ),
    );

    try_join!(
        ingress.serve(ingress_address.parse()?),
        egress.serve(egress_address.parse()?),
    )
    .await?;
    Ok(())
}
