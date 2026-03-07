use std::borrow::Cow;
use std::collections::HashSet;
use std::path::Path;
use std::fs;
use crate::graphics::{ShaderLoader, ShaderLoaderTrait};

/// 递归处理 include 指令
fn process_includes(root_dir: &Path, source: &str, included: &mut HashSet<String>) -> Result<String, String> {
    let mut final_source = String::new();

    for line in source.lines() {
        let trimmed = line.trim();
        // 兼容 #include "..." 和 #include <...>
        if trimmed.starts_with("#include") {
            // 提取引号或尖括号内的路径
            let start_idx = trimmed.find('"').or_else(|| trimmed.find('<'));
            let end_idx = trimmed.rfind('"').or_else(|| trimmed.rfind('>'));

            if let (Some(start), Some(end)) = (start_idx, end_idx) {
                if start >= end {
                    continue; // 格式错误
                }
                let include_path = &trimmed[start + 1..end];
                
                // 避免循环/重复引用
                if included.contains(include_path) {
                    continue;
                }
                included.insert(include_path.to_string());

                // 解析路径 (相对于 shaders 根目录)
                let file_abs_path = root_dir.join(include_path);

                let include_content = fs::read_to_string(&file_abs_path)
                    .map_err(|e| format!("Failed to read included shader {}: {}", file_abs_path.display(), e))?;

                // 递归处理被引用文件中的 include
                let processed_include = process_includes(root_dir, &include_content, included)?;
                
                final_source.push_str(&processed_include);
                final_source.push('\n');
            }
        } else {
            final_source.push_str(line);
            final_source.push('\n');
        }
    }

    Ok(final_source)
}

impl ShaderLoaderTrait for ShaderLoader {
    fn new(assets_dir: impl AsRef<Path>) -> Self {
        Self {
            root_dir: assets_dir.as_ref().join("shaders"),
        }
    }

    /// 加载并预处理着色器代码
    ///
    /// 支持语法：`#include "core/input.wgsl"` 或 `#include <core/input.wgsl>`
    fn load_shader_source(&self, shader_path: &str) -> Result<String, String> {
        // Normalize slashes for consistency
        let shader_path_normalized = shader_path.replace('\\', "/");
        
        let full_path = if shader_path_normalized.starts_with("assets/") {
            Path::new(&shader_path_normalized).to_path_buf()
        } else {
            self.root_dir.join(&shader_path_normalized)
        };
        
        let content = fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read shader file {}: {}", full_path.display(), e))?;

        process_includes(&self.root_dir, &content, &mut HashSet::new())
    }

    /// 创建 wgpu ShaderModule
    fn create_shader_module(
        &self, 
        device: &wgpu::Device, 
        shader_path: &str, 
        label: Option<&str>
    ) -> Result<wgpu::ShaderModule, String> {
        let source = self.load_shader_source(shader_path)?;
        
        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(source)),
        }))
    }
}