use std::env;

fn main() {
    if env::var("RECOMPILE_PROTO").is_ok() {
        println!("cargo::warning={}", "Compiling proto files");
        tonic_prost_build::configure()
            .out_dir("src")
            .compile_protos(&["proto/pbt_service.proto"], &[])
            .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    } else {
        println!("cargo::warning={}", "Skipping proto compilation");
    }
}
