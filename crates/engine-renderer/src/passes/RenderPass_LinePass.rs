use super::{RenderPass, LinePass, PassResource, ResourceAccess};
use crate::renderer::{MainRenderer, SurfaceContextTrait, FrameStartError};
use wgpu::util::DeviceExt;

impl RenderPass for LinePass {
    fn render(
        &self,
        renderer: &MainRenderer,
        ctx: &mut dyn SurfaceContextTrait,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Result<(), FrameStartError> {
        if renderer.lines.is_empty() {
            return Ok(());
        }

        let depth_texture = renderer.render_targets.get("Depth Texture").expect("Depth Texture not found");
        
        // Create vertex buffer for lines
        let vertex_buffer = ctx.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Debug Line Vertex Buffer"),
            contents: bytemuck::cast_slice(&renderer.lines),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Get shader via material
        let material_name = "line";
        if let Some(material) = renderer.material_cache.get(material_name) {
            if let Some(shader) = renderer.shader_cache.get(&material.shader_name) {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Debug Line Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                rpass.set_pipeline(&shader.pipeline);
                
                // Set Group 0 (Frame)
                rpass.set_bind_group(0, &renderer.frame_bind_group, &[]);
                // Set Group 1 (Pass)
                rpass.set_bind_group(1, &renderer.pass_bind_group, &[]);
                // Set Group 2 (Material)
                rpass.set_bind_group(2, &material.bind_group, &[]);

                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.draw(0..renderer.lines.len() as u32, 0..1);
            } else {
                eprintln!("Error: Shader '{}' not found for material '{}'", material.shader_name, material_name);
            }
        } else {
            eprintln!("Error: Material '{}' not found", material_name);
        }

        Ok(())
    }
}
