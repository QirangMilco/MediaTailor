# MTC 语法草案 v0.1

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

一个 `.mtc` 文件通常由一个或多个 `canvas` 声明构成，可选包含 `import` 与 `style`：

```text
import "styles/base.mtc"

style heading
  font 72
  color white

canvas poster
  size 1080x1920
  background #101418
```

### 顶层元素

- `import`
- `style`
- `canvas`

---

## 4. 核心语法形态

## 4.1 import

```text
import "styles/base.mtc"
```

- 只能出现在顶层
- 路径相对当前文件解析
- 必须检测循环导入

## 4.2 style

```text
style heading
  font 72
  color white
  weight 700
```

- 用于定义可复用样式
- 只允许样式属性，不允许子节点
- 节点可通过 `style heading` 应用

## 4.3 canvas

```text
canvas poster
  size 1080x1920
  background #101418
```

一个文件可包含多个 canvas，项目入口由 `MediaTailor.toml` 指定。

---

## 5. 画布级属性

推荐首版支持：

- `size 1080x1920`
- `width 1080`
- `height 1920`
- `background #101418`
- `padding 40`
- `dpi 144`（可选）

其中：

- `size` 与 `width/height` 不应重复表达同一语义
- 允许 `height auto` 以支持长图/内容驱动高度

示例：

```text
canvas longform
  width 1200
  height auto
  background white
  padding 40
```

---

## 6. 图层与容器

MTC 的主体由图层和容器组成。

### 6.1 layer

```text
layer hero
  image "hero.png"
  width 80%
  align center
```

`layer` 是通用图层包装器，适合需要命名、复用、相对定位的场景。

### 6.2 直接节点

也允许不显式写 `layer`：

```text
image "hero.png"
  width 80%
  align center
```

对于无需引用的节点，推荐直接节点写法。

### 6.3 容器节点

推荐首版支持：

- `group`
- `row`
- `column`
- `stack`

示例：

```text
column gallery
  gap 24

  image "1.jpg" width 100%
  image "2.jpg" width 100%
  image "3.jpg" width 100%
```

语义：

- `row`：横向布局
- `column`：纵向布局
- `stack`：重叠布局
- `group`：逻辑分组，不强制布局策略

---

## 7. 节点类型

v0.1 推荐支持以下节点：

- `image`
- `text`
- `rect`
- `svg`
- `canvas`（嵌套其他 canvas）
- `timeline`（嵌入 timeline 作为动态源）

### 7.1 image

```text
image "hero.png"
  width 80%
  fit contain
  align center
```

### 7.2 text

```text
text "夏日旅行"
  style heading
  align top-center
  margin-top 160
```

### 7.3 rect

```text
rect
  width 100%
  height 4
  fill #ffffff22
```

### 7.4 svg

```text
svg "logo.svg"
  width 180
  align top-right
```

### 7.5 嵌套 canvas

```text
canvas card-grid
  column gap 32
    use canvas "header.mtc"
    use canvas "body.mtc"
    use canvas "footer.mtc"
```

### 7.6 嵌套 timeline

```text
canvas mockup
  size 1920x1080

  timeline "screen-demo.mtt"
    x 240
    y 160
    width 1440
    height 810
```

---

## 8. 常用布局属性

推荐首版支持：

- `x 120`
- `y 160`
- `width 720`
- `height 480`
- `min-width 200`
- `max-width 80%`
- `align center`
- `align top-center`
- `below title 24`
- `right-of hero 20`
- `center-in parent`
- `padding 24`
- `gap 16`
- `margin-top 40`
- `z 10`

设计原则：

- MTC 优先支持 **关系定位** 与 **容器布局**
- 绝对坐标作为基础能力保留，但不应成为唯一主路径

---

## 9. 常用视觉属性

推荐首版支持：

- `opacity 0.8`
- `rotate 12deg`（或 `rotate 12`）
- `scale 1.2`
- `fill #ffffff`
- `stroke #000000`
- `stroke-width 2`
- `radius 24`
- `shadow sm`（可选）
- `blend multiply`（保留）

文本属性建议：

- `font 72`
- `font-family "Inter"`
- `weight 700`
- `line-height 1.4`
- `letter-spacing 2`
- `color white`
- `align center`

---

## 10. 典型写法

### 10.1 海报

```text
canvas poster
  size 1080x1920
  background #101418

  image "bg.jpg"
    fit cover

  text "夏日旅行"
    font 96
    color white
    align top-center
    margin-top 160

  text "Summer Travel 2026"
    font 36
    color #d0d6dc
    below "夏日旅行" 24

  image "hero.png"
    width 82%
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

1. 一个节点可匿名，也可命名。
2. 只有命名节点可被 `below/right-of/anchor` 等关系显式引用。
3. `row/column/stack` 下的直接子节点参与对应布局。
4. `height auto` 时，canvas 高度由内容计算得到。
5. `timeline` 嵌入 canvas 时，视为动态 layer source。
6. `use canvas` 应引用另一个可解析的 canvas 定义。

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
