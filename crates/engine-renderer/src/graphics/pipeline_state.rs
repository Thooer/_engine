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
}

pub fn default_cull_mode() -> CullMode { CullMode::Back }
pub fn default_depth_write() -> bool { true }
pub fn default_depth_compare() -> DepthCompare { DepthCompare::Less }
pub fn default_blend_mode() -> BlendMode { BlendMode::Opaque }

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            cull_mode: default_cull_mode(),
            depth_write: default_depth_write(),
            depth_compare: default_depth_compare(),
            blend_mode: default_blend_mode(),
        }
    }
}
