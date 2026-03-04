//! Simple 3D mesh render pass helper (v0)
//!
//! 把最基础的“单一 3D 渲染通路 + 深度”封装到库里，供示例与后续系统复用。
//! 设计目标：
//! - 集中处理 frame_start / encoder / render_pass / 提交队列 与 SurfaceError 分支；
//! - 调用方只关心：如何在给定的 render pass 中录制具体的绘制命令；
//! - 暂时只做最小封装，避免过早锁死更高层的 RenderGraph / Camera 设计。

use crate::renderer::{FrameStartError, SurfaceContext, SurfaceContextTrait};

/// 简单 3D 渲染通路配置。
///
/// 当前仅支持：
/// - 清屏颜色
/// - 深度缓冲清除值
#[derive(Clone, Copy, Debug)]
pub struct SimpleMesh3DPassConfig {
    /// 颜色缓冲清屏颜色
    pub clear_color: wgpu::Color,
    /// 深度缓冲清除值（典型为 1.0）
    pub depth_clear: f32,
}

/// 在单一 render pass 中执行一帧 3D 渲染。
///
/// - 内部会处理 `frame_start` / `frame_show` 以及常见的 SurfaceError 分支；
/// - OOM 仍然直接 panic；其它 SurfaceError 则静默返回，保持与示例中原始逻辑一致；
/// - 调用方通过闭包在 render pass 中录制具体绘制命令，可自由设置 pipeline / 绑定 / draw 调用等。
pub fn draw_simple_mesh3d_pass<F>(
    ctx: &mut SurfaceContext,
    depth_view: &wgpu::TextureView,
    cfg: SimpleMesh3DPassConfig,
    record_commands: F,
) where
    F: for<'a> FnOnce(&mut wgpu::RenderPass<'a>),
{
    let (frame, view) = match ctx.frame_start() {
        Ok(v) => v,
        Err(FrameStartError::NoSurfaceSize) => return,
        Err(FrameStartError::Surface(wgpu::SurfaceError::OutOfMemory)) => panic!("oom"),
        Err(FrameStartError::Surface(_)) => return,
    };

    let mut encoder = ctx
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("simple_mesh3d encoder"),
        });

    {
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("simple_mesh3d pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(cfg.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(cfg.depth_clear),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        record_commands(&mut rp);
    }

    ctx.queue().submit(Some(encoder.finish()));
    ctx.frame_show(frame);
}

