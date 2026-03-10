use super::{RenderPass, MeshForwardPass, PassResource, ResourceAccess};
use crate::renderer::{MainRenderer, SurfaceContextTrait, FrameStartError};
use glam::Mat4;

use crate::graphics::InstanceRaw;
use wgpu::util::DeviceExt;

impl RenderPass for MeshForwardPass {
    fn render(
        &self,
        renderer: &MainRenderer,
        ctx: &mut dyn SurfaceContextTrait,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Result<(), FrameStartError> {
        let depth_texture = renderer.render_targets.get("Depth Texture").expect("Depth Texture not found");

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesh Forward Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Set Group 0,1 (Frame, Pass)
            rpass.set_bind_group(0, &renderer.frame_bind_group, &[]);
            rpass.set_bind_group(1, &renderer.pass_bind_group, &[]);

            // Draw Objects
            for (model, transform) in &renderer.model_objects {
                let model_matrix = Mat4::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.translation,
                );

                let mut stack = Vec::new();
                for node in &model.root_nodes {
                    stack.push((node, model_matrix));
                }

                while let Some((node, parent_matrix)) = stack.pop() {
                    let node_local_matrix = Mat4::from_scale_rotation_translation(
                        node.transform.scale,
                        node.transform.rotation,
                        node.transform.translation,
                    );
                    let node_matrix = parent_matrix * node_local_matrix;

                    if let Some(mesh_idx) = node.mesh_index {
                        if let Some(mesh) = model.meshes.get(mesh_idx) {
                            
                            // Create Instance Buffer for this draw call
                            // Note: In production, use dynamic buffer or batched instancing
                            let instance_data = InstanceRaw {
                                model: node_matrix.to_cols_array_2d(),
                            };
                            
                            let instance_buffer = ctx.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Instance Buffer"),
                                contents: bytemuck::cast_slice(&[instance_data]),
                                usage: wgpu::BufferUsages::VERTEX,
                            });

                            // Bind Vertex Buffer
                            rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                            // Bind Instance Buffer (Slot 1)
                            rpass.set_vertex_buffer(1, instance_buffer.slice(..));
                            
                            rpass.set_index_buffer(
                                mesh.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );

                            for primitive in &mesh.primitives {
                                if let Some(mat_name) =
                                    model.material_names.get(primitive.material_index)
                                {
                                    if let Some(material) = renderer.material_cache.get(mat_name) {
                                        // Bind Pipeline
                                        if let Some(shader) =
                                            renderer.shader_cache.get(&material.shader_name)
                                        {
                                            rpass.set_pipeline(&shader.pipeline);

                                            // Bind Material Group (Group 2)
                                            rpass.set_bind_group(2, &material.bind_group, &[]);
                                            
                                            rpass.draw_indexed(
                                                primitive.index_start
                                                    ..primitive.index_start + primitive.index_count,
                                                0,
                                                0..1, // Instance count 1 for now
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    for child in &node.children {
                        stack.push((child, node_matrix));
                    }
                }
            }
        }
        Ok(())
    }

    fn inputs(&self) -> Vec<PassResource> {
        vec![
            PassResource::Buffer("Camera Uniform".to_string(), ResourceAccess::Read),
        ]
    }

    fn outputs(&self) -> Vec<PassResource> {
        vec![
            PassResource::Texture("Depth Texture".to_string(), ResourceAccess::Write),
            PassResource::Texture("Screen Texture".to_string(), ResourceAccess::Write),
        ]
    }
}
