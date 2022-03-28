use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::auth::grpc;
use crate::auth::grpc::envoy::config::core::v3::HeaderValue;
use crate::auth::grpc::envoy::service::auth::v3::{CheckRequest, CheckResponse};
use crate::auth::responses::noop_ok_response;

#[tonic::async_trait]
pub trait Translator: Send + Sync {
    async fn ingress(
        &self,
        subject_id: &str,
        request: CheckRequest,
    ) -> Result<IngressResult, Status>;

    async fn egress(&self, request: CheckRequest) -> Result<EgressResult, Status>;
}

pub struct IngressResult {
    skip: bool,
    forbidden: String,
    headers_to_add: Vec<HeaderValue>,
    headers_to_remove: Vec<String>,
}

pub struct EgressResult {
    skip: bool,
    forbidden: String,
    headers_to_add: Vec<HeaderValue>,
    user_id: String,
}

pub struct IngressServer {
    translator: Arc<dyn Translator>,
}

impl IngressServer {
    pub fn new(translator: Arc<dyn Translator>) -> Self {
        IngressServer { translator }
    }
}

#[tonic::async_trait]
impl grpc::envoy::service::auth::v3::authorization_server::Authorization for IngressServer {
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        // let r = self.translator.ingress("", request.into_inner()).await?;

        Ok(Response::new(noop_ok_response()))
    }
}

pub struct EgressServer {
    translator: Arc<dyn Translator>,
}

impl EgressServer {
    pub fn new(translator: Arc<dyn Translator>) -> Self {
        EgressServer { translator }
    }
}

#[tonic::async_trait]
impl grpc::envoy::service::auth::v3::authorization_server::Authorization for EgressServer {
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let r = self.translator.egress(request.into_inner()).await?;

        todo!()
    }
}
