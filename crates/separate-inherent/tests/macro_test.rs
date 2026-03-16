// 测试 separate_inherent 宏
// 这个文件演示了宏的用法

use separate_inherent::separate_inherent;

mod user {
    use separate_inherent::separate_inherent;
    
    pub struct User {
        name: String,
    }

    // 使用新的接口：显式传递实现文件路径
    separate_inherent!("user/User.rs", {
        impl User {
            fn new(name: String) -> Self;
            fn name(&self) -> &str;
            fn set_name(&mut self, name: String);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::user::User;

    #[test]
    fn test_user() {
        let mut user = User::new("Alice".to_string());
        assert_eq!(user.name(), "Alice");
        user.set_name("Bob".to_string());
        assert_eq!(user.name(), "Bob");
    }
}
