use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use ci::config::Config;
use ci::checker::Runner;
use ci::fixer::Fixer;
use ci::report::print_report;
use ci::VerboseLevel;
use ci::verbose::debug_log;

/// 默认配置文件名
const DEFAULT_CONFIG_FILE: &str = "ci.toml";

/// 从当前目录向上查找配置文件
fn find_config_file(start_dir: &PathBuf) -> Option<PathBuf> {
    let mut current = start_dir.clone();

    // 向上查找直到根目录
    loop {
        let config_path = current.join(DEFAULT_CONFIG_FILE);
        if config_path.exists() {
            return Some(config_path);
        }

        // 到达根目录
        if !current.pop() {
            break;
        }
    }

    None
}

/// 获取配置文件路径（支持环境变量和分层查找）
fn get_config_path() -> PathBuf {
    // 1. 首先检查环境变量 CI_CONFIG
    if let Ok(config_env) = env::var("CI_CONFIG") {
        let path = PathBuf::from(&config_env);
        if path.exists() {
            return path;
        }
        // 如果环境变量指定了路径但文件不存在，使用该路径（让加载时报错）
        return path;
    }

    // 2. 从当前工作目录向上查找 ci.toml
    if let Ok(cwd) = env::current_dir() {
        if let Some(path) = find_config_file(&cwd) {
            return path;
        }
    }

    // 3. 默认路径
    PathBuf::from(DEFAULT_CONFIG_FILE)
}

/// 获取检查根目录（支持环境变量 CI_ROOT）
fn get_root_dir() -> PathBuf {
    // 1. 首先检查环境变量 CI_ROOT
    if let Ok(root_env) = env::var("CI_ROOT") {
        return PathBuf::from(root_env);
    }

    // 2. 默认当前目录
    PathBuf::from(".")
}

/// 日志级别参数（用于命令行解析）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[derive(clap::ValueEnum)]
enum VerboseArg {
    #[default]
    Quiet,
    Detail,
}

impl From<VerboseArg> for VerboseLevel {
    fn from(arg: VerboseArg) -> Self {
        match arg {
            VerboseArg::Quiet => VerboseLevel::Quiet,
            VerboseArg::Detail => VerboseLevel::Detail,
        }
    }
}

#[derive(Parser)]
#[command(name = "ci")]
#[command(about = "Rust 模块结构 CI 检查工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行所有检查
    Check {
        /// 配置文件路径（默认从当前目录向上查找 ci.toml）
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// 要检查的根目录（默认当前目录）
        #[arg(short, long)]
        root: Option<PathBuf>,

        /// 自动修复发现的问题
        #[arg(short, long)]
        fix: bool,

        /// 输出详细调试信息
        /// - quiet: 只输出错误（默认）
        /// - detail: 输出所有调试信息
        #[arg(short, long, value_enum, default_value = "quiet")]
        verbose: VerboseArg,
    },
    /// 验证配置文件
    Validate {
        /// 配置文件路径（默认从当前目录向上查找 ci.toml）
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { config, root, fix, verbose } => {
            // 根据 verbose 参数设置环境变量
            let verbose_level: VerboseLevel = verbose.into();
            env::set_var("CI_VERBOSE", match verbose_level {
                VerboseLevel::Detail => "detail",
                VerboseLevel::Quiet => "quiet",
            });
            
            let config_path = config.unwrap_or_else(get_config_path);
            let root_path = root.unwrap_or_else(get_root_dir);
            run_check(&config_path, &root_path, fix)?;
        }
        Commands::Validate { config } => {
            let config_path = config.unwrap_or_else(get_config_path);
            validate_config(&config_path)?;
        }
    }

    Ok(())
}

fn run_check(config_path: &PathBuf, root: &PathBuf, fix: bool) -> anyhow::Result<()> {
    // 加载配置
    let config = Config::load(config_path)?;

    // 确定实际的工作目录
    let work_root = if config.global.root.is_empty() {
        root.clone()
    } else {
        // 只有当用户没有通过 -r 指定路径时，才使用配置文件中的 root
        // 如果用户指定了 -r，则忽略配置中的 root
        if root.as_path() == PathBuf::from(".") {
            root.join(&config.global.root)
        } else {
            debug_log(format!("User specified root, ignoring config.root = {:?}", config.global.root));
            root.clone()
        }
    };

    // 创建运行器并运行检查
    let runner = Runner::new(config.clone());
    let report = runner.run(&work_root)?;

    // 打印报告
    print_report(&report, config.output.color);

    // 如果有错误，返回非零退出码
    if !report.is_success() {
        // 如果启用了 --fix，尝试自动修复
        if fix {
            println!("\n尝试自动修复...");
            let fixer = Fixer::new(config);
            match fixer.apply_fixes(&report) {
                Ok(result) => {
                    if result.has_fixes() {
                        println!("\n✓ 已修复 {} 个问题", result.fix_count());
                    }
                    if !result.errors.is_empty() {
                        println!("\n✗ {} 个问题修复失败", result.errors.len());
                        for (_, err) in &result.errors {
                            println!("  - {}", err);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n✗ 自动修复失败: {}", e);
                }
            }
        }

        std::process::exit(1);
    }

    Ok(())
}

fn validate_config(config_path: &PathBuf) -> anyhow::Result<()> {
    let config = Config::load(config_path)?;
    println!("✓ 配置文件有效: {}", config_path.display());
    println!("  全局配置:");
    println!("    root = {:?}", config.global.root);
    
    if config.global.entries.is_empty() {
        println!("    模式 = 目录扫描（传统模式）");
    } else {
        println!("    模式 = 模块树追踪");
        println!("    entries = [");
        for entry in &config.global.entries {
            println!("      {:?},", entry);
        }
        println!("    ]");
    }
    
    println!("  检查规则: enabled = {}", config.checks.enabled);
    Ok(())
}
