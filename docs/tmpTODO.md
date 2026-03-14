## 当前状态总结

### 1. 架构概览
ToyEngine 是一个使用 Rust + wgpu 构建的轻量级游戏引擎，包含以下核心 crate：
- **engine-app**: 应用层，处理 winit 事件循环和运行时执行流
- **engine-core**: 核心系统，包含 ECS 组件、输入系统等
- **engine-renderer**: 渲染系统，基于 wgpu
- **engine-physics**: 物理系统，基于 Rapier
- **engine-events**: 事件系统

### 2. 已解决的问题（根据 tmpTODO.md）
- ✅ 统一为 `engine-app::App` 接口
- ✅ 引入 `SystemSchedule`，支持 setup/update 两阶段系统
- ✅ `RenderPlugin` 自动注册 `spawn_grid_system` 和 `orbit_camera_system`

### 3. 当前的设计不足

#### 3.1 ECS 系统调度器过于简化
当前 `SystemSchedule` 只是简单的 Vec<Box<dyn Fn>>，缺少：
- 系统依赖排序
- 并行执行支持
- 条件系统（仅在特定条件下运行）
- 系统标签和调度阶段细分

#### 3.2 物理与渲染仍存在脱节
- `physics_world::update` 手动调用 `init_bodies`/`step`/`sync_transforms`，而不是使用 ECS 系统


- 物理 ECS 系统（如 `apply_external_forces_system`）没有被调度

#### 3.3 资源管理缺失
- 缺少现代引擎的异步资源加载
- 缺少资源依赖追踪和热重载
- 资源路径硬编码问题

#### 3.4 输入系统未集成到 ECS
- `InputState` 是独立的状态对象，没有作为 Resource 插入 ECS
- 没有基于 ECS 的输入事件系统

#### 3.5 渲染管

线硬编码
- `MainRenderer::collect_from_world` 直接查询 ECS 组件，缺乏可配置性
- 渲染 passes 顺序固定，无法动态增删
- 缺少灵活的渲染图抽象

#### 3.6 缺少场景抽象
- 没有场景文件支持
- 没有场景切换机制

#### 3.7 插件系统不完善
- `PhysicsPlugin` 和 `RenderPlugin` 虽然已实现，但其他功能缺失
- 缺乏统一的插件接口
- 无法动态加载/卸载插件
</think>

现在我对整个引擎有了全面的理解。让我给出深度的架构分析和抽象设计建议。

---

# ToyEngine 架构问题分析与抽象设计建议

## 一、当前架构现状

### 1.1 模块依赖关系

```
┌─────────────────────────────────────────────────────────────────────┐
│                          demo_app.rs                                 │
│           (业务层：场景配置 + 用户输入处理)                           │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       engine-app crate                              │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ App trait + AppRunner                                          ││
│  │ - systems(): SystemSchedule (注册 ECS 系统)                    ││
│  │ - on_start() / on_update() / on_render()                      ││
│  └─────────────────────────────────────────────────────────────────┘│
│                              │                                       │
│              ┌───────────────┼───────────────┐                      │
│              ▼               ▼               ▼                      │
│     SystemSchedule    plugins/mod.rs   Engine {                     │
│     (简单调度器)        (Physics/       World, ctx,                  │
│                        Render)          window, input                │
└─────────────────────────────────────────────────────────────────────┘
              │               │               │
              ▼               ▼               ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  engine-core    │  │ engine-renderer │  │ engine-physics  │
│                 │  │                 │  │                 │
│ - ECS 组件      │  │ - MainRenderer │  │ - PhysicsContext│
│ - 输入系统       │  │ - 渲染管线     │  │ - ECS 系统*(未用)│
│ - FrameCounter  │  │ - 材质/网格     │  │ - physics_world │
└─────────────────┘  └─────────────────┘  └─────────────────┘
              │               │               │
              └───────────────┴───────────────┘
                              │
                              ▼
                      bevy_ecs::World
```

### 1.2 已取得的进步

| 特性 | 状态 | 说明 |
|------|------|------|
| 统一 App 接口 | ✅ | `engine-app::App` 是唯一入口 |
| ECS 系统调度器 | ✅ | 基础版 `SystemSchedule` 已实现 |
| 插件系统 | ✅ | `PhysicsPlugin` / `RenderPlugin` 已工作 |
| 物理自动调度 | ✅ | 通过插件自动注册 |
| 相机/网格自动调度 | ✅ | 通过 `RenderPlugin` 自动注册 |

