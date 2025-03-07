fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../../proto");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("Generated files will be stored in: {}", out_dir);

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(
            &[
                "../../../proto/node.proto",
                "../../../proto/bdhke.proto",
                "../../../proto/keyset_rotation.proto",
            ],
            &["../../../proto"],
        )?;
    Ok(())
}
