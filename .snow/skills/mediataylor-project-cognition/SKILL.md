# MediaTailor Project Cognition Skill

## Purpose

该 skill 用于约束 AI agent 在 MediaTailor 仓库中的分析、设计、编码和文档行为。所有 agent 在处理本项目时，必须优先遵守这里定义的项目架构、语法边界与实现策略。

---

## 1. 项目定位

MediaTailor 是一个基于 Rust 的声明式媒体编排系统，但项目不再沿用单一媒体 DSL 路线，而是采用 **双 DSL + 统一运行时** 的设计：

- `MediaTailor.toml`：项目配置
- `*.mtc`：Canvas DSL，面向图片/画布/图层合成
- `*.mtt`：Timeline DSL，面向视频/轨道/片段编排

目标心智模型：

- **MTC ≈ Photoshop / Figma / 图层排版**
- **MTT ≈ Premiere / Resolve / NLE 时间线**

二者可以：

- 单独使用
- 嵌套使用
- 相互引用

项目核心不是“一个统一作者语法”，而是“多个创作前端 + 一个统一编译内核”。

---

## 2. 输入职责分离规则

### 2.1 TOML 的职责

`MediaTailor.toml` 只允许承载项目级配置，例如：

- 项目名
- 默认入口文件
- 输出目录
- 默认渲染参数
- 资源目录
- profile / cache / build 模式

### 2.2 MTC 的职责

`*.mtc` 只用于画布与图层合成，例如：

- canvas
- layer
- image
- text
- rect
- svg
- row / column / stack / group
- use canvas
- timeline 作为嵌入源

### 2.3 MTT 的职责

`*.mtt` 只用于时间轴编排，例如：

- timeline
- track
- clip
- frames
- audio
- text overlay
- canvas clip
- timeline clip

### 2.4 强制约束

AI agent **不得**：

- 把 timeline/track/clip 结构塞回 MTC
- 把 layer/column/stack 等画布排版心智塞进 MTT 作为主语法
- 把 scene / animation 之类旧 MTL 模型继续作为新主路线扩展
- 把项目配置逻辑混进 MTC 或 MTT
- 设计“双配置源”来同时控制同一项媒体语义
- 让 renderer 直接依赖 TOML 原始结构

---

## 3. 核心架构规则

MediaTailor 必须保持“**双前端 + 统一内核**”分层：

```text
MediaTailor.toml
   └── ProjectConfig

*.mtc
   └── Canvas Parser
       └── CanvasAst
           └── CanvasHir

*.mtt
   └── Timeline Parser
       └── TimelineAst
           └── TimelineHir

CanvasHir + TimelineHir
   └── Unified Composition Model
       └── Layout IR
           └── Timeline IR
               └── Render Plan
                   └── Output
```

### 3.1 必须遵守的架构边界

1. **Parser 只做语法解析**，不直接做复杂渲染业务。
2. **CanvasAst / TimelineAst 只表达源码结构**。
3. **HIR 承担语义展开与归一化**。
4. **Layout 只解决空间问题**。
5. **Timeline 层只解决时间问题**。
6. **Render 只消费已归一化的组合模型**。
7. **Assets 必须统一经由资产管理层处理**。

### 3.2 禁止行为

AI agent **不得**：

- 让 renderer 直接吃 DSL 原始 AST
- 在 parser 中写资源探测逻辑
- 在 layout 中混入 ffmpeg 编码逻辑
- 在 timeline 逻辑中硬编码 TOML 解析
- 在节点实现中绕过统一 Source 抽象直接拼接外部命令

---

## 4. 推荐 crate / module 认知

推荐认知边界如下：

- `mt-cli`：命令行入口
- `mt-project`：项目 TOML 配置解析
- `mt-canvas-dsl`：MTC 解析
- `mt-timeline-dsl`：MTT 解析
- `mt-ast`：CanvasAst / TimelineAst
- `mt-hir`：统一语义层
- `mt-diagnostics`：统一诊断
- `mt-assets`：资源管理与元数据探测
- `mt-layout`：空间布局
- `mt-timeline`：时间编排与展开
- `mt-render`：图像/帧渲染
- `mt-ffmpeg`：媒体探测与编码封装
- `mt-runtime`：总调度器
- `mt-common`：公共基础设施

### 4.1 依赖方向原则

- 前端层不依赖渲染后端
- CanvasAst / TimelineAst 不依赖 HIR
- Renderer 不直接理解 TOML
- Layout 不关心 parser 细节
- Project 配置层不承载 DSL 业务语义

---

## 5. MTC 认知摘要

MTC 是 **空间优先** 的画布 DSL。

### 5.1 主要对象