---

## 二、当前设计不足分析

### 2.1 系统调度器过于简化

**现状** (`schedule.rs`):
```rust
pub struct SystemSchedule {
    pub setup_systems: Vec<SystemFn>,   // Box<dyn Fn(&mut World)>
    pub update_systems: Vec<SystemFn>,
}
```

**问题**:
1. **无依赖排序**: 系统按注册顺序执行，无法声明执行先后依赖
2. **无并行支持**: 所有系统串行执行，无法利用多核
3. **无条件执行**: 没有 `run_if()` 机制
4. **无阶段细分**: 只有 setup/update，缺少 FixedUpdate、Render 等细分阶段
5. **无系统状态**: 无法查询系统是否已运行

**对标现代引擎 (Bevy)**:
```rust
// Bevy 的调度器
App::new()
    .add_systems(Update, (physics, camera, input))                           // 并行
    .add_systems(FixedUpdate, physics_fixed_step.run_if(physics_enabled))    // 条件
    .add_systems(Startup, (init_scene, load_assets).in_base_set(StartupSet)) // 阶段
    .configure_set(Update, PhysicsSet::Sync.before(PhysicsSet::Step))        // 依赖
```

---

### 2.2 物理系统与 ECS 未完全集成

**现状**:
- `physics_world::update()` 手动调用了 `init_bodies()` → `step()` → `sync_transforms()`
- 5 个 ECS 系统定义在 `lib.rs` 但**未被调度**:
  - `apply_external_forces_system`
  - `init_physics_bodies_system`
  - `sync_transform_to_physics_system`
  - `step_physics_system`
  - `sync_physics_to_transform_system`

**问题**:
- 物理系统实际上是"手动模式"，没有利用 ECS 的变更检测 (`Changed<T>`)
- 无法在 ECS 层面控制物理系统的开启/关闭
- 物理配置 (`PhysicsConfig`) 与 `PhysicsContext` 分离

---

### 2.3 资源管理系统缺失

**现状**:
- `AssetManager` 只支持同步加载本地 RON 文件
- 渲染器的 `model_cache` / `mesh_cache` 是简单的 `HashMap<String, Arc<T>>`
- 资源路径硬编码: `"assets/materials"`, `"assets/models/monkey.glb"`

**问题**:
- 阻塞式加载，无异步支持
- 无资源依赖追踪（加载场景 A 前需先加载材质 B）
- 无热重载能力
- 无资源生命周期管理（何时释放未使用的资源）

**对标现代引擎**:
```rust
// 理想设计
let handle: Res<AssetServer>.load("models/monkey.glb");
// 异步加载完成自动触发回调
asset_server.watch_for_changes().unwrap();
```

---

### 2.4 输入系统未接入 ECS

**现状**:
```rust
pub struct Engine {
    pub world: World,
    pub input: InputState,  // 独立状态，未插入 World
}
```

**问题**:
- 输入状态不在 ECS 中，无法通过 Query 查询
- 无法利用 ECS 的变更检测 (`Changed<InputState>`)
- 无法在 ECS 系统中处理输入
- 难以实现输入组合和绑定系统

---

### 2.5 渲染管线缺乏抽象

**现状** (`RendererTrait_MainRenderer.rs`):
```rust
fn render<C: SurfaceContextTrait>(&mut self, ctx: &mut C) {
    // 硬编码顺序
    MeshForwardPass.render(...);
    LinePass.render(...);
    render_ui(...);
}
```

**问题**:
- Pass 顺序写死，无法动态添加/移除 Pass
- 无法配置 Pass 之间的依赖
- 无法实现延迟渲染、光线追踪等高级特性
- 缺少 Render Graph 抽象

---

### 2.6 场景系统缺失

**现状**: Demo 直接在 `on_start()` 中 spawn 实体

**问题**:
- 无法持久化场景到文件
- 无法在编辑器中编辑场景
- 无法热重载场景配置
- 场景切换需要手动处理资源释放

---

## 三、抽象设计建议

### 3.1 增强型系统调度器

