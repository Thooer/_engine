use crate::config::Config;
use crate::report::Report;
use crate::checker::walker::Module;
use std::path::PathBuf;

/// Rule trait - 检查规则抽象
pub trait Rule: Send + Sync {
    /// 规则名称
    fn name(&self) -> &str;

    /// 规则描述
    fn description(&self) -> &str;

    /// 是否启用
    fn enabled(&self) -> bool;

    /// 执行检查
    fn check(&self, context: &RuleContext) -> Report;
}

/// RuleContext - 规则执行上下文
pub struct RuleContext<'a> {
    /// 模块目录
    pub module_dir: &'a PathBuf,
    /// mod.rs 或 lib.rs 路径
    pub mod_rs_path: Option<&'a PathBuf>,
    /// 该模块目录下的 impl 文件列表
    pub impl_files: &'a [PathBuf],
    /// 解析后的 mod.rs 内容
    pub parsed_mod_rs: Option<&'a dyn crate::parser::ParsedSource>,
    /// 全局配置
    pub config: &'a Config,
}

impl<'a> RuleContext<'a> {
    pub fn new(
        module: &'a Module,
        parsed_mod_rs: Option<&'a dyn crate::parser::ParsedSource>,
        config: &'a Config,
    ) -> Self {
        Self {
            module_dir: &module.dir,
            mod_rs_path: module.mod_rs_path.as_ref().or(module.lib_rs_path.as_ref()),
            impl_files: &module.impl_files,
            parsed_mod_rs,
            config,
        }
    }
}
