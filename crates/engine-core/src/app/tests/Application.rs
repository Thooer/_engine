//! Application trait 测试

use crate::app::Application;

/// 测试用的简单 Application 实现
struct TestApp {
    init_called: bool,
    update_count: u32,
    shutdown_called: bool,
}

impl TestApp {
    fn new() -> Self {
        Self {
            init_called: false,
            update_count: 0,
            shutdown_called: false,
        }
    }
}

impl Application for TestApp {
    fn init(&mut self) {
        self.init_called = true;
    }

    fn update(&mut self, _dt: f32) {
        self.update_count += 1;
    }

    fn shutdown(&mut self) {
        self.shutdown_called = true;
    }
}

#[test]
fn test_application_lifecycle() {
    let mut app = TestApp::new();

    // 测试初始化
    assert!(!app.init_called);
    app.init();
    assert!(app.init_called);

    // 测试更新
    assert_eq!(app.update_count, 0);
    app.update(0.016); // 模拟 60 FPS
    assert_eq!(app.update_count, 1);
    app.update(0.016);
    assert_eq!(app.update_count, 2);

    // 测试关闭
    assert!(!app.shutdown_called);
    app.shutdown();
    assert!(app.shutdown_called);
}
