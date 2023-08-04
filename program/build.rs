use spirv_builder::Capability::Float64;
use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("../shader", "spirv-unknown-vulkan1.0")
        .print_metadata(MetadataPrintout::Full)
        .capability(Float64)
        .build()?;
    Ok(())
}
