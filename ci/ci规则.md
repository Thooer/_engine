# CI 规范

CI 是一个 Rust 模块结构检查工具，让模块的"对外形状"集中在 mod.rs，实现细节拆分到同级 impl 文件、internal/ 目录和 tests/ 目录。

## 模块结构

```
my_module/
├── mod.rs           # 对外暴露的 trait、struct
├── Trait_Type.rs    # impl 实现文件
├── internal/        # 内部辅助函数（可选）
│   └── helper.rs    # 必须用 {} 包裹
└── tests/          # 测试文件（可选）
    └── Trait.rs     # 文件名必须与 trait 同名
```

## mod.rs 规则

| 规则 | 说明 |
|------|------|
| 禁止 impl | 任何形式的 impl 块都不允许 |
| 禁止 include! | include! 只能放在实现文件中 |
| 禁止私有 struct | struct 必须 pub |
| 禁止顶层 fn | 所有方法必须通过 trait 暴露 |
| 禁止 mod.rs中impl的任意声明 | 声明impl必须紧跟trait后，并且命名符合要求 |

## impl 文件规则

| 规则 | 说明 |
|------|------|
| 单一 impl 块 | 同一 trait 的空 impl 除外 |
| 文件命名 | TraitName_TypeName.rs 格式 |
| 禁止 pub | impl 文件内部不需要 pub |
| 禁止固有实现 | 必须通过 trait 暴露方法 |

## internal/ 规则

- 禁止 mod.rs
- 每个 .rs 文件必须用 `{}` 包裹
- 禁止方法签名，只能有 `fn name() { ... }` 形式

## tests/ 规则

- 禁止 mod.rs
- 必须在父 mod.rs 中声明：`mod tests;`
- 声明必须紧跟 trait 后
- 测试文件名必须与 trait 同名

## 命名禁止

- 禁止 `*impl.rs`、`*_impl.rs`
- 禁止 `*tests.rs`、`*test.rs`

---

## CI 命令

```bash
# 安装（从项目根目录）
cargo install --path ci

# 检查当前目录
ci check

# 检查指定目录
ci check -r ./src

# 自动修复（默认不要运行，应该人类手动运行）
ci check -r ./src -f

# 验证配置
ci validate
```

## 自动修复 (--fix)

| 修复类型 | 触发条件 |
|----------|----------|
| RenameFile | 文件命名与 trait 不匹配 |
| SplitImplFile | impl 文件包含多个 impl 块 |
| MoveImplToFile | mod.rs 包含 impl 块 |
| RemovePub | impl 文件包含 pub |
| AddModuleDeclaration | tests/ 未在 mod.rs 声明 |

---


编写 Rust 代码时遵循上述模块结构规范。使用 `ci check` 验证代码是否符合规范，使用 `ci check -f` 自动修复问题。