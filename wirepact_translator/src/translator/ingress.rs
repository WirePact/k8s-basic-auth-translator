use crate::grpc::envoy::config::core::v3::HeaderValue;

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

    pub fn allowed(
        headers_to_add: Vec<HeaderValue>,
        headers_to_remove: Vec<String>,
    ) -> Self {
        Self {
            skip: false,
            forbidden: None,
            headers_to_add,
            headers_to_remove,
        }
    }
}
