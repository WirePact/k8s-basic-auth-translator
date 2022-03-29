use crate::grpc::envoy::service::auth::v3::CheckRequest;
use crate::translator::ingress::IngressResult;

pub(crate) const WIREPACT_IDENTITY_HEADER: &str = "x-wirepact-identity";
pub(crate) const HTTP_AUTHORIZATION_HEADER: &str = "authorization";

#[tonic::async_trait]
pub trait Translator: Send + Sync {
    async fn ingress(
        &self,
        subject_id: &str,
        request: CheckRequest,
    ) -> Result<IngressResult, Status>;

    async fn egress(&self, request: CheckRequest) -> Result<EgressResult, Status>;
}