```rust
// ============================================================================
// 阶段定义 - 替代简单的 setup/update
// ============================================================================

/// 系统调度阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemStage {
    /// 预启动阶段（引擎初始化前）
    PreStartup,
    /// 启动阶段（加载资源、初始化场景）
    Startup,
    /// 预更新阶段（处理输入、准备数据）
    PreUpdate,
    /// 主更新阶段（游戏逻辑）
    Update,
    /// 固定更新阶段（物理模拟）
    FixedUpdate,
    /// 后更新阶段（同步、清理）
    PostUpdate,
    /// 渲染阶段（提交渲染命令）
    Render,
}

/// 系统集合 - 用于组织相关系统
#[derive(Debug, Clone)]
pub struct SystemSet {
    pub stage: SystemStage,
    pub run_if: Option<Box<dyn Fn(&World) -> bool + Send + Sync>>,
    pub before: Vec<SystemSetId>,
    pub after: Vec<SystemSetId>,
}

// ============================================================================
// 系统调度器 (兼容现有设计，渐进增强)
// ============================================================================

pub struct SystemSchedule {
    /// 系统阶段映射
    stages: HashMap<SystemStage, Vec<SystemFn>>,
    /// 系统集合（用于依赖排序）
    sets: HashMap<String, SystemSet>,
    /// 集合成员关系
    set_members: HashMap<String, Vec<SystemFn>>,
}

impl SystemSchedule {
    /// 添加带阶段的系统
    pub fn add_system<F>(mut self, system: F, stage: SystemStage) -> Self 
    where F: Fn(&mut World) + Send + Sync + 'static 
    {
        self.stages
            .entry(stage)
            .or_insert_with(Vec::new)
            .push(Box::new(system));
        self
    }

    /// 添加条件系统
    pub fn add_system_with_condition<F, C>(
        self, system: F, stage: SystemStage, condition: C
    ) -> Self 
    where 
        F: Fn(&mut World) + Send + Sync + 'static,
        C: Fn(&World) -> bool + Send + Sync + 'static,
    {
        // 实现...
        self
    }

    /// 添加系统集合（带依赖声明）
    pub fn configure_set(mut self, name: &str, set: SystemSet) -> Self {
        self.sets.insert(name.to_string(), set);
        self
    }

    /// 运行指定阶段
    pub fn run_stage(&self, world: &mut World, stage: SystemStage) {
        if let Some(systems) = self.stages.get(&stage) {
            for system in systems {
                system(world);
            }
        }
    }

    /// 运行完整调度
    pub fn run(&self, world: &mut World) {
        // 按顺序执行所有阶段
        for stage in [
            SystemStage::Startup,
            SystemStage::PreUpdate,
            SystemStage::Update,
            SystemStage::FixedUpdate,
            SystemStage::PostUpdate,
            SystemStage::Render,
        ] {
            self.run_stage(world, stage);
        }
    }
}
```

---

### 3.2 完整的物理 ECS 集成

```rust
// ============================================================================
// 物理系统插件 - 完全基于 ECS
// ============================================================================

pub struct PhysicsPlugin {
    config: PhysicsConfig,
}

impl PhysicsPlugin {
    pub fn build(&self, schedule: &mut SystemSchedule) {
        schedule
            // 初始化阶段：创建物理刚体
            .add_system(
                init_physics_bodies_system,
                SystemStage::Startup,
            )
            // 预更新：应用外力（可能需要先于物理步进）
            .add_system(
                apply_external_forces_system,
                SystemStage::PreUpdate,
            )
            // 固定更新：物理模拟
            .add_system(
                step_physics_system,
                SystemStage::FixedUpdate,
            )
            // 后更新：同步物理位置到 Transform
            .add_system(
                sync_physics_to_transform_system,
                SystemStage::PostUpdate,
            );
    }
}

// ============================================================================
// 物理组件 - 增强设计
// ============================================================================

/// 物理初始化标记 - 用于区分已初始化和未初始化实体
#[derive(Component)]
pub struct PhysicsInitialized;

/// 物理查询组件 - 用于射线检测等
#[derive(Component)]
pub struct PhysicsQuery {
    pub query_type: PhysicsQueryType,
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: f32,
}

pub enum PhysicsQueryType {
    RayCast,
    ShapeCast,
    PointProject,
}
```

---

### 3.3 ECS 化输入系统

