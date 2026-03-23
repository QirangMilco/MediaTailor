# MediaTailor 架构路线设计

## 1. 项目目标

MediaTailor 的目标是构建一个基于 Rust 的声明式媒体编排系统，但不再追求单一 DSL 同时承担图片排版与视频时间轴两种完全不同的创作心智。

新的核心路线是：

- 使用 **TOML** 负责项目级配置
- 使用 **MTC**（MediaTailor Canvas）负责图片/画布/图层合成
- 使用 **MTT**（MediaTailor Timeline）负责视频/轨道/片段编排
- 两种 DSL 可单独使用，也可以彼此嵌套
- 底层统一到同一个 Rust 编译运行时中

MediaTailor 的产品定位应接近：

- **MTC ≈ PS / Figma / 图层排版工具**
- **MTT ≈ PR / Resolve / NLE 时间线工具**

也就是说：

- **Canvas 负责空间组织**
- **Timeline 负责时间组织**

二者是正交关系，而不是谁取代谁。

---

## 2. 输入格式分工

### 2.1 `MediaTailor.toml`

TOML 只负责项目级配置，例如：

- 项目名
- 默认入口文件
- 输出目录
- 默认渲染参数
- 资源目录
- profile / cache / build 模式

示例：

```toml
[project]
name = "demo"
entry = "main.mtt"
version = "0.1.0"

[render]
width = 1920
height = 1080
fps = 30
background = "#000000"

[assets]
dirs = ["assets", "shared-assets"]

[output]
dir = "dist"
video = "output.mp4"
image = "cover.png"
```

### 2.2 `*.mtc`

MTC 是画布/图层 DSL，负责：

- 海报
- 封面
- 长图
- 拼图
- 文字与图层排版
- 嵌套 canvas
- 在画布中嵌入 timeline 作为动态图层

### 2.3 `*.mtt`

MTT 是时间轴 DSL，负责：

- 视频剪辑
- 多轨道编排
- 序列帧转视频
- 字幕与音轨
- 嵌套 timeline
- 引用 canvas 作为片头/片尾/静态卡片/动态图层

---

## 3. 总体架构

MediaTailor 必须采用“**双前端 + 统一内核**”的架构，而不是单一 DSL。

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
                   └── PNG / Frame Sequence / MP4 / Audio Mix
```

关键点：

1. MTC 与 MTT 是两个 authoring surface。
2. 二者共享底层资源系统、诊断系统、渲染系统和导出系统。
3. 二者都 lower 到统一的组合模型（Composition Model）。
4. 嵌套能力通过统一的 Source 抽象实现。

---

## 4. 核心设计原则

1. **TOML 只负责项目配置**。
2. **MTC 只负责空间合成，不承担主时间轴剪辑语义**。
3. **MTT 只负责时间编排，不承担完整图层排版心智**。
4. **Canvas 与 Timeline 必须可以彼此嵌套**。
5. **所有执行都必须经过统一中间层**，不得让 renderer 直接依赖 DSL 原始结构。
6. **诊断能力是一等公民**，错误应尽量绑定源码位置。

---

## 5. Rust workspace 结构建议

```text
MediaTailor/
├── Cargo.toml
├── crates/
│   ├── mt-cli/
│   ├── mt-project/
│   ├── mt-canvas-dsl/
│   ├── mt-timeline-dsl/
│   ├── mt-ast/
│   ├── mt-hir/
│   ├── mt-diagnostics/
│   ├── mt-assets/
│   ├── mt-layout/
│   ├── mt-timeline/
│   ├── mt-render/
│   ├── mt-ffmpeg/
│   ├── mt-runtime/
│   └── mt-common/
├── examples/
│   ├── poster/
│   │   ├── MediaTailor.toml
│   │   └── poster.mtc
│   ├── trailer/
│   │   ├── MediaTailor.toml
│   │   └── trailer.mtt
│   └── hybrid/
│       ├── MediaTailor.toml
│       ├── cover.mtc
│       └── main.mtt
├── docs/
└── tests/
```

前期若想降低复杂度，也可先保留：

- `mt-cli`
- `mt-project`
- `mt-canvas-dsl`
- `mt-timeline-dsl`
- `mt-ast`
- `mt-hir`
- `mt-assets`
- `mt-layout`
- `mt-render`
- `mt-runtime`

在基础图片导出与时间轴导出跑通后，再拆 `mt-ffmpeg` 和更细的 timeline crate。

---

## 6. crate 职责建议

### 6.1 `mt-cli`

负责：

- 解析 CLI 命令
- 加载项目
- 调用 runtime
- 输出诊断

建议命令：

```bash
mt check
mt build
mt render
mt render canvas poster
mt render timeline trailer
mt dump canvas-ast
mt dump timeline-ast
mt dump hir
```

### 6.2 `mt-project`

负责解析 `MediaTailor.toml`，构建 `ProjectConfig` 与 `ProjectContext`。

### 6.3 `mt-canvas-dsl`

负责 `.mtc` 前端：

- lexer / parser
- 缩进式语法解析
- Canvas AST
- Canvas 语法诊断

### 6.4 `mt-timeline-dsl`

负责 `.mtt` 前端：

- lexer / parser
- 轨道式/清单式语法解析
- Timeline AST
- Timeline 语法诊断

### 6.5 `mt-ast`

定义公共语法层抽象，或分别承载：

- `CanvasAst`
- `TimelineAst`

### 6.6 `mt-hir`

负责：

- 语义分析
- 样式展开
- 资源绑定
- 归一化尺寸/颜色/时间
- 将 Canvas / Timeline lower 到统一组合模型

### 6.7 `mt-assets`

统一管理：

- 图片
- 视频
- 音频
- 字体
- SVG
- canvas / timeline 引用
- 元数据探测

### 6.8 `mt-layout`

负责空间布局：

- canvas 画布尺寸
- row / column / stack / group
- 对齐与相对定位
- 嵌套 canvas / timeline 的空间放置

### 6.9 `mt-timeline`

负责时间组织：

- track
- clip
- frames
- start/end
- 时间范围检查
- 嵌套 timeline / canvas 的展开

### 6.10 `mt-render`

负责：

- 静态图片渲染
- 单帧渲染
- 帧序列渲染
- 可视组合节点渲染

### 6.11 `mt-ffmpeg`

负责：

- ffprobe 元数据探测
- 帧序列编码
- 音视频混合
- 最终视频输出

### 6.12 `mt-runtime`

负责串联整个流程：

```text
project -> parse canvas/timeline -> hir -> assets -> layout -> timeline -> render -> encode
```

---

## 7. 统一中间层建议

关键在于定义统一的可组合源类型：

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

这使得：

- timeline 中可以引用 canvas
- canvas 中可以嵌入 timeline
- canvas 可以复用 canvas
- timeline 可以嵌套 timeline

建议再进一步抽象统一组合节点：

```rust
enum CompositionNode {
    Visual(VisualNode),
    Audio(AudioNode),
    Container(ContainerNode),
    Sequence(SequenceNode),
}
```

---

## 8. 互相嵌套模型

### 8.1 Timeline 引用 Canvas

适合：

- 片头卡
- 片尾卡
- 标题页
- 静态海报转视频片段

示例：

```text
timeline main
  track video
    0s..2s canvas "cover.mtc"
    2s..8s clip "demo.mp4"
