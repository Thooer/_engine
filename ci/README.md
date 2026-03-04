## 这是什么
这是一个 **Rust 模块结构检查工具**（`ci/` 目录是一个独立 crate，二进制名 `ci`）。

目标是让一个模块的“对外形状”集中在 `mod.rs`，而实现细节被拆到同级的 impl 文件、以及可选的 `internal/` 与 `tests/` 目录里。

## 规则（速记版）
- **`mod.rs`**：禁止出现任何形式的 `impl`；需要对外 `pub` 的东西集中写在 `mod.rs`。
- **同级非 `mod.rs` 的 `.rs` 文件（impl 文件）**：
  - 必须 **有且只有一个** `impl` 块（特殊：同一 trait 的多个空 impl 可共存）。
  - 文件名必须和这个 `impl` 对应（工具会按 `TraitName_TypeName.rs` 推导期望名）。
  - 禁止出现 `pub`（对外 API 放回 `mod.rs`）。
- **`internal/` 目录**：禁止 `mod.rs`；每个 `.rs` 必须用 `{}` 包裹；只允许“函数体片段”（不能出现 `fn ...` 签名）。
- **`tests/` 目录**：禁止 `mod.rs`；必须在父目录 `mod.rs` 里声明 `#[cfg(test)] mod tests;` 且该声明要紧跟某个 trait 后；测试文件名必须与 trait 同名。
- **命名禁用**：禁止 `*impl.rs`、`*_impl.rs`、`*tests.rs`、`*test.rs` 这类后缀命名（tests 目录除外）。

## 怎么用（本地运行）
建议在 **仓库根目录** 运行：

```bash
cargo run --manifest-path ci/Cargo.toml -- check
```

## 检查逻辑（工具怎么扫）
- 从你指定的 `--root` 开始，遇到目录就看是否存在 `mod.rs`，然后做 **mod.rs / 同级 impl 文件** 检查。
- 对子目录递归；其中 `internal/`、`tests/` 走专门规则，其他目录继续当作普通模块目录处理。

常用参数：
- **指定配置文件**：

```bash
cargo run --manifest-path ci/Cargo.toml -- check -c ci/ci.toml
```

- **指定检查根目录**（只检查某个子目录/模块）：

```bash
cargo run --manifest-path ci/Cargo.toml -- check -r crates/toyengine-core/src
```

- **只验证配置文件是否能解析**：

```bash
cargo run --manifest-path ci/Cargo.toml -- validate -c ci/ci.toml
```

## 怎么看输出 / 怎么定位
- **通过**：输出 `✓ 所有检查通过`
- **失败**：输出 `✗ 发现 N 个错误`，并逐条打印：
  - `ERROR: <path>:<line>`（如果能定位到行，会带 `:<line>`）
  - 下一行是具体原因（例如 “mod.rs 禁止包含任何形式的 impl”、“impl 文件禁止使用 pub 关键字” 等）

处理方式通常是：按报错路径打开文件 → 跳到行号 → 按下面“常见修复”改结构/命名。

## 常见报错 & 修复对照
- **`mod.rs 禁止包含任何形式的 impl`**
  - 把 `impl` 块移到同级独立文件（见下条“impl 文件”命名规则）。
- **`impl 文件只能包含一个 impl 块 ... 期望拆分成 ...`**
  - 一个 `.rs` 里有多个 impl：拆分成多个文件，每个文件保留一个 impl。
- **`文件命名必须和 trait 对应：期望 X，实际 Y`**
  - 将文件改名为工具提示的期望名（通常是 `TraitName_TypeName.rs`）。
- **`impl 文件禁止使用 pub 关键字`**
  - 把 `pub` 改为私有或 `pub(crate)`；需要对外暴露的 API 在 `mod.rs` 提供 `pub fn/struct/...` 转发。
- **`internal/ 目录中的文件必须用大括号 {} 包裹` / `只能包含函数体`**
  - `internal/*.rs` 文件内容必须形如 `{ ... }`，且里面不要写 `fn xxx(...) {}` 这种签名，只保留函数体片段。
- **`tests/ 目录必须在 mod.rs 中声明模块` / `tests/ 模块声明必须跟在某个 trait 后面`**
  - 在父目录 `mod.rs` 的某个 trait 定义后面紧跟：`#[cfg(test)] mod tests;`

## 配置（`ci/ci.toml`）
- **`[global].exclude_patterns`**：排除目录（默认已经排除了 `target/`、`.git/`、`node_modules/`、`thirdparty/` 等）。
  - 注意：当前匹配实现偏“包含子串”，写 pattern 时尽量用稳定的路径片段。
- **`[[whitelist]]`**：白名单路径（当前实现：命中后直接跳过该路径下的所有检查）。