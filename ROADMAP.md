# yadig — 项目规划与路线图

> 最后更新: 2026-05-26

## 一、项目定位

yadig 是一款面向音乐爱好者的桌面/Web 应用，核心能力是**音乐发现与资源挖掘**：

- **音乐资讯聚合** — 用户自定义信息源，自动抓取、去重、摘要，按周期推送
- **音乐资源挖掘** — 多关键词模糊搜索 + 多数据源交叉查询 + LLM 智能整合
- **音频资源获取** — 从合法免费源获取可直接播放/下载的音频，摆脱流媒体依赖

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

---

## 六、音频资源挖掘功能

### 背景与目标

当前 yadig 的搜索结果只能找到音乐资讯（评论、新闻、专辑信息），但用户真正想要的是**可直接播放/下载的音频资源**，摆脱对 Spotify/Apple Music 等流媒体的依赖。

### 调研结果：合法免费音频源

| 音频源 | 类型 | 音频可用性 | API 可用性 | 接入难度 | 备注 |
|---|---|---|---|---|---|
| **Jamendo** | API | 直接提供 stream/download URL | 官方 API v3.0，免费注册 | 低 | 60万+独立音乐，CC 授权，支持 mp3/ogg/flac |
| **Internet Archive** | API | 提供音频文件直接下载 | Advanced Search API + Metadata API | 中 | 海量公共领域音频，需两步获取（搜索+元数据） |
| **Free Music Archive** | 已归档 | 在 archive.org 上的子集 | 同 Internet Archive | 中 | 10万+ CC 授权曲目，已由 archive.org 接管 |
| **FreePD** | Scraper | 提供直接下载链接 | 无 API，需 HTML 爬取 | 低 | CC0 公共领域，约500+首 |
| **Bandcamp** | 部分 | 部分艺术家提供免费下载 | 已有 scraper | 中 | "name your price"（含免费）曲目可下载 |
| **SoundCloud** | 部分 | 部分曲目可下载 | 已关闭公开 API 注册 | 高 | 需要逆向工程，不稳定，不建议 |
| **YouTube** | 音频提取 | 需要从视频中提取音频 | 无官方 API | 高 | 灰色地带，技术复杂，不建议 |

### 推荐接入方案（按优先级）

#### 第一优先：Jamendo（API 源）

**为什么首选：**
- 官方 API 免费提供，月限 35,000 次请求
- 直接返回 `audio`（流式播放 URL）和 `audiodownload`（下载 URL）
- 60万+独立音乐，覆盖全流派
- 返回数据包含封面图、专辑信息、标签、CC 授权信息

**接入方式：**
```rust
// src-tauri/src/source/api/jamendo.rs
pub struct JamendoSource {
    client: reqwest::Client,
    client_id: Arc<RwLock<Option<String>>>, // 用户注册后提供
}

// 搜索 API:
// GET https://api.jamendo.com/v3.0/tracks/?client_id=XXX&search=query&format=json&include=musicinfo&audioformat=mp32

// 关键返回字段:
// - audio: 流式播放 URL (mp31=96kbps, mp32=VBR)
// - audiodownload: 下载 URL (需检查 audiodownload_allowed)
// - album_image: 封面图
// - artist_name, album_name, name: 元数据
// - license_ccurl: CC 授权信息
```

**ContentItem 扩展：**
```rust
ContentItem {
    source_id: "jamendo",
    title: "Artist — Track Name",
    url: "https://www.jamendo.com/track/1848357",           // 页面链接
    summary: Some("Album Name"),
    author: Some("Artist Name"),
    published_at: Some("2021-04-11"),
    image_url: Some("https://usercontent.jamendo.com/..."),  // 封面图
    extra: Some(json!({
        "audio_url": "https://prod-1.storage.jamendo.com/?trackid=1848357&format=mp32&from=app-yadig",
        "download_url": "https://prod-1.storage.jamendo.com/download/track/1848357/mp32/",
        "download_allowed": true,
        "duration": 272,
        "genres": ["rock"],
        "license": "https://creativecommons.org/licenses/by-nc-nd/3.0/"
    })),
}
```

#### 第二优先：Internet Archive（API 源）

