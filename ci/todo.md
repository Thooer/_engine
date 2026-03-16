# 我建议的最终形态

假设目录是：

```text
src/user/
  mod.rs
  User.rs
```

其中：

* `mod.rs` 里写接口
* `User.rs` 里写真正实现

---

## `mod.rs`

```rust
use separate_inherent::separate_inherent;

pub struct User {
    name: String,
}

separate_inherent! {
    impl User {
        pub fn new(name: String) -> Self;
        pub fn name(&self) -> &str;
        pub fn rename(&mut self, name: String);
    }
}
```

这里就已经把“固有实现的 API 部分”放进过程宏里了。

---

## `User.rs`

```rust
impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename(&mut self, name: String) {
        self.name = name;
    }
}
```

---

# 这个方案的语义

`separate_inherent!` 做这些事：

1. 解析宏里的

```rust
impl User {
    ...
}
```

拿到：

* 类型名 `User`
* API 签名列表

2. 根据类型名自动推导实现文件路径：

* 当前调用文件所在目录
* 拼上 `User.rs`

也就是如果宏在 `src/user/mod.rs` 里调用，就去找：

```text
src/user/User.rs
```

3. 读取 `User.rs`

4. 解析里面的：

```rust
impl User { ... }
```

5. 校验签名和宏里的声明一致

6. 直接生成最终的：

```rust
impl User {
    ...
}
```

这就是**真实固有实现**

---

# 这套设计为什么比前面的更对

因为你真正想要的是：

## 1. `mod.rs` 是“接口入口”

只看 `mod.rs`，别人就知道这个类型有哪些固有方法。

## 2. 实现文件是自动发现的

不用额外写字段，减少噪音。

## 3. 文件命名自然

`User.rs` 一看就是 `User` 的固有实现文件。

## 4. 过程宏只负责“接口声明 + 装配”

不需要额外的 api 文件。

这个设计明显更贴合“前后端分离”的直觉。

---

# 我建议的命名规则

你说“实现文件就叫做固有实现的那个 impl 的名字.rs”。

那最自然就是：

```rust
impl User { ... }  ->  User.rs
impl Account { ... } -> Account.rs
```

也就是：

* 取 `impl` 后面的 self type 的最后一个标识符
* 文件名 = `<TypeName>.rs`

---

## 推荐先只支持最简单这一类

也就是只支持：

```rust
impl User { ... }
```

先**不要**支持这些复杂情况：

```rust
impl<T> User<T> { ... }
impl crate::user::User { ... }
impl my_mod::User { ... }
impl User<'a> { ... }
```

因为一旦支持这些，文件名推导会立刻复杂起来：

* `User<T>.rs` 不合法
* `crate::user::User.rs` 不适合作文件名
* 路径型 self type 要不要映射目录
* 泛型参数要不要忽略

所以第一版最好限制成：

* `self_ty` 必须是简单标识符类型 `User`
* 实现文件就是 `User.rs`

这样最稳。

---

# 宏输入 DSL 的最终推荐

就这个：

```rust
separate_inherent! {
    impl User {
        pub fn new(name: String) -> Self;
        pub fn name(&self) -> &str;
        pub fn rename(&mut self, name: String);
    }
}
```

我认为这是你这套设计里最优雅的形式。

优点：

* 像 Rust
* API 就地声明
* 宏输入短
* 自动推导实现文件
* `mod.rs` 一眼能懂

---

# 实现文件应该长什么样

我建议实现文件也保持**完整 `impl` 块**，不要只放方法列表。

也就是：

```rust
impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename(&mut self, name: String) {
        self.name = name;
    }
}
```

而不是：

```rust
pub fn new(name: String) -> Self { ... }
pub fn name(&self) -> &str { ... }
```

原因很简单：

* 更像原生 Rust
* 更好校验 `impl` 的 self type
* 后续扩展泛型 / where / 属性时更自然
* 出错信息更直观

---

# 路径推导规则

这个规则最好定死：

如果宏调用点文件是：

```text
src/user/mod.rs
```

并且宏输入是：

```rust
impl User { ... }
```

那么实现文件就是：

```text
src/user/User.rs
```

也就是：

* 取调用点文件所在目录
* 取 `impl` 的类型名 `User`
* 拼成 `User.rs`

---

# 这套机制的边界

你这个设计成立的前提是：

## 只支持“简单命名类型的固有 impl”

支持：

```rust
impl User { ... }
```

不支持或先不支持：

```rust
impl<T> User<T> { ... }
impl crate::m::User { ... }
impl super::User { ... }
impl User<'a> { ... }
```

否则“文件名自动推导”会变得不自然。

---

# 解析模型该怎么定

## 宏里的 API 部分

解析成：

```rust
struct ApiImpl {
    self_ty: syn::TypePath,
    methods: Vec<ApiMethod>,
}
```

每个方法：

```rust
struct ApiMethod {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    sig: syn::Signature,
}
```

要求：

* 方法必须以 `;` 结束
* 不能有 body

---

## 文件里的实现部分

解析成：

```rust
struct ImplFile {
    self_ty: syn::TypePath,
    methods: Vec<ImplMethod>,
}
```

每个方法：

```rust
struct ImplMethod {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    sig: syn::Signature,
    block: Box<syn::Block>,
}
```

要求：

* 必须有 body

---

# 一致性校验怎么做

宏里声明的是契约，实现文件里必须逐项匹配。

至少校验：

* 方法名
* receiver：`self` / `&self` / `&mut self`
* 参数个数
* 参数类型
* 返回类型
* `async`
* `const`
* `unsafe`
* 泛型参数
* `where` 子句
* 可见性 `pub`

建议**不强制属性完全一致**，例如 `#[inline]` 可以只存在于实现文件。

---

# 最终展开结果

宏最终直接展开成：

```rust
impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename(&mut self, name: String) {
        self.name = name;
    }
}
```

所以从使用者角度：

* 这就是原生固有实现
* 没有 trait
* 没有转发
* 没有 `include!`
* 只有过程宏装配

---

# 我建议你再加一条小规则

为了避免一个类型多个分离入口冲突，建议约定：

**同一个模块目录下，一个类型只允许一个 `separate_inherent! { impl Type { ... } }`。**

否则如果有人在两个地方都写：

```rust
separate_inherent! {
    impl User { ... }
}
```

那都去找 `User.rs`，语义会乱。

---

# 我帮你收敛后的最终方案

## 目录

```text
src/user/
  mod.rs
  User.rs
```

## `mod.rs`

```rust
use separate_inherent::separate_inherent;

pub struct User {
    name: String,
}

separate_inherent! {
    impl User {
        pub fn new(name: String) -> Self;
        pub fn name(&self) -> &str;
        pub fn rename(&mut self, name: String);
    }
}
```

## `User.rs`

```rust
impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename(&mut self, name: String) {
        self.name = name;
    }
}
```

## 展开后

```rust
impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename(&mut self, name: String) {
        self.name = name;
    }
}
```

---

这套设计我认同，确实比我前面说的那版更干净。

下一步真正关键的是：
**`syn` 里怎么解析宏输入里的“无 body 方法签名”，以及怎么从过程宏里定位调用点目录并读取 `User.rs`。**
