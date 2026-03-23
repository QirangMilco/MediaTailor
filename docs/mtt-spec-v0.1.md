# MTT 语法草案 v0.1

## 1. 定位

MTT（MediaTailor Timeline）是 MediaTailor 的时间轴 DSL，面向视频剪辑、序列帧拼接、字幕、音轨、嵌套序列与最终视频输出场景。它的目标更接近 PR / Resolve / NLE 时间线，而不是图层排版工具。

MTT 负责回答：

- 哪些素材在什么时候出现
- 素材位于哪条轨道
- 轨道如何叠加
- 哪些片段是视频、音频、序列帧、canvas 或嵌套 timeline

MTT 采用 **轨道式 + 清单式 + 缩进式** 语法，强调时间顺序可读性和快速拼接。

---

## 2. 文件与基本约定

- 扩展名：`.mtt`
- 编码：UTF-8
- 注释：`# ...`
- 缩进：建议 2 空格
- 一行一个片段声明，子属性可继续缩进

---

## 3. 顶层结构

一个 `.mtt` 文件通常由：

- `import`
- `timeline`

构成，可选支持共享样式或字幕风格声明。

示例：

```text
import "common/title-cards.mtc"

timeline trailer
  size 1920x1080
  fps 30

  track video
    0s..3s clip "intro.mp4"
```

---

## 4. timeline 声明

```text
timeline trailer
  size 1920x1080
  fps 30
  duration 15s
```

推荐首版属性：

- `size 1920x1080`
- `width 1920`
- `height 1080`
- `fps 30`
- `duration 15s`
- `background black`

其中：

- `duration` 可显式指定，也可由最长轨道片段推导
- `size` 与 `width/height` 不应重复表达同一语义

---

## 5. 轨道模型

时间线通过 `track` 组织。

```text
track video
  0s..3s clip "intro.mp4"
  3s..8s clip "main.mp4"

track audio
  0s..8s clip "bgm.mp3"
```

推荐首版轨道类型：

- `track video`
- `track audio`
- `track text`
- `track overlay`

说明：

- `video`：主视频画面或可视素材
- `audio`：音频素材
- `text`：字幕/标题/文字片段
- `overlay`：额外覆盖层，可承载 canvas 或特效素材

---

## 6. 片段类型

MTT 的核心单位是片段（clip item）。

首版建议支持：

- `clip "intro.mp4"`
- `frames "frames/*.png" fps 30`
- `image "cover.png"`
- `text "Hello"`
- `canvas "cover.mtc"`
- `timeline "intro-seq.mtt"`

---

## 7. 片段语法

### 7.1 视频片段

```text
0s..3s clip "intro.mp4"
```

### 7.2 序列帧片段

```text
3s..8s frames "shots/*.png" fps 30
```

或：

```text
3s..8s frames "shots/*.png" each 1f
```

### 7.3 图片片段

```text
8s..10s image "ending.png"
```

### 7.4 文本片段

```text
1s..2.5s text "Chapter 1" bottom-center
```

### 7.5 canvas 片段

```text
0s..2s canvas "cover.mtc"
```

### 7.6 timeline 片段

```text
10s..20s timeline "feature-seq.mtt"
```

---

## 8. 片段扩展属性

当一行不足以表达时，可向下缩进补充属性：

```text
0s..3s clip "intro.mp4"
  trim 0.5s..3.5s
  fit cover
  volume 0.8
```

或：

```text
7s..9s text "The End"
  align center
  valign bottom
  margin-bottom 120
  font 64
  color white
```

推荐首版支持的片段属性：

- `trim 1s..5s`
- `fit contain|cover|fill`
- `volume 0.8`
- `mute true`
- `speed 1.25`
- `align center`
- `valign bottom`
- `margin-bottom 80`
- `transition fade 300ms`（保留）

---

## 9. 典型写法

### 9.1 简单剪辑

```text
timeline trailer
  size 1920x1080
  fps 30
  duration 15s

  track video
    0s..3s   clip "intro.mp4"
    3s..8s   clip "main.mp4"
    8s..12s  clip "ending.mp4"

  track audio
    0s..15s  clip "bgm.mp3"

  track text
    1s..2.5s text "Chapter 1" bottom-center
    12s..14s text "The End" center
```

### 9.2 序列帧转视频

```text
timeline shots
  size 1920x1080
  fps 30

  track video
    0s..6s frames "frames/*.png" fps 30

  track audio
    0s..6s clip "bgm.mp3"
```

### 9.3 嵌套 canvas

```text
timeline ad
  size 1920x1080
  fps 30

  track video
    0s..3s  canvas "cover.mtc"
    3s..8s  clip "product.mp4"
    8s..10s canvas "ending-card.mtc"
```

### 9.4 嵌套 timeline

```text
timeline master
  size 1920x1080
  fps 30

  track video
    0s..10s timeline "intro-seq.mtt"
    10s..20s timeline "feature-seq.mtt"
```

---

## 10. 与 Canvas 的关系

MTT 与 MTC 是正交关系：

- MTC 负责空间合成
- MTT 负责时间编排

两者可互相嵌套：

### Timeline 引用 Canvas

```text
0s..2s canvas "cover.mtc"
```

语义：把某个 canvas 渲染为静态或可时长扩展的可视片段。

### Timeline 引用 Timeline

```text
10s..20s timeline "feature-seq.mtt"
```

语义：嵌套一个已定义的序列作为 clip。

---

## 11. 时间与轨道约束

1. 时间范围采用 `start..end` 形式。
2. 必须满足 `start < end`。
3. 同一轨道内，首版建议默认不允许重叠片段，除非后续引入 lane 或 layer 语义。
4. 不同轨道可自然叠加。
5. 未声明 `duration` 时，可根据最晚结束时间自动推导。
6. `frames` 片段必须能推导出总帧数与时长。

---

## 12. 文本片段建议

文本片段是视频中的常见 overlay，建议保留一行式简写：

```text
8.5s..9.5s text "The End" center
```

同时允许扩展为多行：

```text
8.5s..9.5s text "The End"
  align center
  valign bottom
  font 72
  color white
```

---

## 13. 实现建议

MTT 前端建议单独建模为：

- `TimelineAst`
- `TimelineHir`
- `TimelineSequence`

并通过统一 `SourceKind` 支持：

- `Image`
- `Video`
- `Audio`
- `Frames`
- `Canvas`
- `Timeline`

这样可无缝实现 `canvas in timeline` 与 `timeline in canvas`。

---

## 14. v0.2 以后可扩展方向

- transition
- subtitle styles
- speed ramp
- reverse
- nested effects
- lane / multi-layer track
- markers
- export region
