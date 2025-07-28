use std::fs;
use std::path::Path;

fn main() {
    let url = "https://spkg.io/tdelabro/ethereum-invoice-substream-v0.1.0.spkg";
    let output_path = "ethereum-invoice-substream-v0.1.0.spkg";

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../../substreams/ethereum/proto/invoice_contract.proto");

    // Generate protobuf files from the Ethereum substream proto files
    let mut config = prost_build::Config::new();
    config.out_dir("src/pb");

    // Generate the invoice_contract protobuf
    config
        .compile_protos(
            &["../../../substreams/ethereum/proto/invoice_contract.proto"],
            &["../../../substreams/ethereum/proto/"],
        )
        .expect("Failed to compile invoice_contract.proto");

    // Only download if file doesn't exist
    if !Path::new(output_path).exists() {
        let response = reqwest::blocking::get(url).expect("Failed to download file");

        let content = response.bytes().expect("Failed to read response body");

        fs::write(output_path, content).expect("Failed to write file");
    }
}
