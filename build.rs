// build.rs

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Path to your manifest.xml
    let manifest_src = "manifest.xml";

    // Output directory where the executable will be built
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");

    println!("OUT_DIR is set to: {}", out_dir);
    let manifest_dest = Path::new(&out_dir).join("manifest.xml");

    // Copy the manifest.xml to the output directory
    fs::copy(manifest_src, &manifest_dest).expect("Failed to copy manifest.xml");

    println!("cargo:rerun-if-changed={}", manifest_src);
}
