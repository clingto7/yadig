# yadig 开发者指南

本指南面向希望参与 yadig 开发的开发者，特别是 Rust 初学者。我们将从项目技术栈开始，逐步深入到具体代码实现细节。

## 一、技术栈概览

yadig 是一个桌面应用，采用前后端分离架构：

| 层 | 技术 | 作用 |
|---|---|---|
| 前端 | React 19 + TypeScript | 用户界面 |
| 样式 | Tailwind CSS v4 | 原子化 CSS |
| 状态管理 | @tanstack/react-query | 异步数据管理 |
| 后端 | Rust | 核心业务逻辑 |
| 桌面框架 | Tauri 2 | 前后端通信、窗口管理 |
| 数据库 | SQLite | 本地数据存储 |
| 设置存储 | tauri-plugin-store | JSON 文件持久化 |

### 为什么选择 Rust？

1. **性能** — 并发抓取多个音乐源，Rust 的 async/await 零开销抽象
2. **安全** — 编译期内存安全，无 GC 停顿
3. **Tauri 原生支持** — Tauri 2 的后端就是 Rust

## 二、Rust 基础（结合项目代码）

### 2.1 项目结构

```
src-tauri/
├── Cargo.toml          # 依赖声明（类似 package.json）
├── src/
│   ├── main.rs         # 入口点
│   ├── lib.rs          # 应用初始化
│   ├── error.rs        # 错误类型定义
│   ├── config.rs       # 运行时配置
│   ├── commands/       # Tauri 命令（前端可调用的函数）
│   │   ├── mod.rs
│   │   └── search.rs
│   └── source/         # 音乐源实现
│       ├── mod.rs
│       ├── types.rs    # 数据结构定义
│       ├── provider.rs # 核心 trait
│       ├── registry.rs # 源注册与并发执行
│       ├── rss/        # RSS 源实现
│       ├── api/        # API 源实现
│       └── scraper/    # 爬虫源实现
└── migrations/         # SQL 迁移文件
```

### 2.2 Cargo.toml — 依赖管理

```toml
[package]
name = "yadig"
version = "0.1.0"
edition = "2021"        # Rust 版本（类似 ES2021）

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }  # 序列化框架
serde_json = "1"        # JSON 处理
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }  # HTTP 客户端
async-trait = "0.1"     # 在 trait 中使用 async
thiserror = "2"         # 错误处理派生宏
tokio = { version = "1", features = ["time"] }  # 异步运行时
scraper = "0.22"        # HTML 解析
rss = "2"               # RSS 解析
futures = "0.3"         # 异步工具（join_all 等）
```

**关键概念：**
- `features` — Cargo 允许按需启用功能，减小编译体积
- `derive` — 自动实现 trait（如 `Serialize`、`Deserialize`）
- `async-trait` — Rust 原生 async trait 还不稳定，用这个 crate 桥接

### 2.3 所有权系统 — Rust 的核心

Rust 的所有权系统是它与其他语言最大的区别。核心规则：

1. **每个值有且仅有一个所有者**
2. **所有者离开作用域时，值被自动释放（drop）**
3. **同一时刻只能有一个可变引用，或多个不可变引用**

看项目中的实际例子：

```rust
// src-tauri/src/config.rs
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Clone)]
pub struct DiscogsKeys {
    pub key: Arc<RwLock<Option<String>>>,
    pub secret: Arc<RwLock<Option<String>>>,
}
```

**逐层解析：**

| 类型 | 含义 |
|---|---|
| `String` | 堆上分配的字符串（拥有所有权） |
| `Option<String>` | 可选值，可能是 `Some("key")` 或 `None` |
| `RwLock<Option<String>>` | 读写锁保护的值，允许多读单写 |
| `Arc<RwLock<...>>` | 原子引用计数，允许多线程共享所有权 |

**为什么需要 `Arc<RwLock<...>>`？**

因为 `DiscogsKeys` 需要被两处同时持有：
1. Tauri 的命令处理器（通过 `update_discogs_keys` 命令写入新 key）
2. `DiscogsSource`（在 API 请求时读取 key）

