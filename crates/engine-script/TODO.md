# Engine Script 重构计划

## 阶段一：修复核心问题 (Priority: High)

### 1.1 修复 World 共享问题

| 任务 | 文件 | 状态 |
|------|------|------|
| 修改 `EcsScriptContext::new()` 接受 `&'a mut World` 而非空 World | `lib.rs` | ✅ DONE |
| 修改 `main.rs` 传入 `engine.world_mut()` | `main.rs` | ✅ DONE |
| 验证 spawn/despawn 功能生效 | - | ✅ DONE |

**核心问题已解决**: 脚本现在可以访问引擎的真实 World!

**关键修改**:
```rust
// lib.rs - 修改构造函数
impl EcsScriptContext {
    pub fn new(world: &'a mut World, frame_data: FrameData) -> Self {
        Self { world, frame_data }
    }
}
```

```rust
// main.rs - 修改调用
let mut script_ctx = EcsScriptContext::new(
    engine.world_mut(),  // 传入真实 World!
    FrameData::new(dt, total_time, frame_count).with_input_mask(input_mask)
);
```

---

## 阶段二：添加 WASM 导入函数 (Priority: High)

### 已实现方案：命令队列模式

由于 wasmer v4 要求导入函数闭包满足 Send+Sync，而 ScriptContext 不满足此条件，采用**命令队列模式**：
- WASM 导入函数将命令写入队列
- 引擎主循环在脚本执行后处理命令

### 已实现的导入函数

| 函数签名 | 功能 | 命令类型 |
|---------|------|---------|
| `spawn_entity(type_id: i32) -> i32` | 创建实体，返回 entity_bits | Spawn |
| `despawn_entity(entity_bits: i32) -> i32` | 销毁实体 | Despawn |
| `entity_exists(entity_bits: i32) -> i32` | 检查实体是否存在 | - |
| `get_transform_x(entity_bits: i32) -> i32` | 获取 Transform X | - |
| `get_transform_y(entity_bits: i32) -> i32` | 获取 Transform Y | - |
| `get_transform_z(entity_bits: i32) -> i32` | 获取 Transform Z | - |
| `set_transform(entity_bits, x, y, z, sx, sy, sz)` | 设置 Transform | SetTransform |
| `query_entity_count() -> i32` | 查询实体数量 | - |
| `log(level: i32, message_ptr: i32, message_len: i32)` | 输出日志 | Log |

**状态**: ✅ DONE

### 已实现命令执行方法

在 `wasm_host.rs` 中添加了 `execute_commands` 方法：

```rust
/// 执行命令队列 - 在脚本更新后调用
/// 将命令队列中的命令应用到 ECS World
pub fn execute_commands(&mut self, world: &mut World)
```

支持的命令类型：
- `Spawn` - 创建实体（待完善）
- `Despawn` - 销毁实体
- `SetTransform` - 设置 Transform
- `Log` - 输出日志

**状态**: ✅ DONE

---

## 阶段三：迁移游戏逻辑 (Priority: High)

### 3.1 从 main.rs 移除游戏逻辑

| 任务 | 文件 | 状态 |
|------|------|------|
| 移除手动 spawn/despawn 处理 | `main.rs` | ✅ DONE |
| 移除手动相机位置同步 | `main.rs` | ✅ DONE |
| 移除手动 Satellite 位置同步 | `main.rs` | ✅ DONE |

**注意**: 引擎不再处理任何游戏逻辑（相机、卫星、物体），所有游戏逻辑必须由 WASM 脚本通过 ECS 直接操作组件。

### 3.2 在 WASM 脚本中实现游戏逻辑

| 任务 | 文件 | 状态 |
|------|------|------|
| 实现相机控制 (4种轨道模式) | `demos/.../src/lib.rs` | PENDING |
| 实现动态物体生成/删除 | `demos/.../src/lib.rs` | PENDING |
| 实现卫星运动逻辑 | `demos/.../src/lib.rs` | PENDING |
| 重新编译 WASM | - | PENDING |

---

## 阶段四：测试与优化 (Priority: Medium)

### 4.1 功能测试

| 任务 | 状态 |
|------|------|
| 相机控制正常切换 | PENDING |
| 物体从高空下落 | PENDING |
| 边界检测和自动删除 | PENDING |
| 定时生成新物体 | PENDING |

### 4.2 性能优化

| 任务 | 状态 |
|------|------|
| 减少不必要的 WASM 调用 | PENDING |
| 优化 World 访问模式 | PENDING |

---

## 阶段五：清理与文档 (Priority: Low)

| 任务 | 状态 |
|------|------|
| 清理未使用的代码 | PENDING |
| 更新文档 | PENDING |
| 添加单元测试 | PENDING |

---

## 依赖关系

```
阶段一 ──► 阶段二 ──► 阶段三 ──► 阶段四 ──► 阶段五
   │          │          │          │          │
   ▼          ▼          ▼          ▼          ▼
  核心      导入函数   游戏逻辑    测试       清理
  修复       添加      迁移       验证
```

## 预期成果

- **main.rs** 从 550+ 行减少到约 400 行
- **engine-script** 保持纯接口层，无游戏逻辑
- **WASM 脚本** 完全掌控游戏逻辑
- spawn/despawn 功能正常工作
