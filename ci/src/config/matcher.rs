use crate::config::{GlobalConfig, WhitelistEntry};
use std::path::Path;
use glob::Pattern;

/// PathMatcher - 路径匹配器
pub struct PathMatcher {
    exclude_patterns: Vec<Pattern>,
    whitelist: Vec<Pattern>,
}

impl PathMatcher {
    pub fn new(config: &GlobalConfig, whitelist: &[WhitelistEntry]) -> Self {
        let exclude_patterns: Vec<Pattern> = config
            .exclude_patterns
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();

        let whitelist: Vec<Pattern> = whitelist
            .iter()
            .filter_map(|e| Pattern::new(&e.path).ok())
            .collect();

        Self {
            exclude_patterns,
            whitelist,
        }
    }

    pub fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().replace('\\', "/");

        for pattern in &self.exclude_patterns {
            if pattern.matches(&path_str) || pattern.matches(&format!("{}/**", path_str)) {
                return true;
            }
        }

        false
    }

    pub fn is_whitelisted(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().replace('\\', "/");

        for pattern in &self.whitelist {
            if pattern.matches(&path_str) || pattern.matches(&format!("{}/**", path_str)) {
                return true;
            }
        }

        false
    }
}
