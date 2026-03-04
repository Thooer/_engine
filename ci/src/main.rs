mod config;
mod parser;
mod checks;
mod report;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use config::Config;
use checks::Checker;

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
        /// 配置文件路径
        #[arg(short, long, default_value = "ci/ci.toml")]
        config: PathBuf,
        
        /// 要检查的根目录
        #[arg(short, long, default_value = ".")]
        root: PathBuf,
    },
    /// 验证配置文件
    Validate {
        /// 配置文件路径
        #[arg(short, long, default_value = "ci/ci.toml")]
        config: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { config, root } => {
            run_check(&config, &root)?;
        }
        Commands::Validate { config } => {
            validate_config(&config)?;
        }
    }

    Ok(())
}

fn run_check(config_path: &PathBuf, root: &PathBuf) -> anyhow::Result<()> {
    // 加载配置
    let config = Config::load(config_path)?;
    
    // 确定实际的工作目录
    let work_root = if config.global.root.is_empty() {
        root.clone()
    } else {
        root.join(&config.global.root)
    };

    // 创建检查器并运行检查
    let mut checker = Checker::new(config.clone());
    let report = checker.check(&work_root)?;

    // 打印报告
    report.print(config.output.color);

    // 如果有错误，返回非零退出码
    if !report.is_success() {
        std::process::exit(1);
    }

    Ok(())
}

fn validate_config(config_path: &PathBuf) -> anyhow::Result<()> {
    let config = Config::load(config_path)?;
    println!("✓ 配置文件有效");
    println!("  全局配置: root = {:?}", config.global.root);
    println!("  检查规则: enabled = {}", config.checks.enabled);
    Ok(())
}
