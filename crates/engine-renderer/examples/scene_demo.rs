// use engine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait};
// use engine_renderer::renderer::{
//     create_simple_mesh3d_resources, draw_simple_mesh3d_pass, SimpleMesh3DPassConfig,
//     SimpleMesh3DResources, SurfaceContextTrait, SurfaceSize,
// };
// use engine_core::camera::camera3d_fly_wasd;
// use engine_core::ecs::{Camera3D, Transform, World};

// use glam::{Mat4, Quat, Vec3};

// const MAX_FRAMES: u32 = 600;

// fn main() {
//     RunApp::run_app(
//         AppConfig {
//             title: "Scene Demo",
//             max_frames: Some(240),
//             fixed_dt_seconds: Some(1.0 / 60.0),
//         },
//         SceneDemoApp {
//             pipe: None,
//             frames: 0,
//         },
//     );
// }