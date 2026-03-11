#### ⚠️ 需要改进的瑕疵（The Bad：过度设计了 System）
你目前为每一个 System 都定义了一个 `struct` 和一个 `trait`（例如 `StepPhysicsSystem` 和 `StepPhysicsSystemTrait`）。
**在 `bevy_ecs` 中，这是严重的反模式（Anti-pattern）。**

`bevy_ecs` 的核心魔法在于它的 **依赖注入（Dependency Injection）**。系统不应该是实现了某个 Trait 的结构体，而应该**仅仅是一个普通的 Rust 函数**。

**你应该这样改（以同步系统为例）：**

```rust
// 删掉 struct SyncPhysicsToTransformSystem 和 trait ...
// 直接写一个普通的函数！
pub fn sync_physics_to_transform_system(
    physics_context: Res<PhysicsContext>, // Bevy 会自动注入 Resource
    mut transform_query: Query<(&PhysicsHandle, &mut Transform)>, // Bevy 会自动注入 Query
) {
    for (handle, mut transform) in transform_query.iter_mut() {
        if let Some(rb) = physics_context.rigid_body_set.get(handle.rigid_body_handle) {
            let pos = rb.translation();
            // 同步逻辑...
        }
    }
}
```
**为什么必须改成普通函数？**
因为 `bevy_ecs` 的调度器（Scheduler）是通过分析函数的参数（`Res`, `Query`, `Commands`）来自动推断系统之间的读写冲突，从而实现**自动多线程并发执行**的。如果你用 Trait 包装起来，你就破坏了 Bevy 的并发调度机制，写起来也全是冗余的样板代码。

---

### 二、 如何推进性能测试（Profiling & Benchmarking）

既然你的胶水层已经写好了，接下来就是验证它的含金量。测试物理引擎胶水层，核心是测试**“同步开销”**和**“缓存命中率”**。

你需要构建以下 **3 个极限测试场景（Stress Tests）**：

#### 场景 1：物理沉睡测试（The Graveyard Test）
*   **设置**：生成 **100,000** 个动态刚体（方块），让它们全部掉在地上，静止不动（进入 Sleeping 状态）。
*   **测试目的**：测试你的胶水层在物体不动时的开销。
*   **预期结果**：`SyncTransformToPhysics` 耗时应为 0（因为没有 `Changed<Transform>`）；`SyncPhysicsToTransform` 耗时应极低（你需要优化它，跳过 sleeping 的刚体）；`StepPhysics` 耗时极低。

#### 场景 2：物理混沌测试（The Chaos Test）
*   **设置**：生成 **10,000** 个动态刚体，在一个封闭的漏斗里疯狂碰撞、挤压，永远无法休眠。
*   **测试目的**：测试 Rapier 的极限求解性能，以及你的 `SyncPhysicsToTransform`（物理到渲染的同步）在满载情况下的内存拷贝开销。
*   **预期结果**：`StepPhysics` 将占据 90% 的时间。你需要观察 `SyncPhysicsToTransform` 是否成为了瓶颈（比如 `glam` 和 `nalgebra` 的转换是否够快）。

#### 场景 3：上帝之手测试（The Kinematic Test）
*   **设置**：生成 **10,000** 个运动学刚体（KinematicPositionBased），在游戏逻辑中每帧通过修改 ECS 的 `Transform` 让它们绕圈运动。
*   **测试目的**：测试 `SyncTransformToPhysics`（渲染到物理的同步）的开销。
*   **预期结果**：测试你的胶水层把 ECS 数据塞进 Rapier 的效率。

#### 🛠️ 性能测试工具链推荐

不要用肉眼看帧率，那是不专业的。你需要引入以下工具：

1.  **基础测量：`std::time::Instant`**
    在你的 Game Loop 中，简单粗暴地测量每个 System 的耗时：
    ```rust
    let start = std::time::Instant::now();
    // 运行 sync_physics_to_transform_system
    let duration = start.elapsed();
    println!("Sync to Transform took: {:?}", duration);
    ```

2.  **专业火焰图：`tracing` + `tracy` (强烈推荐)**
    这是现代游戏引擎开发的标配。
    *   引入 `tracing` 和 `tracing-tracy` crate。
    *   在你的系统函数上加宏：
        ```rust
        #[tracing::instrument(skip_all)]
        pub fn step_physics_system(...) { ... }
        ```
    *   下载 [Tracy Profiler](https://github.com/wolfpld/tracy)，运行你的引擎。你将能直观地看到每一帧里，物理步进占了多少毫秒，数据同步占了多少毫秒，是否有不合理的内存分配（Spikes）。

### 总结你的下一步行动：

1.  **重构 System**：把所有的 `struct` 和 `trait` 删掉，全部改成符合 `bevy_ecs` 规范的普通函数。
2.  **完善同步逻辑**：在 `SyncPhysicsToTransformSystem` 中，尝试加入判断：如果 Rapier 中的刚体是 `is_sleeping()`，则跳过更新 ECS 的 `Transform`（这能极大提升场景 1 的性能）。
3.  **写一个 Demo App**：用 `bevy_ecs` 搭建一个最小的 Game Loop，把这几个系统加进 `Schedule` 里。