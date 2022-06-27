use ash::vk;
use ash::version::DeviceV1_0;
use std::ptr;
pub fn create_framebuffer(device:&ash::Device,image_views:&Vec<vk::ImageView>,render_pass:&vk::RenderPass,swapchain_extent:&vk::Extent2D)->Vec<vk::Framebuffer> {
//    image_views.iter().map(|image_view| {
//         let attachments = [*image_view];
//         let framebuffer_info = 
//         vk::FramebufferCreateInfo::builder()
//             .render_pass(*render_pass)
//             .attachments(&attachments)
//             .width(swap_chain_extent.width)
//             .height(swap_chain_extent.height)
//             .layers(1)
//             .build();
//         unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() }
//     }).collect()
let mut framebuffers = vec![];

for &image_view in image_views.iter() {
    let attachments = [image_view];

    let framebuffer_create_info = vk::FramebufferCreateInfo {
        s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::FramebufferCreateFlags::empty(),
        render_pass:*render_pass,
        attachment_count: attachments.len() as u32,
        p_attachments: attachments.as_ptr(),
        width: swapchain_extent.width,
        height: swapchain_extent.height,
        layers: 1,
    };

    let framebuffer = unsafe {
        device
            .create_framebuffer(&framebuffer_create_info, None)
            .expect("Failed to create Framebuffer!")
    };

    framebuffers.push(framebuffer);
}

framebuffers
}