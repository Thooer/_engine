use std::path::Path;
use glam::{Vec3, Quat};
use wgpu::util::DeviceExt;
use crate::graphics::{GpuModel, GpuMesh, MeshPrimitive, ModelNode, ModelLoaderTrait, Vertex, ModelLoader};

impl ModelLoaderTrait for ModelLoader {
    fn load_gltf(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        path: impl AsRef<Path>,
    ) -> Result<GpuModel, String> {
        let path = path.as_ref();
        let (document, buffers, _images) = gltf::import(path)
            .map_err(|e| format!("Failed to load glTF: {}", e))?;

        // 1. 加载所有 Mesh
        let mut meshes = Vec::new();
        for mesh in document.meshes() {
            let gpu_mesh = load_mesh(device, &mesh, &buffers)?;
            meshes.push(gpu_mesh);
        }

        // 2. 加载所有材质名称
        let mut material_names = Vec::new();
        for material in document.materials() {
            let name = material.name().unwrap_or("default").to_string();
            material_names.push(name);
        }
        // 确保至少有一个默认材质名称
        if material_names.is_empty() {
            material_names.push("default".to_string());
        }

        // 3. 构建节点树
        let mut root_nodes = Vec::new();
        for scene in document.scenes() {
            for node in scene.nodes() {
                root_nodes.push(process_node(&node));
            }
        }

        Ok(GpuModel {
            meshes,
            material_names,
            root_nodes,
            name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
        })
    }
}

use engine_core::ecs::Transform;

fn process_node(node: &gltf::Node) -> ModelNode {
    let (t, r, s) = node.transform().decomposed();
    
    let transform = Transform {
        translation: Vec3::from(t),
        rotation: Quat::from_array(r),
        scale: Vec3::from(s),
    };

    let mesh_index = node.mesh().map(|m| m.index());
    
    let mut children = Vec::new();
    for child in node.children() {
        children.push(process_node(&child));
    }

    ModelNode {
        transform,
        mesh_index,
        children,
    }
}

fn load_mesh(
    device: &wgpu::Device,
    mesh: &gltf::Mesh,
    buffers: &[gltf::buffer::Data],
) -> Result<GpuMesh, String> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut primitives = Vec::new();

    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        // 读取顶点属性
        let positions: Vec<[f32; 3]> = reader.read_positions()
            .ok_or("Mesh primitive missing positions")?
            .collect();
        
        let normals: Vec<[f32; 3]> = reader.read_normals()
            .map(|iter| iter.collect())
            .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
            
        let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0)
            .map(|iter| iter.into_f32().collect())
            .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

        // 这里的 color 如果没有就默认白色
        let colors: Vec<[f32; 4]> = reader.read_colors(0)
            .map(|iter| iter.into_rgba_f32().collect())
            .unwrap_or_else(|| vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]);

        // 索引偏移
        let index_start = indices.len() as u32;
        let vertex_start = vertices.len() as u32; // 如果我们合并所有 primitive 的顶点，这需要处理

        // 读取索引
        if let Some(iter) = reader.read_indices() {
            let primitive_indices: Vec<u32> = iter.into_u32().collect();
            // 注意：因为我们把所有 primitive 的顶点合并到一个 buffer，索引需要加上 vertex_start 偏移
            // 但是 wgpu 绘制时可以用 base_vertex，所以这里可以只存原始索引，或者合并。
            // 简单起见，我们这里合并所有 primitive 到一个大的 vertex buffer
            // 那么索引必须加上 vertex_start
            indices.extend(primitive_indices.iter().map(|i| i + vertex_start));
        } else {
            // 没有索引，自动生成
            let count = positions.len() as u32;
            indices.extend((0..count).map(|i| i + vertex_start));
        }

        // 组装顶点
        for i in 0..positions.len() {
            vertices.push(Vertex {
                position: positions[i],
                normal: normals[i],
                uv: uvs[i],
                color: colors[i],
            });
        }

        primitives.push(MeshPrimitive {
            index_start,
            index_count: indices.len() as u32 - index_start,
            material_index: primitive.material().index().unwrap_or(0),
        });
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Vertex Buffer", mesh.name().unwrap_or("Mesh"))),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Index Buffer", mesh.name().unwrap_or("Mesh"))),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Ok(GpuMesh {
        vertex_buffer,
        index_buffer,
        vertex_count: vertices.len() as u32,
        index_count: indices.len() as u32,
        primitives,
    })
}
