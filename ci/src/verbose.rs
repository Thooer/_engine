use std::env;
use std::fmt;

/// 日志级别
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerboseLevel {
    Quiet,   // 极简模式：只输出错误
    Detail,  // 详细模式：输出所有调试信息
}

impl Default for VerboseLevel {
    fn default() -> Self {
        VerboseLevel::Quiet
    }
}

impl From<&str> for VerboseLevel {
    fn from(s: &str) -> Self {
        match s {
            "detail" | "debug" | "verbose" => VerboseLevel::Detail,
            _ => VerboseLevel::Quiet,
        }
    }
}

/// 获取当前的日志级别
pub fn get_verbose_level() -> VerboseLevel {
    match env::var("CI_VERBOSE") {
        Ok(val) => VerboseLevel::from(val.as_str()),
        Err(_) => VerboseLevel::Quiet,
    }
}

/// 调试日志输出（只在详细模式下输出）
#[inline]
pub fn debug_log<T: fmt::Display>(msg: T) {
    if get_verbose_level() == VerboseLevel::Detail {
        eprintln!("DEBUG: {}", msg);
    }
}
