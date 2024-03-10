use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "tonic")]
    {
        use std::{env, path::PathBuf};

        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        tonic_build::configure()
            .file_descriptor_set_path(out_dir.join("iot_system_descriptor.bin"))
            .compile(&["proto/iot_system.proto"], &["proto"])?;

        tonic_build::compile_protos("proto/iot_system.proto")?;
    }

    Ok(())
}
