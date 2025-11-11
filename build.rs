fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = "proto";
    let protos: Vec<_> = ["token"]
        .iter()
        .map(|f| format!("{}/{}.proto", proto_dir, f))
        .collect();

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&protos, &[proto_dir.to_string()])?;

    Ok(())
}
