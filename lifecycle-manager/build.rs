use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: run this only with cargo test
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"));

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("file_descriptor_set.bin"))
        .compile(&["proto/test/TestService.proto"], &["proto/test"])?;

    Ok(())
}
