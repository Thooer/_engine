use engine_core::engine::EngineConfig;

// 不再需要为 EngineConfig 实现 Default，因为 engine-core 已经提供了
// AppConfig 是 EngineConfig 的别名，使用 engine_core::engine::EngineConfig::default()

