# WASM 脚本编译指南

## 编译 WASM 脚本

```bash
cd demos/simple_demo/assets/scripts
cargo build --release --target wasm32-unknown-unknown
```

输出文件在 `target/wasm32-unknown-unknown/release/game_script.wasm`

复制到项目 assets 目录：

```bash
cp target/wasm32-unknown-unknown/release/game_script.wasm ../game.wasm
```

## 脚本 API

WASM 模块需要导出 `update` 函数：

```rust
/// update 函数签名
/// 参数: (dt: i32, frame_count: i32, radius: i32, height: i32, speed: i32)
/// 返回: (x: i32, y: i32, z: i32)
///
/// 参数和返回值都是 i32，因为 WASM 不直接支持 f32
/// 需要通过位表示转换：f32 <-> i32
#[no_mangle]
pub fn update(dt: i32, frame_count: i32, radius: i32, height: i32, speed: i32) -> (i32, i32, i32)
```
