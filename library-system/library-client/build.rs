fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../buf/");

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["../buf/library/v1/library.proto"], &["../buf"])?;
    Ok(())
}
