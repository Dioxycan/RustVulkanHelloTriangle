use ash::vk;
use ash::version::{DeviceV1_0};
use std::ptr;
use crate::queue::find_queue_families;
pub fn create_command_pool(instance:&ash::Instance,physical_device:&vk::PhysicalDevice,device:&ash::Device,surface:&vk::SurfaceKHR)->vk::CommandPool {
    let queue_family_indices = find_queue_families(instance, &physical_device, &surface);
    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::CommandPoolCreateFlags::empty(),
        queue_family_index: queue_family_indices.graphics_family.unwrap(),
    };

    unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create Command Pool!")
    }
}
// pub fn create_command_buffers(device:&ash::Device,command_pool:&vk::CommandPool,render_pass:&vk::RenderPass,framebuffer:&vk::Framebuffer,swap_chain_extent:&vk::Extent2D,pipeline:&vk::Pipeline)->vk::CommandBuffer {
//     let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
//         .command_buffer_count(1)
//         .command_pool(*command_pool)
//         .level(vk::CommandBufferLevel::PRIMARY);
//     let command_buffer = unsafe{
//         device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0]
//     };
//     record_command_buffer(device,&command_buffer,render_pass,framebuffer,swap_chain_extent,pipeline);
//     command_buffer
// }
// pub fn record_command_buffer(
//     device:&ash::Device,
//     command_buffer:&vk::CommandBuffer,
//     render_pass:&vk::RenderPass,
//     framebuffer:&vk::Framebuffer,
//     swap_chain_extent:&vk::Extent2D,
//     pipeline:&vk::Pipeline
// ){
//     let command_buffer_begin_info = vk::CommandBufferBeginInfo {
//         s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
//         p_next: ptr::null(),
//         p_inheritance_info: ptr::null(),
//         flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
//     };
//     unsafe{
//         device.begin_command_buffer(*command_buffer,&command_buffer_begin_info).unwrap();
//     }
//     let clear_values = [vk::ClearValue{
//         color:vk::ClearColorValue{
//             float32: [1.0,1.0,0.0,1.0]
//         }
//     }];
//     let render_pass_info= vk::RenderPassBeginInfo::builder()
//         .render_pass(*render_pass)
//         .framebuffer(*framebuffer)
//         .render_area(*vk::Rect2D::builder()
//             .extent(*swap_chain_extent)
//             .offset(vk::Offset2D::default()))
//         .clear_values(&clear_values);
//     unsafe{
//         device.cmd_begin_render_pass(*command_buffer,&render_pass_info,vk::SubpassContents::INLINE);
//         device.cmd_bind_pipeline(*command_buffer,vk::PipelineBindPoint::GRAPHICS,*pipeline);
//         device.cmd_draw(*command_buffer,3,1,0,0);
//         device.cmd_end_render_pass(*command_buffer);
//         device.end_command_buffer(*command_buffer).expect("Failed to record command buffer");
//     };
// }

pub fn create_command_buffers(
    device: &ash::Device,
    command_pool: &vk::CommandPool,
    graphics_pipeline: &vk::Pipeline,
    framebuffers: &Vec<vk::Framebuffer>,
    render_pass: &vk::RenderPass,
    surface_extent: &vk::Extent2D,
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: framebuffers.len() as u32,
        command_pool:*command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };

    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: *render_pass,
            framebuffer: framebuffers[i],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: *surface_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                *graphics_pipeline,
            );
            device.cmd_draw(command_buffer, 3, 1, 0, 0);

            device.cmd_end_render_pass(command_buffer);

            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    command_buffers
}