use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CullMode {
    None,
    Front,
    Back,
}

impl From<CullMode> for Option<wgpu::Face> {
    fn from(val: CullMode) -> Self {
        match val {
            CullMode::None => None,
            CullMode::Front => Some(wgpu::Face::Front),
            CullMode::Back => Some(wgpu::Face::Back),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepthCompare {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl From<DepthCompare> for wgpu::CompareFunction {
    fn from(val: DepthCompare) -> Self {
        match val {
            DepthCompare::Never => wgpu::CompareFunction::Never,
            DepthCompare::Less => wgpu::CompareFunction::Less,
            DepthCompare::Equal => wgpu::CompareFunction::Equal,
            DepthCompare::LessEqual => wgpu::CompareFunction::LessEqual,
            DepthCompare::Greater => wgpu::CompareFunction::Greater,
            DepthCompare::NotEqual => wgpu::CompareFunction::NotEqual,
            DepthCompare::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            DepthCompare::Always => wgpu::CompareFunction::Always,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    Opaque,
    AlphaBlend,
    Add,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

impl From<PrimitiveTopology> for wgpu::PrimitiveTopology {
    fn from(val: PrimitiveTopology) -> Self {
        match val {
            PrimitiveTopology::PointList => wgpu::PrimitiveTopology::PointList,
            PrimitiveTopology::LineList => wgpu::PrimitiveTopology::LineList,
            PrimitiveTopology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            PrimitiveTopology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            PrimitiveTopology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct PipelineState {
    #[serde(default = "default_cull_mode")]
    pub cull_mode: CullMode,
    #[serde(default = "default_depth_write")]
    pub depth_write: bool,
    #[serde(default = "default_depth_compare")]
    pub depth_compare: DepthCompare,
    #[serde(default = "default_blend_mode")]
    pub blend_mode: BlendMode,
    #[serde(default = "default_primitive_topology")]
    pub topology: PrimitiveTopology,
}

pub fn default_cull_mode() -> CullMode { CullMode::Back }
pub fn default_depth_write() -> bool { true }
pub fn default_depth_compare() -> DepthCompare { DepthCompare::Less }
pub fn default_blend_mode() -> BlendMode { BlendMode::Opaque }
pub fn default_primitive_topology() -> PrimitiveTopology { PrimitiveTopology::TriangleList }

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            cull_mode: default_cull_mode(),
            depth_write: default_depth_write(),
            depth_compare: default_depth_compare(),
            blend_mode: default_blend_mode(),
            topology: default_primitive_topology(),
        }
    }
}
