use std::fs::create_dir_all;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = PathBuf::new();

    let proto_dir = &[current_dir.join("proto")];
    let out_dir = current_dir.join("src").join("proto");
    create_dir_all(out_dir.clone())?;

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("auction_descriptor.bin"))
        .out_dir(out_dir)
        .compile(&[current_dir.join("proto/auction.proto")], proto_dir)?;

    Ok(())
}
