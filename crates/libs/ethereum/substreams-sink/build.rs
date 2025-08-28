use std::fs;
use std::path::Path;

fn main() {
    let url = "https://spkg.io/JoE11-y/eth-invoice-substreams-v0.1.6.spkg";
    let output_path = "eth-invoice-substream-v0.1.6.spkg";

    println!("cargo:rerun-if-changed=build.rs");

    // Only download if file doesn't exist
    if !Path::new(output_path).exists() {
        let response = reqwest::blocking::get(url).expect("Failed to download file");

        let content = response.bytes().expect("Failed to read response body");

        fs::write(output_path, content).expect("Failed to write file");
    }
}
