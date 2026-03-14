//! 渲染图抽象
//!
//! 提供灵活的渲染管线配置，支持动态添加/删除/重排序渲染节点

use std::any::TypeId;
use std::collections::HashMap;
use bevy_ecs::prelude::World;

/// 渲染端口 - 连接渲染节点
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Port {
    pub name: String,
    pub type_id: TypeId,
}

impl Port {
    pub fn new(name: &str, type_id: TypeId) -> Self {
        Self {
            name: name.to_string(),
            type_id,
        }
    }
}

/// 渲染节点 trait - 代表一个渲染操作
pub trait RenderNode: Send + Sync {
    /// 节点名称
    fn name(&self) -> &str;
    
    /// 执行渲染
    fn execute(&self, ctx: &mut RenderContext, world: &World);
    
    /// 输入端口
    fn inputs(&self) -> Vec<Port>;
    
    /// 输出端口
    fn outputs(&self) -> Vec<Port>;
}

/// 渲染上下文
pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub encoder: wgpu::CommandEncoder,
    pub target_view: wgpu::TextureView,
    /// 中间渲染目标
    pub textures: HashMap<String, wgpu::TextureView>,
}

impl RenderContext {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        encoder: wgpu::CommandEncoder,
        target_view: wgpu::TextureView,
    ) -> Self {
        Self {
            device,
            queue,
            encoder,
            target_view,
            textures: HashMap::new(),
        }
    }
}

/// 渲染图边 - 连接两个节点
#[derive(Debug, Clone)]
pub struct RenderEdge {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

/// 渲染图
///
/// 支持：
/// - 添加/删除渲染节点
/// - 连接节点（边）
/// - 拓扑排序执行
/// - 动态管线配置
pub struct RenderGraph {
    nodes: HashMap<String, Box<dyn RenderNode>>,
    edges: Vec<RenderEdge>,
    output_node: Option<String>,
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            output_node: None,
        }
    }

    /// 添加渲染节点
    pub fn add_node<N: RenderNode + 'static>(&mut self, name: &str, node: N) {
        self.nodes.insert(name.to_string(), Box::new(node));
    }

    /// 移除渲染节点
    pub fn remove_node(&mut self, name: &str) {
        self.nodes.remove(name);
        // 移除相关的边
        self.edges.retain(|e| e.from_node != name && e.to_node != name);
    }

    /// 连接节点
    pub fn add_edge(&mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) {
        self.edges.push(RenderEdge {
            from_node: from_node.to_string(),
            from_port: from_port.to_string(),
            to_node: to_node.to_string(),
            to_port: to_port.to_string(),
        });
    }

    /// 移除边
    pub fn remove_edge(&mut self, from_node: &str, to_node: &str) {
        self.edges.retain(|e| !(e.from_node == from_node && e.to_node == to_node));
    }

    /// 设置输出节点
    pub fn set_output(&mut self, node_name: &str) {
        self.output_node = Some(node_name.to_string());
    }

    /// 获取节点
    pub fn get_node(&self, name: &str) -> Option<&dyn RenderNode> {
        self.nodes.get(name).map(|b| b.as_ref() as &dyn RenderNode)
    }

    /// 获取所有节点名称
    pub fn node_names(&self) -> Vec<&str> {
        self.nodes.keys().map(|s| s.as_str()).collect()
    }

    /// 获取所有边
    pub fn edges(&self) -> &[RenderEdge] {
        &self.edges
    }

    /// 检查节点是否存在
    pub fn contains_node(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    /// 获取节点的输入
    pub fn get_node_inputs(&self, name: &str) -> Vec<Port> {
        self.nodes
            .get(name)
            .map(|n| n.inputs())
            .unwrap_or_default()
    }

    /// 获取节点的输出
    pub fn get_node_outputs(&self, name: &str) -> Vec<Port> {
        self.nodes
            .get(name)
            .map(|n| n.outputs())
            .unwrap_or_default()
    }

    /// 执行渲染图（简单的顺序执行）
    pub fn execute(&self, ctx: &mut RenderContext, world: &World) {
        // 如果设置了输出节点，只执行到该节点的路径
        // 否则执行所有节点
        if let Some(ref output) = self.output_node {
            self.execute_to(ctx, world, output);
        } else {
            // 按添加顺序执行
            for node in self.nodes.values() {
                node.execute(ctx, world);
            }
        }
    }

    /// 执行到指定节点
    fn execute_to(&self, ctx: &mut RenderContext, world: &World, target: &str) {
        // 简单实现：按拓扑排序执行
        // 实际需要更复杂的依赖解析
        let mut executed = std::collections::HashSet::new();
        self.execute_node(ctx, world, target, &mut executed);
    }

    fn execute_node(&self, ctx: &mut RenderContext, world: &World, name: &str, executed: &mut std::collections::HashSet<String>) {
        if executed.contains(name) {
            return;
        }

        // 执行依赖节点
        for edge in &self.edges {
            if edge.to_node == name {
                self.execute_node(ctx, world, &edge.from_node, executed);
            }
        }

        // 执行当前节点
        if let Some(node) = self.nodes.get(name) {
            node.execute(ctx, world);
            executed.insert(name.to_string());
        }
    }
}
