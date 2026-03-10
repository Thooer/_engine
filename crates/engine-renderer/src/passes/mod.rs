use crate::renderer::{MainRenderer, SurfaceContextTrait, FrameStartError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAccess {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone)]
pub enum PassResource {
    Texture(String, ResourceAccess),
    Buffer(String, ResourceAccess),
}

pub trait RenderPass {
    fn render(
        &self,
        renderer: &MainRenderer,
        ctx: &mut dyn SurfaceContextTrait,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Result<(), FrameStartError>;

    // 注意，inputs 与 outputs 仅保存资源的名称，且该资源必须为GPU资源。
    fn inputs(&self) -> Vec<PassResource> {
        Vec::new()
    }

    fn outputs(&self) -> Vec<PassResource> {
        Vec::new()
    }
}

pub struct MeshForwardPass;

mod RenderPass_MeshForwardPass;

pub struct LinePass;

mod RenderPass_LinePass;