```rust
// ============================================================================
// 输入资源 - 插入 ECS World
// ============================================================================

/// 输入状态资源
#[derive(Resource)]
pub struct InputContext {
    /// 当前帧按键状态
    pub keyboard: KeyboardInputState,
    /// 当前帧鼠标状态
    pub mouse: MouseInputState,
    /// 当前帧手柄状态
    pub gamepad: GamepadInputState,
}

#[derive(Default)]
pub struct KeyboardInputState {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl KeyboardInputState {
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }
    
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }
}

/// 鼠标输入状态
#[derive(Default)]
pub struct MouseInputState {
    pub position: Vec2,
    pub delta: Vec2,
    pub wheel_delta: Vec2,
    pub buttons_pressed: HashSet<MouseButton>,
}

/// 输入事件组件 - 附加到实体以响应特定输入
#[derive(Component)]
pub struct InputReceiver {
    pub required_keys: HashSet<KeyCode>,
    pub mode: InputMode,
}

#[derive(Clone, Copy)]
pub enum InputMode {
    /// 任意按键触发
    AnyKey,
    /// 所有按键同时按下才触发
    AllKeys,
}
```

---

### 3.4 资源管理系统

```rust
// ============================================================================
// 资源加载器抽象
// ============================================================================

/// 资源句柄 - 异步加载的核心
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle(Uuid);

/// 资源元数据
pub struct AssetMetadata {
    pub handle: AssetHandle,
    pub path: AssetPath,
    pub loader_id: AssetLoaderId,
    pub dependencies: Vec<AssetHandle>,
    pub load_status: LoadStatus,
}

/// 资源加载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStatus {
    NotLoaded,
    Loading,
    Loaded,
    Failed(Error),
}

/// 资源服务器 - 核心资源管理
#[derive(Resource)]
pub struct AssetServer {
    loaders: HashMap<AssetLoaderId, Box<dyn AssetLoader>>,
    assets: HashMap<AssetHandle, Arc<dyn Any + Send + Sync>>,
    metadata: HashMap<AssetHandle, AssetMetadata>,
    load_queue: crossbeam_channel::Sender<LoadRequest>,
}

pub trait AssetLoader {
    fn loader_id(&self) -> AssetLoaderId;
    fn extensions(&self) -> &[&str];
    fn load(&self, bytes: &[u8]) -> Result<Box<dyn Asset>, Error>;
}

/// 通用资源 trait
pub trait Asset: Send + Sync {
    fn name(&self) -> &str;
}

// ============================================================================
// 资源组件 - 用于引用资源
// ============================================================================

/// 模型资源引用
#[derive(Component)]
pub struct ModelRef(pub AssetHandle);

/// 材质资源引用
#[derive(Component)]
pub struct MaterialRef(pub AssetHandle);

/// 纹理资源引用
#[derive(Component)]
pub struct TextureRef(pub AssetHandle);
```

---

### 3.5 渲染管线抽象

```rust
// ============================================================================
// 渲染图抽象
// ============================================================================

/// 渲染节点 - 代表一个渲染操作
pub trait RenderNode: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, ctx: &mut RenderContext);
    fn inputs(&self) -> Vec<Port>;
    fn outputs(&self) -> Vec<Port>;
}

/// 渲染端口 - 连接渲染节点
#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub type_id: TypeId,
}

/// 渲染图
pub struct RenderGraph {
    nodes: HashMap<String, Box<dyn RenderNode>>,
    edges: Vec<(Port, Port)>,  // (source, target)
}

impl RenderGraph {
    /// 添加渲染节点
    pub fn add_node<N: RenderNode + 'static>(&mut self, name: &str, node: N) {
        self.nodes.insert(name.to_string(), Box::new(node));
    }

    /// 连接节点
    pub fn add_edge(&mut self, from: (&str, &str), to: (&str, &str)) {
        // (node_name, port_name)
    }

    /// 执行渲染图
    pub fn execute(&self, ctx: &mut RenderContext) {
        // 拓扑排序后执行
        for node in self.nodes.values() {
            node.execute(ctx);
        }
    }
}

// ============================================================================
// 渲染 Pass 抽象
// ============================================================================

pub trait RenderPass: Send + Sync {
    fn name(&self) -> &str;
    fn prepare(&mut self, renderer: &mut MainRenderer, world: &World);
    fn render(
        &self, 
        encoder: &mut wgpu::CommandEncoder, 
        view: &wgpu::TextureView,
    );
}

/// 注册到渲染器的 Pass
#[derive(Resource)]
pub struct RenderPipeline {
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderPipeline {
    pub fn add_pass(&mut self, pass: impl RenderPass + 'static) {
        self.passes.push(Box::new(pass));
    }
}
```

