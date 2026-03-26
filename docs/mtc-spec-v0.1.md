# MTC 语法草案 v0.1

> 注：本文仍保留 v0.1 草案名称，但以下内容已尽量与当前 Rust 原型实现保持一致；未实现能力会明确标注为“规划中”或“未实现”。

## 1. 定位

MTC（MediaTailor Canvas）是 MediaTailor 的画布/图层 DSL，面向静态图像、长图、封面、海报、拼图和图层合成场景。它的目标更接近 PS / Figma / 版式工具，而不是时间轴剪辑工具。

MTC 负责回答：

- 画布有多大
- 画面里有哪些图层
- 它们放在哪里
- 它们如何分组、对齐、堆叠与复用

MTC 采用 **缩进式语法**，尽量减少花括号与样板代码，强调快速拼接和低输入负担。

---

## 2. 文件与基本约定

- 扩展名：`.mtc`
- 编码：UTF-8
- 语法风格：缩进式
- 注释：`# ...`
- 缩进：建议 2 空格，禁止混用 tab 与空格
- 一行一个语句，子层级通过缩进表达

---

## 3. 顶层结构

当前原型中，一个 `.mtc` 文件由：

1. **0 个或多个顶层 `style <name>` 块**
2. **1 个 `canvas <name>` 块**

```text
style heading
  font 72
  color white
  letter-spacing 2

canvas poster
  size 1080x1920
  background #101418
```

### 顶层元素

- `style`
- `canvas`

### 当前实现约束

- 顶层 `style` 必须出现在 `canvas` 之前
- 当前仅支持 **单个 canvas**
- `import` 属于规划中，当前 Rust 原型 **尚未实现**

---

## 4. 核心语法形态

## 4.1 import（规划中）

```text
import "styles/base.mtc"
```

- 设计上只允许出现在顶层
- 路径应相对当前文件解析
- 应检测循环导入
- **当前 Rust 原型未实现，使用会报错**

## 4.2 style

```text
style heading
  font 72
  color white
  weight 700
  letter-spacing 2
  align center
```

- 用于定义可复用文本样式
- 当前只允许文本样式属性，不允许子节点
- 节点可通过 `style heading` 应用
- 同名时，节点本地属性覆盖 style 定义

## 4.3 canvas

```text
canvas poster
  size 1080x1920
  background #101418
```

一个文件可包含多个 canvas，项目入口由 `MediaTailor.toml` 指定。

---

## 5. 画布级属性

当前原型已实现：

- `size 1080x1920`
- `width 1080`
- `height 1920`
- `background #101418`

尚未实现但保留为后续方向：

- `padding 40`
- `dpi 144`
- `height auto`

其中：

- `size` 与 `width/height` 二选一表达尺寸
- 当前原型要求最终必须得到确定的 `width` 与 `height`

示例：

```text
canvas poster
  width 1200
  height 1600
  background white
```

---

## 6. 图层与容器

MTC 当前原型的主体由节点和布局容器组成。

### 6.1 当前已实现容器

- `row`
- `column`

示例：

```text
column
  gap 24
  padding 20 24

  image "1.jpg"
    width 320
  image "2.jpg"
    width 320
```

语义：

- `row`：横向顺序布局
- `column`：纵向顺序布局

### 6.2 当前未实现但保留方向

- `layer`
- `group`
- `stack`
- 命名节点与关系定位

---

## 7. 节点类型

当前 Rust 原型已实现以下节点：

- `image`
- `text`
- `rect`
- `row`
- `column`

### 7.1 image

```text
image "hero.png"
  x 120
  y 160
  width 720
  height 480
  fit contain
  opacity 0.92
```

### 7.2 text

```text
text "夏日旅行" style heading
  width 560
  align center
  letter-spacing 2
```

说明：

- `text` 支持单行引用：`text "内容" style body`
- 也支持多行写法：

```text
text "正文内容"
  style body
  width 640
```

### 7.3 rect

```text
rect
  x 0
  y 0
  width 240
  height 4
  fill #ffffff22
```

### 7.4 未实现节点（规划中）

- `svg`
- 嵌套 `canvas`
- `timeline`

---

## 8. 常用布局属性

当前原型已实现：

