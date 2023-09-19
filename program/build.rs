use spirv_builder::Capability::{Float64, Int64, Int8};
use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("../shader", "spirv-unknown-vulkan1.0")
        .print_metadata(MetadataPrintout::Full)
        .capability(Float64)
        .capability(Int64)
        .capability(Int8)
        .extension("SPV_KHR_storage_buffer_storage_class")
        .build()?;
    Ok(())
}