```

### 8.2 Canvas 引用 Timeline

适合：

- 画中画
- 设备屏幕 mockup
- 动态窗口嵌入

示例：

```text
canvas mockup
  size 1920x1080

  timeline "screen-demo.mtt"
    x 240
    y 160
    width 1440
    height 810
```

### 8.3 Canvas 引用 Canvas

适合模板拼装：

```text
canvas page
  column gap 32
    use canvas "header.mtc"
    use canvas "body.mtc"
    use canvas "footer.mtc"
```

### 8.4 Timeline 引用 Timeline

适合序列复用：

```text
timeline master
  track video
    0s..8s timeline "intro.mtt"
    8s..18s timeline "feature.mtt"
```

---

## 9. 语法层建议

### 9.1 MTC

- 缩进式
- 偏图层/容器表达
- 低样板代码
- 强空间心智

### 9.2 MTT

- 轨道式 + 清单式 + 缩进补充属性
- 强时间顺序可读性
- 适合序列帧与视频拼接

### 9.3 不再继续推进旧 MTL

旧的单一 `.mtl` 设计已经不再符合项目方向，应正式废弃，不再作为主语法路线维护。

---

## 10. 示例项目结构

```text
hybrid-demo/
├── MediaTailor.toml
├── cover.mtc
├── main.mtt
├── mockup.mtc
├── assets/
│   ├── bg.jpg
│   ├── hero.png
│   ├── demo.mp4
│   ├── bgm.mp3
│   └── frames/
└── dist/
```

### `MediaTailor.toml`

```toml
[project]
name = "hybrid-demo"
entry = "main.mtt"

[render]
width = 1920
height = 1080
fps = 30

[assets]
dirs = ["assets"]

[output]
dir = "dist"
```

### `cover.mtc`

```text
canvas cover
  size 1920x1080
  background #101418

  image "bg.jpg"
    fit cover

  text "MediaTailor"
    font 88
    color white
    align center
```

### `main.mtt`

```text
timeline main
  size 1920x1080
  fps 30

  track video
    0s..3s canvas "cover.mtc"
    3s..10s clip "demo.mp4"

  track audio
    0s..10s clip "bgm.mp3"
```

---

## 11. 实施路线图

### Phase 0

先打通双 DSL 的最小闭环：

1. `mt-project` 读取 `MediaTailor.toml`
2. `mt-canvas-dsl` 解析 `.mtc`
3. `mt-timeline-dsl` 解析 `.mtt`
4. `mt-hir` lower 到统一组合模型
5. `mt-assets` 完成图片/视频/音频探测
6. `mt-layout` 打通 canvas 静态布局
7. `mt-render` 输出单张 PNG
8. `mt-timeline` + `mt-ffmpeg` 输出简单视频

### Phase 1

- canvas: row / column / stack / group
- timeline: track / clip / frames / text / audio
- timeline 引用 canvas
- canvas 引用 timeline

### Phase 2

- 样式系统稳定化
- 长图自动高度
- 序列帧导入
- 字幕片段
- 嵌套 timeline

### Phase 3

- 转场
- 特效
- mask / blend
- 更多诊断
- 组件与模板化复用

---

## 12. 最终建议

MediaTailor 的正确长期路线应当是：

> **TOML 管项目，MTC 管空间画布，MTT 管时间编排，二者通过统一 Source/Composition 模型互相嵌套，并共享同一 Rust 编译运行时。**

这条路线比单一 DSL 更贴近真实用户心智，也更符合“图片像 PS、视频像 PR”的产品目标。
