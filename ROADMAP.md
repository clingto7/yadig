# yadig — 项目规划与路线图

> 最后更新: 2026-05-25

## 一、项目定位

yadig 是一款面向音乐爱好者的桌面/Web 应用，核心能力是**音乐发现与资源挖掘**：

- **音乐资讯聚合** — 用户自定义信息源，自动抓取、去重、摘要，按周期推送
- **音乐资源挖掘** — 多关键词模糊搜索 + 多数据源交叉查询 + LLM 智能整合

## 二、技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2 (Rust 后端 + Web 前端) |
| 前端 | React 19 + TypeScript + Tailwind CSS |
| 状态管理 | @tanstack/react-query |
| 数据库 | SQLite (tauri-plugin-sql) |
| 设置存储 | tauri-plugin-store |
| 通知 | tauri-plugin-notification |
| LLM | 用户自提供 Key (OpenAI/Anthropic/OpenRouter) |

## 三、信息源架构

yadig 的信息源基于 `SourceProvider` trait，支持三种接入方式：

| 类型 | 说明 | 当前实现 |
|---|---|---|
| **RSS** | 结构化 XML，最稳定 | Pitchfork |
| **API** | REST API 调用 | Discogs |
| **Scraper** | HTML 爬虫 | Bandcamp, Album of the Year |

### 已接入的信息源

| 源 | 类型 | 接入方式 | 备注 |
|---|---|---|---|
| **Pitchfork** | RSS | RSS Feed (`/rss/news/`, `/rss/reviews/albums/`, `/rss/best/`) | 最稳定，官方 RSS |
| **Discogs** | API | REST API (`/database/search`) | 需 Consumer Key/Secret（可选） |
| **Bandcamp** | Scraper | HTML 解析搜索结果页 | 无官方 API，CSS 选择器可能需更新 |
| **Album of the Year** | Scraper | HTML 解析搜索和新发行页 | Cloudflare 保护，可能需代理 |

### 扩展方式

添加新信息源只需实现 `SourceProvider` trait：

```rust
#[async_trait]
impl SourceProvider for MyNewSource {
    fn id(&self) -> &str { "my_source" }
    fn name(&self) -> &str { "My Source" }
    fn kind(&self) -> SourceKind { SourceKind::Rss } // or Api, Scraper

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> { ... }
    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>> { ... }
    async fn get_item(&self, url: &str) -> Result<ContentItem> { ... }
}
```

然后在 `lib.rs` 中注册：
```rust
registry.register(Box::new(MyNewSource::new()));
```

## 四、LLM 集成方案

| Provider | 推荐模型 | 输入 $/1M | 输出 $/1M | 特点 |
|---|---|---|---|---|
| OpenAI | GPT-4o mini | $0.15 | $0.60 | 最便宜够用 |
| Anthropic | Claude Haiku 4.5 | $1.00 | $5.00 | 质量好，Prompt Cache 省 90% |
| **OpenRouter** | 多模型路由 | 按模型 | 按模型 | 一个 Key 访问所有模型 |

## 五、开发路线图

### Phase 0: 项目初始化 ✅

- [x] 确定技术方案：Tauri 2 + React + TypeScript
- [x] 搭建项目脚手架
- [x] Rust 后端基础架构 (SourceProvider trait, SourceRegistry)
- [x] 4 个初始信息源 (Pitchfork RSS, Discogs API, Bandcamp Scraper, AOTY Scraper)
- [x] 前端搜索 UI + 侧边栏 + 路由
- [x] SQLite migrations (favorites, search_history, rss_feeds, articles)

### Phase 1: 音乐搜索 MVP

- [x] 完善搜索结果展示（图片、摘要、来源标签）
- [x] 用户自定义 RSS 信息源添加/删除
- [x] 搜索历史与收藏功能
- [x] 设置页面完善（API Key 管理、来源开关）
- [x] 来源过滤（按类型/名称筛选搜索源）
- [x] Bandcamp discover 最新内容
- [ ] Discogs API Key 持久化 (tauri-plugin-store)
- [ ] 来源启用/禁用开关 (持久化)
- [ ] 搜索结果分页/加载更多

### Phase 2: LLM 智能搜索

- [ ] LLMProvider trait 实现 (OpenAI, Anthropic, OpenRouter)
- [ ] API Key 管理界面
- [ ] 搜索结果智能整合
- [ ] 音乐推荐对话模式

### Phase 3: 资讯聚合

- [ ] RSS 采集管道 (fetch → normalize → dedup → store)
- [ ] 定时刷新 + 桌面通知
- [ ] 资讯流 UI (时间线/卡片布局)
- [ ] LLM 资讯摘要

### Phase 4: 打磨与发布

- [ ] UI 精修（动画、主题、无障碍）
- [ ] AppImage / .deb 打包
- [ ] 内测与反馈收集
