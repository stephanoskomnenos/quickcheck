fn main() {
    // tonic_prost_build::configure()
    // .out_dir("src")
    // .compile_protos(&["proto/pbt_service.proto"], &[])
    //     .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    tonic_prost_build::compile_protos("proto/pbt_service.proto")
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}