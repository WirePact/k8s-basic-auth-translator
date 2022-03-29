use crate::grpc::envoy::config::core::v3::HeaderValue;

pub struct EgressResult {
    skip: bool,
    forbidden: Option<String>,
    headers_to_add: Vec<HeaderValue>,
    user_id: Option<String>,
}

impl EgressResult {
    pub fn skip() -> Self {
        Self {
            skip: true,
            forbidden: None,
            headers_to_add: Vec::new(),
            user_id: None,
        }
    }

    pub fn forbidden(reason: String) -> Self {
        Self {
            skip: false,
            forbidden: Some(reason),
            headers_to_add: Vec::new(),
            user_id: None,
        }
    }

    pub fn no_user_id() -> Self {
        Self {
            skip: false,
            forbidden: Some("No UserID given for outbound communication.".to_string()),
            headers_to_add: Vec::new(),
            user_id: None,
        }
    }

    pub fn allowed(
        user_id: String,
        headers_to_add: Vec<HeaderValue>,
    ) -> Self {
        Self {
            skip: false,
            forbidden: none,
            headers_to_add,
            user_id: Some(user_id),
        }
    }
}
