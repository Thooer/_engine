use crate::checker::rules::{ImplFileRule, InternalRule, ModRsRule, NamingRule, Rule, RuleContext, TestsRule};
use crate::checker::{Module, Walker};
use crate::config::{Config, PathMatcher};
use crate::parser::{Parser, ParsedSource, SynParser};
use crate::report::Report;
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use rayon::prelude::*;

/// Runner - 规则执行器
pub struct Runner {
    config: Config,
    walker: Walker,
    path_matcher: PathMatcher,
    parser: SynParser,
    rules: Vec<Arc<dyn Rule>>,
    /// 解析结果缓存（线程安全）
    parse_cache: Arc<RwLock<HashMap<std::path::PathBuf, Arc<dyn ParsedSource>>>>,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(ModRsRule::new()),
            Arc::new(ImplFileRule::new()),
            Arc::new(InternalRule::new()),
            Arc::new(TestsRule::new()),
            Arc::new(NamingRule::new()),
        ];

        Self {
            walker: Walker::new(config.clone()),
            path_matcher: PathMatcher::new(&config.global, &config.whitelist),
            parser: SynParser::new(),
            config,
            rules,
            parse_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取或解析文件（带缓存）
    fn get_parsed(&self, path: &std::path::Path) -> Option<Arc<dyn ParsedSource>> {
        let path_buf = path.to_path_buf();

        // 先检查缓存
        {
            let cache = self.parse_cache.read();
            if let Some(cached) = cache.get(&path_buf) {
                return Some(Arc::clone(cached));
            }
        }

        // 解析并缓存
        if let Ok(parsed) = self.parser.parse(path) {
            let arc_parsed: Arc<dyn ParsedSource> = Arc::from(parsed);
            self.parse_cache.write().insert(path_buf, Arc::clone(&arc_parsed));
            Some(arc_parsed)
        } else {
            None
        }
    }

    pub fn run(&self, root: &Path) -> anyhow::Result<Report> {
        // 检查路径是否被排除或白名单
        if self.path_matcher.is_excluded(root) || self.path_matcher.is_whitelisted(root) {
            return Ok(Report::new());
        }

        // 遍历目录
        let modules = self.walker.walk(root)?;

        // 递归执行所有模块的规则检查
        let final_report = self.run_modules(&modules);

        Ok(final_report)
    }

    /// 递归执行模块检查
    fn run_modules(&self, modules: &[Module]) -> Report {
        // 并行执行所有模块的规则检查
        let results: Vec<Report> = modules
            .par_iter()
            .filter_map(|module| {
                // 检查模块目录是否被排除或白名单
                if self.path_matcher.is_excluded(&module.dir) || self.path_matcher.is_whitelisted(&module.dir) {
                    return None;
                }

                // 解析 mod.rs（带缓存）
                let mod_rs_path = module.mod_rs_path.as_ref().or(module.lib_rs_path.as_ref());
                let parsed_mod_rs = match mod_rs_path {
                    Some(path) => self.get_parsed(path),
                    None => None,
                };

                // 创建规则上下文
                let context = RuleContext::new(module, parsed_mod_rs.as_deref(), &self.config);

                // 收集所有规则的报告
                let mut module_report = Report::new();
                for rule in &self.rules {
                    if !rule.enabled() {
                        continue;
                    }
                    let rule_report = rule.check(&context);
                    module_report.merge(rule_report);
                }

                Some(module_report)
            })
            .collect();

        // 合并所有报告
        let mut final_report = Report::new();
        for report in results {
            final_report.merge(report);
        }

        // 递归处理子模块
        for module in modules {
            let submodule_report = self.run_modules(&module.submodules);
            final_report.merge(submodule_report);
        }

        final_report
    }
}