---

### 3.6 场景系统

```rust
// ============================================================================
// 场景定义
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub entities: Vec<SceneEntity>,
}

#[derive(Serialize, Deserialize)]
pub struct SceneEntity {
    pub components: HashMap<String, ComponentData>,
}

#[derive(Serialize, Deserialize)]
pub enum ComponentData {
    Transform { translation: Vec3, rotation: Quat, scale: Vec3 },
    Camera3D { position: Vec3, forward: Vec3 },
    RigidBody { body_type: String, mass: f32 },
    // ... 其他组件
}

/// 场景加载器
#[derive(Resource)]
pub struct SceneLoader {
    asset_server: Res<AssetServer>,
}

impl SceneLoader {
    pub fn load_scene(&self, path: &str) -> impl Future<Output = Scene> {
        async move {
            let bytes = self.asset_server.load_raw(path).await;
            serde_json::from_slice(&bytes).unwrap()
        }
    }

    pub fn spawn_scene(&self, world: &mut World, scene: &Scene) {
        for entity in &scene.entities {
            // 反序列化并 spawn
        }
    }
}
```

---

### 3.7 插件系统增强

```rust
// ============================================================================
// 统一插件接口
// ============================================================================

pub trait Plugin {
    fn name(&self) -> &str;
    
    /// 插件构建时的回调
    fn build(&self, app: &mut AppBuilder);
}

/// 应用构建器 - 替代当前的 App + SystemSchedule
pub struct AppBuilder {
    world: World,
    schedule: SystemSchedule,
    plugins: Vec<Box<dyn Plugin>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedule: SystemSchedule::new(),
            plugins: Vec::new(),
        }
    }

    pub fn add_plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        plugin.build(&mut self);
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn add_systems<M>(mut self, stage: SystemStage, systems: M) -> Self 
    where M: IntoSystemTuple,
    {
        // 添加多个系统
        self
    }

    pub fn insert_resource<R: Resource>(mut self, resource: R) -> Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn run(mut self) -> ! {
        loop {
            self.schedule.run(&mut self.world);
        }
    }
}

// ============================================================================
// 插件示例
// ============================================================================

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn name(&self) -> &str { "Physics" }
    
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(PhysicsContext::new())
           .add_systems(SystemStage::FixedUpdate, (
               init_physics_bodies_system,
               step_physics_system,
               sync_physics_to_transform_system,
           ));
    }
}
```

---

## 四、演进路线建议

### 阶段 1: 完善 ECS 系统调度（短期）

| 任务 | 优先级 | 工作量 |
|------|--------|--------|
| 添加系统阶段 | P0 | 小 |
| 启用物理 ECS 系统 | P0 | 中 |
| ECS 化输入 | P1 | 中 |
| 修复资源路径硬编码 | P1 | 小 |

### 阶段 2: 资源与渲染抽象（中期）

| 任务 | 优先级 | 工作量 |
|------|--------|--------|
| 异步资源加载器 | P0 | 大 |
| 渲染图抽象 | P1 | 大 |
| 场景序列化 | P1 | 中 |

### 阶段 4: 高级特性（长期）

| 任务 | 优先级 | 工作量 |
|------|--------|--------|
| 插件热插拔 | P0 | 大 |
| 编辑器集成 | P1 | 很大 |
| 网络多人 | P2 | 大 |

---

## 五、总结

ToyEngine 当前已经具备了游戏引擎的核心骨架：
- ✅ 统一的应用入口
- ✅ 基础 ECS 系统
- ✅ 渲染、物理基础能力
- ✅ 简化的插件系统

但与现代游戏引擎（如 Bevy、Godot）相比，在以下方面存在明显差距：

1. **系统调度**: 需要支持依赖排序、并行执行、条件系统
2. **物理集成**: 需要完全 ECS 化，而非手动调用
3. **资源管理**: 需要异步加载、依赖追踪、热重载
4. **输入系统**: 需要接入 ECS
5. **渲染管线**: 需要渲染图抽象
6. **场景系统**: 需要序列化/反序列化支持

通过上述抽象设计，可以逐步将 ToyEngine 演进为一个更现代、更健壮的游戏引擎。