```rust
// lib.rs — 注册时克隆
let discogs_keys = DiscogsKeys::new();
registry.register(Box::new(DiscogsSource::new(discogs_keys.clone())));  // Arc 引用计数 +1
.manage(discogs_keys)  // Tauri 管理另一个引用
```

`clone()` 只是增加 `Arc` 的引用计数（类似 JS 中的浅拷贝），两个副本指向同一块堆内存。

### 2.4 Trait — Rust 的接口

Trait 类似其他语言的接口（interface），定义一组方法签名：

```rust
// src-tauri/src/source/provider.rs
use async_trait::async_trait;

#[async_trait]
pub trait SourceProvider: Send + Sync {
    fn id(&self) -> &str;           // 同步方法，返回字符串引用
    fn name(&self) -> &str;
    fn kind(&self) -> SourceKind;
    fn base_url(&self) -> &str;

    // 异步方法，返回 Result（可能失败）
    async fn search(&self, query: &str, limit: usize, page: usize) -> Result<Vec<ContentItem>>;
    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>>;
}
```

**关键语法解释：**

- `#[async_trait]` — 宏，将 async 方法转换为返回 `Pin<Box<dyn Future>>` 的普通方法，使 trait 可以动态分发
- `Send + Sync` — 标记 trait 对象可以安全地跨线程传递和共享
  - `Send` — 值可以发送到其他线程
  - `Sync` — 值可以被多个线程同时引用
- `&self` — 不可变引用（只读访问）
- `Result<Vec<ContentItem>>` — 返回 `Vec<ContentItem>` 或错误

**实现示例（DiscogsSource）：**

```rust
// src-tauri/src/source/api/discogs.rs
pub struct DiscogsSource {
    client: reqwest::Client,    // HTTP 客户端
    keys: DiscogsKeys,          // API 密钥（Arc 克隆）
}

#[async_trait]
impl SourceProvider for DiscogsSource {
    fn id(&self) -> &str { "discogs" }
    fn name(&self) -> &str { "Discogs" }
    fn kind(&self) -> SourceKind { SourceKind::Api }
    fn base_url(&self) -> &str { "https://www.discogs.com" }

    async fn search(&self, query: &str, limit: usize, page: usize) -> Result<Vec<ContentItem>> {
        let url = self.search_url(query, "release", limit, page);
        let response = self.client.get(&url).send().await?;  // ? 自动传播错误
        // ... 解析响应 ...
        Ok(items)  // 返回成功结果
    }
}
```

### 2.5 错误处理 — Result 和 ?

Rust 没有异常，用 `Result<T, E>` 表示可能失败的操作：

```rust
// src-tauri/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum YadigError {
    #[error("Discogs API error: {0}")]
    Discogs(String),

    #[error("Feed error: {0}")]
    Feed(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("{0}")]
    NotFound(String),
}

// 为 Tauri IPC 实现序列化
impl Serialize for YadigError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: serde::Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

// 类型别名，简化签名
pub type Result<T> = std::result::Result<T, YadigError>;

// 自动转换 reqwest 错误
impl From<reqwest::Error> for YadigError {
    fn from(e: reqwest::Error) -> Self {
        YadigError::Network(e.to_string())
    }
}
```

**`?` 操作符：**

```rust
async fn search(&self, query: &str, limit: usize, page: usize) -> Result<Vec<ContentItem>> {
    let response = self.client.get(&url).send().await?;  // 如果失败，自动 return Err(...)
    let body = response.text().await?;                     // 同上
    // 如果都成功，继续执行
}
```

`?` 等价于：
```rust
let response = match self.client.get(&url).send().await {
    Ok(r) => r,
    Err(e) => return Err(e.into()),  // 自动调用 From 转换
};
```

### 2.6 异步编程 — async/await

Rust 的异步编程基于 `Future` trait 和 `async/await` 语法：

```rust
// async fn 返回一个 Future（惰性的，不会立即执行）
async fn search(&self, query: &str, limit: usize, page: usize) -> Result<Vec<ContentItem>> {
    // .await 暂停当前任务，等待 Future 完成
    let response = self.client.get(&url).send().await?;
    let body = response.text().await?;
    Ok(items)
}
```

**并发执行多个 Future：**

