# Engine Script 系统设计文档

## 一、设计目标

提供一套通用的 WASM 脚本接口，让游戏逻辑完全运行在 WASM 脚本中，引擎仅负责 ECS 调度和渲染。

## 二、核心架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                           引擎 (Rust)                               │
│                                                                     │
│   ECS World ◄────────────────┬──────────────────────┐             │
│        │                     │                       │             │
│        │                     │                       │             │
│        ▼                     │                       │             │
│  ┌─────────────┐             │                       │             │
│  │ 渲染系统    │             │                       │             │
│  │ 物理系统    │             │                       │             │
│  │ ...        │             │                       │             │
│  └─────────────┘             │                       │             │
│                              │                       │             │
│         ◄────────────────────┴───────────────────────┘             │
│                         ScriptHost                                  │
│              (持有 World 引用，传递给 WASM)                         │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      WASM 脚本 (游戏逻辑)                           │
│                                                                     │
│   脚本直接调用:                                                     │
│   - world.spawn((Transform, MeshRenderable, ...))                  │
│   - world.despawn(entity)                                          │
│   - world.query::<(&Transform, &mut Velocity)>()                   │
│   - world.get_mut::<Transform>(entity)                             │
│                                                                     │
│   所有游戏逻辑都在这里:                                             │
│   - 相机控制                                                       │
│   - 物体生成/销毁                                                  │
│   - 物理模拟                                                       │
│   - AI 行为                                                        │
└─────────────────────────────────────────────────────────────────────┘
```

## 三、模块职责划分

### 3.1 engine-script (纯接口层，无游戏逻辑)

| 文件 | 职责 |
|------|------|
| `lib.rs` | ScriptContext trait 定义（仅接口） |
| `wasm_host.rs` | WASM 运行时（仅加载/调用） |
| **不含任何游戏逻辑** | - |

### 3.2 engine-app (引擎入口，仅组装系统)

| 文件 | 职责 |
|------|------|
| `main.rs` | 组装 ECS 系统，注册脚本 Host |
| **不含游戏逻辑** | - |

### 3.3 demos/*/assets/scripts (所有游戏逻辑)

| 文件 | 职责 |
|------|------|
| `src/lib.rs` | 相机控制、物体管理、碰撞检测 |
| `game.wasm` | 编译后的游戏逻辑 |

## 四、核心接口设计

### 4.1 ScriptContext Trait

```rust
/// 脚本上下文 - 脚本访问引擎的唯一入口
/// 这个 trait 足够简单，让脚本可以做任何操作
pub trait ScriptContext: Send + Sync {
    /// 获取可变的 ECS World 引用
    /// 脚本通过这个直接创建/查询/修改实体
    fn world_mut(&mut self) -> &mut World;
    
    /// 只读帧信息
    fn delta_time(&self) -> f32;
    fn total_time(&self) -> f64;
    fn frame_count(&self) -> u32;
    
    /// 输入状态
    fn input_mask(&self) -> u8;
    
    /// 日志 (可选)
    fn log(&self, level: &str, message: &str);
}
```

### 4.2 WASM 导入函数

脚本通过 WASM 导入函数直接操作 ECS：

| 函数名 | 参数 | 返回 | 功能 |
|--------|------|------|------|
| `spawn_entity` | `type_id: i32` | `i32` | 创建实体，返回 entity bits |
| `despawn_entity` | `entity_bits: i32` | - | 删除实体 |
| `get_transform_x` | `entity_bits: i32` | `i32` | 获取 Transform.x |
| `get_transform_y` | `entity_bits: i32` | `i32` | 获取 Transform.y |
| `get_transform_z` | `entity_bits: i32` | `i32` | 获取 Transform.z |
| `set_transform` | `entity_bits, x, y, z, sx, sy, sz` | - | 设置 Transform |
| `query_entity_count` | - | `i32` | 获取实体数量 |
| `get_entity_id_by_index` | `index: i32` | `i32` | 按索引获取 entity bits |

## 五、World 共享机制

### 5.1 关键设计

- **不能创建空 World**：EcsScriptContext 必须持有引擎的真实 World 引用
- **直接修改**：脚本在 World 上的所有操作直接生效，无需同步
- **零拷贝**：通过引用传递，避免数据复制

### 5.2 调用流程

```
engine.on_update()
    │
    ├── 构建 FrameData
    │
    ├── 创建 EcsScriptContext::new(engine.world_mut(), frame_data)
    │       │
    │       └─ 传入引擎的真实 World 引用!
    │
    ├── host.update(&mut script_ctx)
    │       │
    │       └── WASM 脚本通过导入函数直接操作 engine.world_mut()
    │
    └── 引擎继续渲染 (ECS 已被脚本修改)
```

## 六、错误处理

| 场景 | 处理方式 |
|------|----------|
| WASM 加载失败 | 返回错误，日志记录，不影响引擎运行 |
| 脚本执行错误 | 捕获异常，记录日志，继续下一帧 |
| 无效 Entity 操作 | 返回 false/0，不崩溃 |

## 七、性能考虑

1. **WASM 调用开销**：通过内联导入函数最小化
2. **World 访问**：直接引用，无锁竞争（单线程假设）
3. **内存**：WASM 和引擎共享虚拟内存空间

## 八、扩展性

未来可添加：
- 热重载支持
- 调试/ profiler 接口
- 多脚本支持
- 事件系统
