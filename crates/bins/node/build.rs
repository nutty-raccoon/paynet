use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("node_service_desciptor.bin"))
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
