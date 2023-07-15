use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=shader");
    std::process::Command::new("C:\\VulkanSDK\\1.3.250.1\\Bin\\glslc.exe")
        .args(&["-c", "shader\\shader.vert", "-o", "shader\\vert.spv"])
        .output()?;
    std::process::Command::new("C:\\VulkanSDK\\1.3.250.1\\Bin\\glslc.exe")
        .args(&["-c", "shader\\shader.frag", "-o", "shader\\frag.spv"])
        .output()?;
    SpirvBuilder::new("shader", "spirv-unknown-vulkan1.0")
        .print_metadata(MetadataPrintout::Full)
        .build()?;
    Ok(())
}
