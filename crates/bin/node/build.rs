fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
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
