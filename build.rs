fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/proto_out")
        .compile(
            &["proto/geyser.proto", "proto/solana-storage.proto"],
            &["proto"],
        )?;
    Ok(())
}
