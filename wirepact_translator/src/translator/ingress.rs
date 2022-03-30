use log::{debug, info};
use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::grpc::envoy::config::core::v3::HeaderValue;
use crate::grpc::envoy::service::auth::v3::authorization_server::Authorization;
use crate::grpc::envoy::service::auth::v3::{CheckRequest, CheckResponse};
use crate::translator::responses::{forbidden_response, ingress_ok_response, noop_ok_response};
use crate::translator::{Translator, WIREPACT_IDENTITY_HEADER};

pub struct IngressResult {
    skip: bool,
    forbidden: Option<String>,
    headers_to_add: Vec<HeaderValue>,
    headers_to_remove: Vec<String>,
}

impl IngressResult {
    pub fn skip() -> Self {
        Self {
            skip: true,
            forbidden: None,
            headers_to_add: Vec::new(),
            headers_to_remove: Vec::new(),
        }
    }

    pub fn forbidden(reason: String) -> Self {
        Self {
            skip: false,
            forbidden: Some(reason),
            headers_to_add: Vec::new(),
            headers_to_remove: Vec::new(),
        }
    }

    pub fn allowed(headers_to_add: Vec<HeaderValue>, headers_to_remove: Vec<String>) -> Self {
        Self {
            skip: false,
            forbidden: None,
            headers_to_add,
            headers_to_remove,
        }
    }
}

pub(crate) struct IngressServer {
    translator: Arc<dyn Translator>,
    // TODO: PKI/JWT
}

impl IngressServer {
    pub(crate) fn new(translator: Arc<dyn Translator>) -> Self {
        IngressServer { translator }
    }
}

#[tonic::async_trait]
impl Authorization for IngressServer {
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        debug!("Received ingress check request.");

        let request = request.get_ref();
        let attributes = request
            .attributes
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("attributes not found"))?;
        let inner_request = attributes
            .request
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("request not found"))?;
        let http = inner_request
            .http
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("http not found"))?;
        let wirepact_jwt = http.headers.get(WIREPACT_IDENTITY_HEADER);

        if wirepact_jwt.is_none() {
            debug!("Skipping. No wirepact JWT found in request.");
            // There is no wirepact JWT, so we can't do anything.
            return Ok(Response::new(noop_ok_response()));
        }

        // TODO: get subject, run ingress translator, parse result

        let ingress_result = self.translator.ingress("TODO", request).await?;

        if ingress_result.skip {
            debug!("Skipping ingress request.");
            return Ok(Response::new(noop_ok_response()));
        }

        if let Some(reason) = ingress_result.forbidden {
            info!("Request is forbidden, reason: {}.", reason);
            return Ok(Response::new(forbidden_response(reason.as_str())));
        }

        debug!("Ingress request is allowed.");
        Ok(Response::new(ingress_ok_response(
            ingress_result.headers_to_add,
            ingress_result.headers_to_remove,
        )))
    }
}
