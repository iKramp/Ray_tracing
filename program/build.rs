use spirv_builder::{SpirvBuilder, MetadataPrintout, Capability::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("../shader", "spirv-unknown-vulkan1.0")
        .print_metadata(MetadataPrintout::Full)
        .capability(Int8)
        .extension("SPV_KHR_storage_buffer_storage_class")
        .build()?;
    Ok(())
}
