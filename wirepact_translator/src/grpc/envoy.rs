pub mod service {
    pub mod auth {
        pub mod v3 {
            tonic::include_proto!("envoy.service.auth.v3");
        }
    }
}

pub mod config {
    pub mod core {
        pub mod v3 {
            tonic::include_proto!("envoy.config.core.v3");
        }
    }
}

pub mod r#type {
    pub mod v3 {
        tonic::include_proto!("envoy.r#type.v3");
    }
}