```rust
// src-tauri/src/source/registry.rs
use futures::future::join_all;

pub async fn search(&self, query: &str, ...) -> Result<SearchResult> {
    let mut futures = Vec::new();

    for provider in self.providers.values() {
        if !disabled_ids.contains(provider.id()) {
            futures.push(provider.search(query, limit, page));  // 创建 Future
        }
    }

    // 并发执行所有 Future，等待全部完成
    let results = join_all(futures).await;

    // 收集结果，跳过失败的源
    for res in results {
        match res {
            Ok(r) => items.extend(r),
            Err(e) => eprintln!("Source search error: {}", e),
        }
    }
}
```

**为什么用 `join_all` 而不是循环 `await`？**

```rust
// 串行（慢）：等一个完成再开始下一个
for provider in providers {
    let result = provider.search(query).await;  // 阻塞等待
    items.extend(result);
}

// 并发（快）：同时发起所有请求
let futures: Vec<_> = providers.iter()
    .map(|p| p.search(query))
    .collect();
let results = join_all(futures).await;  // 同时等待所有
```

### 2.7 枚举与模式匹配

Rust 的枚举（enum）比其他语言强大得多，可以携带数据：

```rust
// src-tauri/src/source/types.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Rss,      // 简单变体
    Api,
    Scraper,
}
```

**模式匹配（match）：**

```rust
impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceKind::Rss => write!(f, "rss"),
            SourceKind::Api => write!(f, "api"),
            SourceKind::Scraper => write!(f, "scraper"),
            // 必须穷举所有变体，否则编译报错
        }
    }
}
```

**`Option<T>` — 可选值：**

```rust
// Option 是枚举：Some(value) 或 None
pub struct ContentItem {
    pub summary: Option<String>,    // 可能有摘要，可能没有
    pub author: Option<String>,
    pub image_url: Option<String>,
    pub extra: Option<serde_json::Value>,
}

// 使用模式匹配处理
match item.summary {
    Some(s) => println!("摘要: {}", s),
    None => println!("无摘要"),
}

// 或用 if let（更简洁）
if let Some(s) = &item.summary {
    println!("摘要: {}", s);
}

// 或用 unwrap_or 提供默认值
let summary = item.summary.unwrap_or("无摘要".to_string());
```

### 2.8 迭代器与链式调用

Rust 的迭代器非常强大，类似 JavaScript 的数组方法但更高效：

```rust
// src-tauri/src/source/api/discogs.rs
let items: Vec<ContentItem> = results
    .iter()                    // 创建迭代器
    .filter_map(|r| {         // filter + map，跳过 None
        let title = r["title"].as_str()?.to_string();
        // ... 构建 ContentItem ...
        Some(ContentItem { ... })
    })
    .take(limit)               // 只取前 N 个
    .collect();                // 收集为 Vec
```

**常用迭代器方法：**
- `.map(|x| ...)` — 转换每个元素
- `.filter(|x| ...)` — 过滤元素
- `.filter_map(|x| ...)` — 过滤并转换（跳过 None）
- `.take(n)` — 取前 n 个
- `.collect()` — 收集为集合类型
- `.sort_by(|a, b| ...)` — 排序

### 2.9 宏（Macro）

宏是 Rust 的元编程工具，在编译时展开：

```rust
// #[derive(...)] — 自动实现 trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem { ... }

// #[async_trait] — 将 async 方法转换为兼容的普通方法
#[async_trait]
impl SourceProvider for DiscogsSource { ... }

// #[tauri::command] — 标记为 Tauri IPC 命令
#[tauri::command]
pub async fn search_sources(...) -> Result<SearchResult> { ... }

// #[error("...")] — 定义错误消息格式
#[error("Discogs API error: {0}")]
Discogs(String),

// include_str!("...") — 编译时嵌入文件内容
sql: include_str!("../migrations/001_initial_schema.sql"),

// vec![...] — 创建 Vec 的宏
let feed_urls = vec!["https://...".to_string(), ...];

// format!("...", args) — 格式化字符串（类似 printf）
let url = format!("https://api.discogs.com/search?q={}", query);
```

### 2.10 泛型与生命周期