**接入方式：**
```rust
// src-tauri/src/source/api/archive_org.rs
pub struct ArchiveOrgSource {
    client: reqwest::Client,
}

// 搜索: GET https://archive.org/advancedsearch.php?q=mediatype:audio+AND+title:query&output=json&rows=20
// 元数据: GET https://archive.org/metadata/{identifier}
// 下载: GET https://archive.org/download/{identifier}/{filename}
```

**两步工作流：**
1. 通过 Advanced Search API 搜索 `mediatype:audio` 的条目
2. 对每个结果调用 Metadata API 获取 `files` 列表，筛选 `.mp3`/`.ogg`/`.flac` 文件
3. 构造直接下载 URL: `https://archive.org/download/{id}/{filename}`

#### 第三优先：FreePD（Scraper 源）

**接入方式：**
```rust
// src-tauri/src/source/scraper/freepd.rs
pub struct FreePDSource {
    client: reqwest::Client,
}

// 爬取 https://freepd.com 页面
// 每首歌都有直接的 MP3 下载链接
// 全部 CC0 公共领域授权
```

### 架构变更

#### 1. 扩展 ContentItem（后端 + 前端）

```rust
// src-tauri/src/source/types.rs
pub struct ContentItem {
    // ... 现有字段 ...
    pub audio_url: Option<String>,      // 新增：可直接播放的音频 URL
    pub download_url: Option<String>,   // 新增：可直接下载的 URL
    pub duration: Option<u32>,          // 新增：时长（秒）
    pub license: Option<String>,        // 新增：授权信息
}
```

```typescript
// src/types/source.ts
export interface ContentItem {
  // ... 现有字段 ...
  audioUrl?: string;
  downloadUrl?: string;
  duration?: number;
  license?: string;
}
```

#### 2. 添加音频播放器组件（前端）

```typescript
// src/components/audio-player.tsx
// 全局浮动播放器，支持播放/暂停、进度条、音量
// 从 ContentItem.audioUrl 获取流式播放地址
```

#### 3. 前端搜索结果增强

- 有 `audioUrl` 的结果显示播放按钮
- 有 `downloadUrl` 的结果显示下载按钮
- 显示时长和授权信息
- 可以内联试听（点击播放，不离开页面）

#### 4. 设置页面新增 Jamendo 配置

- Jamendo Client ID 输入（用户在 devportal.jamendo.com 注册获取）
- 存储到 tauri-plugin-store
- 启动时通过 DiscogsKeys 相同的模式注入

### 实现计划

#### Phase 1.5（补充）: 搜索结果体验优化

- [x] 搜索结果链接可点击/复制
- [x] 图片展示优化（24x24 → lazy loading）
- [ ] 搜索结果 URL 完整显示（显示路径而非仅域名）
- [ ] 复制全文链接按钮

#### Phase 2（调整）: 音频资源挖掘

**Step 1: Jamendo 源接入**
- [ ] 注册 Jamendo 开发者账号，获取 client_id
- [ ] 实现 `JamendoSource`（SourceProvider trait）
- [ ] 扩展 ContentItem 添加 audio_url/download_url 字段
- [ ] 设置页面新增 Jamendo Client ID 管理
- [ ] 注册到 SourceRegistry

**Step 2: 音频播放器**
- [ ] 全局浮动 AudioPlayer 组件（播放/暂停/进度/音量）
- [ ] 搜索结果卡片集成播放按钮
- [ ] Tauri 后端音频流代理（避免 CORS 问题）
- [ ] 播放状态持久化（记住上次播放位置）

**Step 3: Internet Archive 源接入**
- [ ] 实现 `ArchiveOrgSource`（Advanced Search + Metadata 两步查询）
- [ ] 筛选音频文件（mp3/ogg/flac）
- [ ] 集成到搜索和 feed

**Step 4: 下载管理**
- [ ] Tauri 命令：download_audio(url, path)
- [ ] 下载进度回调
- [ ] 下载目录设置
- [ ] 下载历史管理

**Step 5: 更多音频源**
- [ ] FreePD scraper
- [ ] Bandcamp 免费下载检测（"name your price" 曲目）

#### Phase 3（调整）: LLM 智能搜索 + 资讯聚合

（原计划不变，LLM 可用于音频推荐）
