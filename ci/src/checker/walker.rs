use crate::config::Config;
use crate::parser::{Parser, SynParser};
use std::path::{Path, PathBuf};
use std::fs;

/// Module - 模块结构
pub struct Module {
    pub dir: PathBuf,
    pub mod_rs_path: Option<PathBuf>,
    pub lib_rs_path: Option<PathBuf>,
    pub impl_files: Vec<PathBuf>,
    pub internal_dir: Option<PathBuf>,
    pub tests_dir: Option<PathBuf>,
    pub submodules: Vec<Module>,
}

impl Module {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            mod_rs_path: None,
            lib_rs_path: None,
            impl_files: Vec::new(),
            internal_dir: None,
            tests_dir: None,
            submodules: Vec::new(),
        }
    }
}

/// Walker - 目录遍历器（支持两种模式：模块树模式和传统目录扫描模式）
pub struct Walker {
    config: Config,
    parser: SynParser,
}

impl Walker {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            parser: SynParser::new(),
        }
    }

    /// 遍历模块，返回模块列表
    /// 如果配置了 entries，使用模块树模式；否则使用传统目录扫描模式
    pub fn walk(&self, root: &Path) -> anyhow::Result<Vec<Module>> {
        if !self.config.global.entries.is_empty() {
            // 模块树模式：从 lib.rs 入口追踪模块树
            self.walk_entries(root)
        } else {
            // 传统模式：目录扫描
            let mut modules = Vec::new();
            self.walk_dir(root, &mut modules)?;
            Ok(modules)
        }
    }

    /// 模块树模式：从配置的入口文件追踪模块树
    fn walk_entries(&self, root: &Path) -> anyhow::Result<Vec<Module>> {
        let mut modules = Vec::new();

        for entry_path in &self.config.global.entries {
            let full_path = root.join(entry_path);
            if !full_path.exists() {
                continue;
            }

            // 获取入口文件所在目录
            let entry_dir = full_path.parent().unwrap_or(root);
            
            // 从入口文件开始追踪模块树
            self.walk_module_tree(&full_path, entry_dir, &mut modules)?;
        }

        Ok(modules)
    }

    /// 从入口文件追踪模块树
    fn walk_module_tree(
        &self,
        entry_file: &Path,
        base_dir: &Path,
        modules: &mut Vec<Module>,
    ) -> anyhow::Result<()> {
        let mut module = Module::new(base_dir.to_path_buf());

        // 设置入口文件路径
        let file_name = entry_file.file_name().unwrap().to_string_lossy();
        if file_name == "lib.rs" {
            module.lib_rs_path = Some(entry_file.to_path_buf());
        } else if file_name == "mod.rs" {
            module.mod_rs_path = Some(entry_file.to_path_buf());
        }

        // 解析入口文件，获取 mod 声明
        let parsed = match self.parser.parse(entry_file) {
            Ok(p) => p,
            Err(_) => {
                modules.push(module);
                return Ok(());
            }
        };

        // 获取所有 mod 声明
        let mod_names = parsed.get_module_names();

        // 收集同目录下的 .rs 文件（impl 文件）
        if let Ok(entries) = fs::read_dir(base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                    continue;
                }

                let file_name = path.file_name().unwrap().to_string_lossy();
                let file_stem = path.file_stem().unwrap().to_string_lossy();

                // 跳过特殊文件
                if file_name == "mod.rs" || file_name == "lib.rs" || file_stem == "main" {
                    continue;
                }

                // 如果这个文件名对应一个目录模块（xxx.rs 对应 xxx/ 目录），跳过
                // 因为它会作为子模块处理
                let potential_dir = base_dir.join(&*file_stem);
                if potential_dir.exists() && potential_dir.is_dir() {
                    continue;
                }

                // 这是一个 impl 文件
                module.impl_files.push(path);
            }
        }

        // 检查 internal/ 和 tests/ 目录
        let internal_dir = base_dir.join("internal");
        if internal_dir.exists() && internal_dir.is_dir() {
            module.internal_dir = Some(internal_dir);
        }

        let tests_dir = base_dir.join("tests");
        if tests_dir.exists() && tests_dir.is_dir() {
            module.tests_dir = Some(tests_dir);
        }

        // 递归处理子模块
        for mod_name in &mod_names {
            // 跳过 internal 和 tests 模块（特殊处理）
            if mod_name == "internal" || mod_name == "tests" {
                continue;
            }

            // 查找子模块路径
            // 优先级：xxx/mod.rs > xxx.rs
            let submod_dir = base_dir.join(mod_name);
            let submod_file = base_dir.join(format!("{}.rs", mod_name));

            if submod_dir.exists() && submod_dir.is_dir() {
                let submod_entry = submod_dir.join("mod.rs");
                if submod_entry.exists() {
                    self.walk_module_tree(&submod_entry, &submod_dir, &mut module.submodules)?;
                }
            } else if submod_file.exists() {
                // xxx.rs 形式的模块，不递归
                // 这些文件已经在 impl_files 中被跳过了
            }
        }

        modules.push(module);
        Ok(())
    }

    /// 传统模式：目录扫描
    fn walk_dir(&self, dir: &Path, modules: &mut Vec<Module>) -> anyhow::Result<()> {
        let mut module = Module::new(dir.to_path_buf());

        let mod_rs_path = dir.join("mod.rs");
        let lib_rs_path = dir.join("lib.rs");

        if mod_rs_path.exists() {
            module.mod_rs_path = Some(mod_rs_path);
        }
        if lib_rs_path.exists() {
            module.lib_rs_path = Some(lib_rs_path);
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                let dir_name = path.file_name().unwrap().to_string_lossy().to_string();

                if dir_name == "internal" {
                    module.internal_dir = Some(path);
                } else if dir_name == "tests" {
                    module.tests_dir = Some(path);
                } else {
                    self.walk_dir(&path, &mut module.submodules)?;
                }
            }
        }

        // 收集 impl 文件
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                    continue;
                }

                let file_name = path.file_name().unwrap().to_string_lossy();
                let file_stem = path.file_stem().unwrap().to_string_lossy();

                if file_name == "mod.rs" || file_name == "lib.rs" || file_stem == "main" {
                    continue;
                }

                module.impl_files.push(path);
            }
        }

        if module.mod_rs_path.is_some() || module.lib_rs_path.is_some() {
            modules.push(module);
        }

        Ok(())
    }
}
