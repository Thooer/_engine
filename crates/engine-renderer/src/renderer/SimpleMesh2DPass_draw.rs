//! Simple 2D mesh render pass helper (v0)
//!
//! 把最基础的“单一 2D 网格 + 清屏”渲染通路封装到库里，供示例与后续系统复用。
//! 设计目标：
//! - 示例代码只需要关心资源与动画逻辑；
//! - 渲染细节（frame_start / encoder / render_pass / 提交队列）集中在这里；
//! - 暂时只做最小封装，避免过早锁死更高层的 RenderGraph 设计。

use crate::renderer::{
    FrameStartError, SimpleMeshPipeline2D, SimpleMeshPipeline2DPipeline, SurfaceContext,
    SurfaceContextTrait,
};

/// Phase4 风格的简单 2D 网格渲染配置。
///
/// 目前只有清屏颜色，未来可以在不破坏示例的前提下，逐步扩展更多选项：
/// - 深度/模板配置
/// - 混合状态
/// - 线框/拓扑等
#[derive(Clone, Copy, Debug)]
pub struct SimpleMesh2DPassConfig {
    /// 清屏颜色
    pub clear_color: wgpu::Color,
}

/// 使用 [`SimpleMeshPipeline2DPipeline`] 在单一 render pass 中绘制一个网格。
///
/// - 内部会处理 `frame_start` / `frame_show` 以及常见的 SurfaceError 分支；
/// - OOM 仍然直接 panic；其它 SurfaceError 则静默返回，保持与示例中原始逻辑一致；
/// - 当前只支持“清屏 + 绘制一次网格”的最小场景。
pub fn draw_simple_mesh2d_pass(
    ctx: &mut SurfaceContext,
    pipe: &SimpleMeshPipeline2DPipeline,
    vertex: &wgpu::Buffer,
    index: &wgpu::Buffer,
    index_count: u32,
    cfg: SimpleMesh2DPassConfig,
) {
    let (frame, view) = match ctx.frame_start() {
        Ok(v) => v,
        Err(FrameStartError::NoSurfaceSize) => return,
        Err(FrameStartError::Surface(wgpu::SurfaceError::OutOfMemory)) => panic!("oom"),
        Err(FrameStartError::Surface(_)) => return,
    };

    let mut encoder = ctx
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("simple_mesh2d encoder"),
        });

    {
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("simple_mesh2d pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(cfg.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pipe.draw(&mut rp, vertex, index, index_count);
    }

    ctx.queue().submit(Some(encoder.finish()));
    ctx.frame_show(frame);
}

