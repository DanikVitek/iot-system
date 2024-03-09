use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "tonic")]
    tonic_build::compile_protos("proto/iot_system.proto")?;

    Ok(())
}
