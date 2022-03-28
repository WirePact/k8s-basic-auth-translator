use parking_lot::RwLock;
use std::borrow::Borrow;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::auth::grpc;
use crate::auth::grpc::envoy::config::core::v3::HeaderValue;
use crate::auth::grpc::envoy::service::auth::v3::{CheckRequest, CheckResponse};

#[tonic::async_trait]
trait IngressTranslator: Send + Sync {
    async fn translate(
        &self,
        subject_id: &str,
        request: CheckRequest,
    ) -> Result<CheckResponse, Status>;
}

pub struct IngressResult {
    skip: bool,
    forbidden: String,
    headers_to_add: Vec<HeaderValue>,
    headers_to_remove: Vec<String>,
}

pub struct IngressServer {
    translator: dyn IngressTranslator,
}

#[tonic::async_trait]
impl grpc::envoy::service::auth::v3::authorization_server::Authorization for IngressServer {
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let r = self.translator.borrow();
        let a = r.translate("", request.into_inner()).await?;
        todo!()
    }
}
