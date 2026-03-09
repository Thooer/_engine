# CI - Rust 模块结构检查工具

`ci/` 是一个独立的 Rust crate，用于强制执行 Rust 模块的结构规范。

## 设计目标

让一个模块的"对外形状"集中在 `mod.rs`，实现细节拆到：
- 同级 impl 文件
- `internal/` 目录（内部实现细节）
- `tests/` 目录（可选）

## 架构

```
ci/
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs               # 模块导出
│   ├── config/              # 配置加载与路径匹配
│   │   └── matcher.rs       # 路径匹配规则
│   ├── parser/              # 源码解析
│   │   ├── trait.rs         # Parser / ParsedSource trait
│   │   ├── ast.rs           # AST 节点定义
│   │   ├── syn_parser.rs    # syn 实现（主要）
│   │   └── tree_sitter_parser.rs # tree-sitter 实现（备用）
│   ├── checker/             # 检查器
│   │   ├── walker.rs        # 目录遍历
│   │   ├── runner.rs        # 规则执行（并行 + 缓存）
│   │   ├── mod.rs           # checker 模块导出
│   │   └── rules/           # 具体规则
│   │       ├── trait.rs     # Rule trait + RuleContext
│   │       ├── mod_rs.rs    # mod.rs 规则
│   │       ├── impl_file.rs # impl 文件规则
│   │       ├── internal.rs  # internal/ 规则
│   │       ├── tests_dir.rs # tests/ 规则
│   │       └── naming.rs    # 命名规则
│   ├── report/              # 报告输出
│   │   ├── types.rs         # 报告类型定义
│   │   ├── formatter.rs     # 格式化输出
│   │   └── mod.rs          # report 模块导出
│   └── fixer/              # 自动修复
│       └── mod.rs          # Fixer 实现
├── Cargo.toml
└── ci.toml                # 配置文件
```

### 核心组件

| 组件 | 说明 |
|------|------|
| **Parser** | 抽象解析接口，当前使用 `syn` 库 |
| **Rule** | 检查规则 trait，每个规则独立实现 |
| **Walker** | 遍历目录，收集模块结构 |
| **Runner** | 协调解析、缓存、并行执行 |
| **Fixer** | 自动修复功能 |

## 规则详解

### mod.rs 规则

| 规则 | 说明 | 配置键 |
|------|------|--------|
| 禁止 `impl` | 任何形式的 impl 块都不允许 | `forbid_impl` |
| 禁止 `include!` 宏 | include! 只能放在实现文件中 | 自动检测 |
| 禁止私有 struct | struct 必须 pub | `struct_must_be_public` |
| 禁止顶层 fn | 所有方法必须通过 trait 暴露 | `forbid_free_functions` |

### impl 文件规则

位置：同级的 `.rs` 文件

| 规则 | 说明 | 配置键 |
|------|------|--------|
| 必须有且只有一个 impl 块 | 同一 trait 的空 impl 除外 | `single_impl_only` |
| 文件命名必须对应 | `TraitName_TypeName.rs` 格式 | `naming_must_match_trait` |
| 禁止 `pub` 关键字 | impl 文件内部不需要 pub | `forbid_pub` |
| 禁止固有实现 | 必须通过 trait 暴露方法 | `forbid_inherent_impl` |

### internal/ 目录规则

位置：`模块名/internal/` 目录

| 规则 | 说明 | 配置键 |
|------|------|--------|
| 禁止 `mod.rs` | 不能有入口文件 | `forbid_mod_rs` |
| 必须用 `{}` 包裹 | 每个 .rs 文件必须是大括号包裹 | `require_brace_wrap` |
| 只允许函数体 | 禁止方法签名，只能有函数体 | `only_function_body` |

### tests/ 目录规则

位置：`模块名/tests/` 目录

| 规则 | 说明 | 配置键 |
|------|------|--------|
| 禁止 `mod.rs` | 不能有入口文件 | `forbid_mod_rs` |
| 父 mod.rs 声明 | 必须在父 mod.rs 中声明 `mod tests;` | `require_mod_declaration_in_parent` |
| 声明位置 | 模块声明必须紧跟 trait 后 | 自动检测 |
| 文件名匹配 | 测试文件名必须与 trait 同名 | `test_file_must_match_trait` |

## 命令

```bash
# 检查当前目录
cargo run --manifest-path ci/Cargo.toml -- check

# 指定配置文件
cargo run --manifest-path ci/Cargo.toml -- check -c ci/ci.toml

# 指定检查根目录
cargo run --manifest-path ci/Cargo.toml -- check -r crates/mycrate/src

# 自动修复问题
cargo run --manifest-path ci/Cargo.toml -- check --fix

# 验证配置文件
cargo run --manifest-path ci/Cargo.toml -- validate -c ci/ci.toml
```

## 配置 (ci.toml)

```toml
[global]
root = ""
exclude_patterns = ["**/target/**", "**/.git/**"]

[checks.mod_rs]
enabled = true
forbid_impl = true
struct_must_be_public = true
forbid_free_functions = true

[checks.impl_file]
enabled = true
single_impl_only = true
naming_must_match_trait = true
forbid_pub = true
forbid_inherent_impl = true

[checks.internal]
enabled = true
forbid_mod_rs = true
require_brace_wrap = true
only_function_body = true

[checks.tests]
enabled = true
forbid_mod_rs = true
require_mod_declaration_in_parent = true
test_file_must_match_trait = true

[checks.naming]
enabled = true
forbid_impl_suffix = true
forbid_tests_suffix = true

[[whitelist]]
path = "path/to/exclude"

[output]
format = "human"
color = true
verbose = false
```

## 自动修复 (--fix)

CI 支持以下自动修复：

| 修复类型 | 触发条件 |
|----------|----------|
| `RenameFile` | 文件命名与 trait 不匹配 → 重命名 |
| `SplitImplFile` | impl 文件包含多个非空 impl 块 → 拆分成多个文件 |
| `MoveImplToFile` | mod.rs 包含 impl 块 → 移动到单独文件 |
| `RemovePub` | impl 文件包含 pub 关键字 → 移除 pub |
| `AddModuleDeclaration` | tests/ 目录未在 mod.rs 声明 → 添加模块声明 |

## 模块导出 (lib.rs)

```rust
pub mod config;
pub mod parser;
pub mod checker;
pub mod report;
pub mod fixer;

pub use parser::{Parser, ParsedSource, SynParser, ImplBlock, TraitBlock, ModuleBlock};
pub use fixer::{Fixer, Fix, FixResult};
```