```rust
// 泛型函数 — 适用于多种类型
pub type Result<T> = std::result::Result<T, YadigError>;

// 泛型结构体
pub struct SearchResult {
    pub items: Vec<ContentItem>,  // Vec<T> 是泛型容器
    // ...
}

// 生命周期标注 — 告诉编译器引用的有效期
fn id(&self) -> &str { "discogs" }
//              ^^ 返回的 &str 的生命周期与 &self 相同
```

## 三、Tauri 前后端通信

### 3.1 Rust 端 — 定义命令

```rust
// src-tauri/src/commands/search.rs
use tauri::State;

#[tauri::command]
pub async fn search_sources(
    registry: State<'_, SourceRegistry>,  // Tauri 注入的状态
    query: String,                         // 从前端传入的参数
    source_ids: Option<Vec<String>>,       // 可选参数
    limit: Option<usize>,
    page: Option<usize>,
) -> Result<SearchResult> {
    let source_ids = source_ids.unwrap_or_default();
    let limit = limit.unwrap_or(20);
    let page = page.unwrap_or(1);
    registry.search(&query, &source_ids, limit, page).await
}
```

### 3.2 TypeScript 端 — 调用命令

```typescript
// src/lib/tauri.ts
import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "@/types/source";

export const tauri = {
  searchSources: (params: {
    query: string;
    sourceIds?: string[];
    limit?: number;
    page?: number;
  }): Promise<SearchResult> => invoke("search_sources", params),
};
```

### 3.3 类型映射

| Rust | TypeScript | 说明 |
|---|---|---|
| `String` | `string` | 自动转换 |
| `usize` | `number` | 自动转换 |
| `bool` | `boolean` | 自动转换 |
| `Vec<T>` | `T[]` | 自动转换 |
| `Option<T>` | `T \| undefined` | None → undefined |
| `HashMap<K,V>` | `Record<K,V>` | 自动转换 |
| `snake_case` | `camelCase` | serde 自动转换字段名 |

## 四、添加新的音乐源

### 步骤 1：创建源文件

```rust
// src-tauri/src/source/api/my_source.rs
use async_trait::async_trait;
use crate::error::Result;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

pub struct MySource {
    client: reqwest::Client,
}

impl MySource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("yadig/0.1.0")
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }
}

#[async_trait]
impl SourceProvider for MySource {
    fn id(&self) -> &str { "my_source" }
    fn name(&self) -> &str { "My Source" }
    fn kind(&self) -> SourceKind { SourceKind::Api }
    fn base_url(&self) -> &str { "https://example.com" }

    async fn search(&self, query: &str, limit: usize, page: usize) -> Result<Vec<ContentItem>> {
        // 实现搜索逻辑
        todo!()
    }

    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>> {
        // 实现获取最新内容
        todo!()
    }
}
```

### 步骤 2：注册源

```rust
// src-tauri/src/lib.rs
mod source;  // 确保模块声明

// 在 run() 函数中注册
registry.register(Box::new(MySource::new()));
```

### 步骤 3：更新模块声明

```rust
// src-tauri/src/source/mod.rs 或对应目录的 mod.rs
pub mod my_source;
```

## 五、调试技巧

### Rust 日志

```rust
// 打印到 stderr（不会显示在前端）
eprintln!("Debug: {:?}", some_value);

// 使用 dbg! 宏（打印文件名、行号、值）
dbg!(&response);
```

### 前端调试

```typescript
// 控制台日志
console.log("Search result:", result);

// React Query 开发者工具
// 已集成在 main.tsx，开发模式下自动显示
```

### 常见编译错误

| 错误 | 原因 | 解决方案 |
|---|---|---|
| `borrow of moved value` | 值已被移动到其他地方 | 使用 `.clone()` 或引用 |
| `cannot borrow as mutable` | 需要可变引用但只有不可变引用 | 使用 `RefCell` 或 `Mutex` |
| `lifetime mismatch` | 引用的生命周期不匹配 | 检查函数签名，可能需要 `'static` |
| `trait bound not satisfied` | 类型没有实现所需 trait | 添加 `#[derive(...)]` 或手动实现 |

## 六、参考资源

- [Rust 官方教程](https://doc.rust-lang.org/book/)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)
- [Tauri 2 文档](https://v2.tauri.app/)
- [serde 序列化框架](https://serde.rs/)
- [reqwest HTTP 客户端](https://docs.rs/reqwest/)
