use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = PathBuf::new();
    let protos = &[current_dir.join("proto/auction.proto")];
    let proto_dir = &[current_dir.join("proto")];
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("auction_descriptor.bin"))
        .out_dir(out_dir)
        .compile(protos, proto_dir)?;

    Ok(())
}