- `canvas`
- `style`
- `layer`
- `group`
- `row`
- `column`
- `stack`
- `image`
- `text`
- `rect`
- `svg`
- `use canvas`
- `timeline`（作为嵌入源）

### 5.2 设计原则

- 使用缩进式语法，降低样板代码
- 优先支持图层、容器、对齐、相对布局
- 支持长图、海报、封面、拼图
- 支持匿名节点与命名节点并存
- 支持嵌入 timeline 作为动态 layer

### 5.3 典型心智

MTC 回答：

- 画面里有哪些元素
- 元素在哪里
- 元素如何组合

---

## 6. MTT 认知摘要

MTT 是 **时间优先** 的时间轴 DSL。

### 6.1 主要对象

- `timeline`
- `track`
- `clip`
- `frames`
- `image`
- `text`
- `canvas`
- `timeline`

### 6.2 设计原则

- 使用轨道式 + 清单式 + 缩进补充属性语法
- 强调时间顺序可读性
- 适合视频、序列帧、音轨、字幕、嵌套序列
- 允许引用 canvas 作为时间轴片段
- 允许 timeline 嵌套 timeline

### 6.3 典型心智

MTT 回答：

- 哪些素材在什么时候出现
- 哪些轨道叠加
- 哪些片段持续多久

---

## 7. 统一 Source / Composition 约束

AI agent 在设计新功能时，必须优先考虑统一源模型，而不是分别为 MTC / MTT 发明孤立实现。

推荐核心抽象：

```rust
enum SourceKind {
    Image(ImageSource),
    Video(VideoSource),
    Audio(AudioSource),
    Frames(FrameSequenceSource),
    Canvas(CanvasSource),
    Timeline(TimelineSource),
}
```

这意味着：

- timeline 中能引用 canvas
- canvas 中能嵌入 timeline
- canvas 可复用 canvas
- timeline 可嵌套 timeline

AI agent **不得**破坏这层统一抽象。

---

## 8. AI agent 行为约束

### 8.1 设计新功能时

AI agent 必须先判断新需求属于：

- 项目配置层（TOML）
- 画布层（MTC）
- 时间轴层（MTT）
- 统一中间层（HIR / Source / Composition）
- 渲染与编码层

不得跳过分层直接硬塞实现。

### 8.2 修改语法时

AI agent 必须：

- 保持 MTC 偏空间排版
- 保持 MTT 偏时间编排
- 避免重新合并为单一 DSL
- 避免把旧 MTL 作为当前主路线继续扩展

### 8.3 编写示例时

AI agent 应遵循：

- `MediaTailor.toml` 只写项目设置
- `.mtc` 示例展示图层/容器/排版
- `.mtt` 示例展示轨道/片段/时间顺序
- 示例必须优先体现“PS + PR”的双心智模型

### 8.4 编码实现时

AI agent 应优先：

- 先稳定 CanvasAst / TimelineAst
- 再统一 lower 到 HIR / Composition Model
- 再接布局、时间轴展开、渲染与 ffmpeg

不得为了“快速跑通”把 DSL 直接绑死到 renderer。

### 8.5 文档与术语

必须保持术语一致：

- `MediaTailor.toml` = 项目配置
- `MTC` = Canvas DSL
- `MTT` = Timeline DSL
- `CanvasAst / TimelineAst` = 前端语法层
- `HIR` = 统一语义层
- `Layout IR` = 空间布局层
- `Timeline IR` = 时间层
- `Render Plan` = 渲染执行计划

---

## 9. 当前阶段路线约束

### Phase 0

- 读取 `MediaTailor.toml`
- 解析 `.mtc` 与 `.mtt`
- 建立 CanvasAst / TimelineAst
- 统一 lower 到 HIR
- 基础资源加载
- 基础图片导出
- 基础视频导出

### Phase 1

- MTC: row / column / stack / group
- MTT: track / clip / frames / text / audio
- timeline 引用 canvas
- canvas 嵌入 timeline

### Phase 2

- 长图自动高度
- 字幕与样式系统
- 嵌套 timeline
- 更多资源探测与诊断

### Phase 3+

- 转场
- mask / blend / filter
- 组件与模板化复用
- 更强布局与时间特效

AI agent 不应在早期擅自重新引入复杂单一 DSL 设计，除非用户明确要求改路线。

---

## 10. 关联文档

本 skill 与以下文档保持一致：

- `docs/architecture-roadmap.md`
- `docs/mtc-spec-v0.1.md`
- `docs/mtt-spec-v0.1.md`

旧的 `MTL` 设计已废弃，不再作为当前项目主路线参考。当相关文档更新时，本 skill 也应同步更新，避免认知漂移。
