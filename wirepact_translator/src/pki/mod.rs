// use case pki:
// fetch CA and store it locally
// create local cert via CSR from CA
// sign JWT

pub(crate) struct Pki {}

impl Pki {
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// Initialize the PKI by fetching the CA certificate if
    /// needed and creating/loading a local private key with the
    /// certificate. If no local certificate is present, a new one
    /// is created via CSR from the CA.
    pub(crate) async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
