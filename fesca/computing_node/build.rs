fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/share_service.proto")?;
    tonic_build::compile_protos("proto/key_exchange_service.proto")?;
    Ok(())
} 