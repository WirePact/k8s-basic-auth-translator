use crate::auth::grpc::envoy::service::auth::v3::check_response::HttpResponse;
use crate::auth::grpc::envoy::service::auth::v3::OkHttpResponse;
use crate::auth::grpc::google::rpc::Status;
use crate::auth::CheckResponse;

const GRPC_OK: i32 = 0;

pub fn noop_ok_response() -> CheckResponse {
    CheckResponse {
        status: Some(Status {
            code: GRPC_OK,
            message: "".to_string(),
            details: vec![],
        }),
        dynamic_metadata: None,
        http_response: Some(HttpResponse::OkResponse(OkHttpResponse {
            headers: vec![],
            headers_to_remove: vec![],
            dynamic_metadata: None,
            response_headers_to_add: vec![],
            query_parameters_to_set: vec![],
            query_parameters_to_remove: vec![],
        })),
    }
}
