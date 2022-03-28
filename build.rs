const INCLUDES: &[&str; 4] = &[
    "external/googleapis",
    "external/envoy/api",
    "external/udpa",
    "external/protoc-gen-validate",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["external/envoy/api/envoy/service/auth/v3/external_auth.proto"],
            INCLUDES,
        )?;

    Ok(())
}