- `x 120`
- `y 160`
- `width 720`
- `height 480`
- `padding 24`
- `gap 16`

当前未实现但仍可作为后续方向：

- `min-width`
- `max-width`
- 节点级关系定位（如 `below` / `right-of`）
- 通用 `align top-center`
- `margin-top`
- `z`

设计原则：

- 当前版本优先保证绝对坐标 + 轻量容器布局可用
- 关系定位系统留待后续版本扩展

---

## 9. 常用视觉属性

当前原型已实现：

- `opacity 0.8`（image）
- `fill #ffffff`（rect）

文本属性当前已实现：

- `font 72`
- `font-family "Times New Roman"`
- `font-path "fonts/windows/times.ttf"`
- `language zh`
- `weight 700`
- `line-height 1.4`
- `letter-spacing 2`
- `color white`
- `align left|center|right`

其中：

- `align` 当前仅用于 **text 节点在 `width` 指定的文本框内** 做左/中/右对齐
- `letter-spacing` 当前通过逐字符附加间距实现，并已参与 **测量、换行、渲染**

尚未实现但保留方向：

- `rotate`
- `scale`
- `stroke`
- `stroke-width`
- `radius`
- `shadow`
- `blend`

MediaTailor 当前内置识别一组常见字体族名（即使未在 `MediaTailor.toml` 的 `fonts.families` 中显式声明，也会尝试解析）：

- `Times New Roman`
- `宋体` / `SimSun`
- `黑体` / `SimHei`
- `楷体` / `KaiTi` / `SimKai`
- `微软雅黑` / `Microsoft YaHei`

默认约定建议：

- 英文使用 `Times New Roman`
- 中文使用 `宋体`

推荐将对应字体文件放到项目目录（以当前 `.mtc` 所在目录为根）下的 `fonts/windows/` 中，例如：

- `fonts/windows/times.ttf`
- `fonts/windows/simsun.ttc`
- `fonts/windows/simhei.ttf`
- `fonts/windows/simkai.ttf`
- `fonts/windows/msyh.ttc`

---

## 10. 典型写法

### 10.1 海报

```toml
# MediaTailor.toml
[fonts.defaults]
default = "Times New Roman"
zh = "宋体"
en = "Times New Roman"

[styles.heading]
font-family = "微软雅黑"
font = 54
color = "#f8fafc"
weight = 700
letter-spacing = 3
align = "center"
```

```text
style heading
  font 58
  color #f8fafc
  letter-spacing 4
  align center

canvas poster
  size 1200x1600
  background #101418

  image "bg.jpg"
    x 110
    y 110
    width 980
    height 980
    fit cover

  text "MEDIA TAILOR" style heading
    x 118
    y 1080
    width 640

  text "快速把素材拼成封面、海报与长图"
    x 136
    y 1172
    width 560
    style body
    align center
```

### 10.2 长图拼接

```text
canvas longform
  width 1200
  height auto
  background white
  padding 40

  column gap 24
    image "1.jpg" width 100%
    image "2.jpg" width 100%
    image "3.jpg" width 100%
    text "完" align center
```

### 10.3 模板拼装

```text
canvas page
  column gap 32
    use canvas "header.mtc"
    use canvas "features.mtc"
    use canvas "footer.mtc"
```

---

## 11. 语义约束

1. 当前原型仅支持单 canvas 文件。
2. 顶层 `style` 必须位于 `canvas` 之前。
3. `row/column` 下的直接子节点按声明顺序参与布局。
4. 文本样式优先级固定为：系统默认 < `MediaTailor.toml` 的 `[styles.*]` < 当前 `.mtc` 顶层 `style` < 节点本地属性。
5. `align` 仅影响 text 在给定 `width` 范围内的绘制起点，不影响容器主轴布局。
6. 未找到的样式名会直接报错。

---

## 12. 实现建议

MTC 前端建议单独建模为：

- `CanvasAst`
- `CanvasHir`
- `LayoutScene`

并在统一运行时中将其 lower 为共享的 `CompositionNode` / `SourceKind::Canvas`。

---

## 13. v0.2 以后可扩展方向

- mask
- blend
- filter
- guide / anchor system
- reusable component params
- 约束布局
- 响应式画布
