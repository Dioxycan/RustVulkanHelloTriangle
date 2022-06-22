use ash::vk;
use ash::version::DeviceV1_0;
use std::{fs::File, io::Read};
pub fn create_graphic_pipeline(device:&ash::Device) {
    let vert_shader_code = read_shader("src/shaders/shader.vert.spv");
    let frag_shader_code = read_shader("src/shaders/shader.frag.spv");

    let vert_shader_module = create_shader_module(device,&vert_shader_code[..]);
    let frag_shader_module = create_shader_module(device,&frag_shader_code[..]);
    let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        module: vert_shader_module,
        p_name: b"main\0".as_ptr() as *const _,
        ..Default::default()
    };
    let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        module: frag_shader_module,
        p_name: b"main\0".as_ptr() as *const _,
        ..Default::default()
    };
    unsafe{
        device.destroy_shader_module(vert_shader_module,None);
        device.destroy_shader_module(frag_shader_module,None);
    }
}
fn read_shader(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).expect("Failed to open shader file");
    let mut contents: Vec<u8> = vec![];
    file.read_to_end(&mut contents)
        .expect("Failed to read shader file");
    contents
}
fn create_shader_module(device: &ash::Device, code: &[u8]) -> vk::ShaderModule {
    let shader_module_create_info = 
        vk::ShaderModuleCreateInfo{
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
            ..Default::default()
        }
    unsafe {
        device
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create shader module")
    }
}
