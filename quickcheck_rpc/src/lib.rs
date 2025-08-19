mod pbt_service {
    tonic::include_proto!("pbt_service");
}

pub use pbt_service::{ExecuteRequest, ExecuteResponse, property_tester_client, property_tester_server};