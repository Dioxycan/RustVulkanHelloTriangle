use ash::version::DeviceV1_0;
use ash::vk;
use std::ptr;
use std::{fs::File, io::Read};
pub fn create_render_pass(
    device: &ash::Device,
    swap_chain_image_format: vk::Format,
) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription {
        format: swap_chain_image_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };
    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };
    let subpass = vk::SubpassDescription {
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        p_color_attachments: &color_attachment_ref,
        color_attachment_count:1,
        ..Default::default()
    };
    let subpass_dependency = vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: vk::AccessFlags::empty(),
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        ..Default::default()
    };
    let create_info = vk::RenderPassCreateInfo {
        attachment_count: 1,
        p_attachments: &color_attachment,
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 1,
        p_dependencies: &subpass_dependency,
        ..Default::default()
    };
    unsafe {
        device
            .create_render_pass(&create_info, None)
            .expect("Failed to create render pass")
    }
}

pub fn create_graphic_pipeline(
    device: &ash::Device,
    swap_chain_extent: &vk::Extent2D,
    render_pass: &vk::RenderPass,
) -> (vk::Pipeline, vk::PipelineLayout) {
    let vert_shader_code = read_shader("src/shaders/shader.vert.spv");
    let frag_shader_code = read_shader("src/shaders/shader.frag.spv");

    let vert_shader_module = create_shader_module(device, &vert_shader_code[..]);
    let frag_shader_module = create_shader_module(device, &frag_shader_code[..]);
    let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        module: vert_shader_module,
        p_name: b"main\0".as_ptr() as *const _,
        stage: vk::ShaderStageFlags::VERTEX,
        ..Default::default()
    };
    let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        module: frag_shader_module,
        p_name: b"main\0".as_ptr() as *const _,
        stage: vk::ShaderStageFlags::FRAGMENT,
        ..Default::default()
    };
    let shader_stages = 
    [vert_shader_stage_info, frag_shader_stage_info];

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: ptr::null(),
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: ptr::null(),
        ..Default::default()
    };
    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    };
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: swap_chain_extent.width as f32,
        height: swap_chain_extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };
    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: *swap_chain_extent,
    };
    let viewport_state = vk::PipelineViewportStateCreateInfo {
        viewport_count: 1,
        p_viewports: &viewport,
        scissor_count: 1,
        p_scissors: &scissor,
        ..Default::default()
    };
    let rasterizer = vk::PipelineRasterizationStateCreateInfo {
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: vk::PolygonMode::FILL,
        line_width: 1.0,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::CLOCKWISE,
        depth_bias_enable: vk::FALSE,
        depth_bias_constant_factor: 0.0,
        depth_bias_clamp: 0.0,
        depth_bias_slope_factor: 0.0,
        ..Default::default()
    };
    let multisampling = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 1.0,
        p_sample_mask: ptr::null(),
        alpha_to_coverage_enable: vk::FALSE,
        alpha_to_one_enable: vk::FALSE,
        ..Default::default()
    };
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::all(),
        blend_enable: vk::FALSE,
        src_color_blend_factor: vk::BlendFactor::ONE,
        dst_color_blend_factor: vk::BlendFactor::ZERO,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ONE,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        ..Default::default()
    };
    let color_blend_info = vk::PipelineColorBlendStateCreateInfo {
        attachment_count: 1,
        p_attachments: &color_blend_attachment,
        ..Default::default()
    };
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
        set_layout_count: 0,
        p_set_layouts: ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: ptr::null(),
        ..Default::default()
    };
    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .expect("Failed to create pipeline layout!")
    };
    let create_info = vk::GraphicsPipelineCreateInfo {
        stage_count: 2,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_info,
        p_input_assembly_state: &input_assembly_info,
        p_viewport_state: &viewport_state,
        p_rasterization_state: &rasterizer,
        p_multisample_state: &multisampling,
        p_color_blend_state: &color_blend_info,
        p_dynamic_state: ptr::null(),
        p_depth_stencil_state: ptr::null(),
        layout: pipeline_layout,
        render_pass: *render_pass,
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: -1,
        ..Default::default()
    };
    let graphics_pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
            .expect("Failed to create graphics pipeline!")
            .pop()
            .unwrap()
    };
    unsafe {
        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);
    };
    (graphics_pipeline, pipeline_layout)
}
fn read_shader(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).expect("Failed to open shader file");
    let mut contents: Vec<u8> = vec![];
    file.read_to_end(&mut contents)
        .expect("Failed to read shader file");
    contents
}
fn create_shader_module(device: &ash::Device, code: &[u8]) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo {
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
        ..Default::default()
    };
    unsafe {
        device
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create shader module")
    }
}
