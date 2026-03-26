# quick-poster

最小 MTC 图片原型示例。

## 文件

- `demo.mtc`：画布描述
- 仓库根 `MediaTailor.toml`：项目级字体配置 + 项目级 `[styles.*]` 文本样式
- 仓库根 `fonts/windows/`：推荐放置 Windows 常见字体文件的位置
- `assets/hero.jpg`：主视觉图片
- `assets/badge.png`：角标素材

> 注意：`quick-poster` 不再维护单独的 `MediaTailor.toml`，示例会直接继承仓库根配置。

## 当前支持

- `rect`
- `image`
- `text`
- `row`
- `column`
- 画布背景色
- 图片 `fit`
- 文本颜色 / 字号 / 字体粗细（粗略映射）
- 文本 `letter-spacing` 与 `align(left|center|right)`
- 容器 `gap` 与子节点顺序布局
- 项目配置字体映射、语言默认字体、每个 `text` 的 `font-family` / `font-path`
- 项目级 `[styles.*]` 文本样式、文件内顶层 `style xxx`、`text "..." style body` 单行引用
- MediaTailor 内置字体族名解析：`Times New Roman`、`宋体`、`黑体`、`楷体`、`微软雅黑`

## 运行

在仓库根目录执行：

```bash
cargo run -p mt-cli -- check examples/quick-poster/demo.mtc
cargo run -p mt-cli -- render examples/quick-poster/demo.mtc examples/quick-poster/output.png
```

生成结果：

- `examples/quick-poster/output.png`

## 字体约定

当前示例默认约定：

- 英文默认字体：`Times New Roman`
- 中文默认字体：`宋体`

如果你希望在 Windows / Linux 上都得到更稳定的结果，建议把常见 Windows 字体文件放到仓库根目录：

- `fonts/windows/times.ttf`
- `fonts/windows/simsun.ttc`
- `fonts/windows/simhei.ttf`
- `fonts/windows/simkai.ttf`
- `fonts/windows/msyh.ttc`

其中族名与推荐文件对应为：

- `Times New Roman` → `times.ttf`
- `宋体` / `SimSun` → `simsun.ttc`
- `黑体` / `SimHei` → `simhei.ttf`
- `楷体` / `KaiTi` / `SimKai` → `simkai.ttf`
- `微软雅黑` / `Microsoft YaHei` → `msyh.ttc`

## 说明

- `mt-cli` 现在会从输入 `.mtc` 所在目录开始，逐级向上查找最近的 `MediaTailor.toml`
- `row` / `column` 当前支持 `x`、`y`、`gap`、`padding`
- `padding` 支持 1 / 2 / 4 个整数值，语义分别类似 CSS 的全量、上下左右、上右下左
- 文本样式优先级：系统默认 < 项目级 `[styles.*]` < 当前 `.mtc` 顶层 `style xxx` < 节点本地属性
- `text` 同时支持多行写法 `style body` 与单行写法 `text "内容" style body`
- `align` 当前仅作用于 `text width ...` 形成的文本框，支持 `left` / `center` / `right`
- `letter-spacing` 当前以逐字符额外间距实现，并参与测量与自动换行
- 字体解析优先级：`font-path` > `font-family` > `language` 对应默认字体 > `default` 默认字体 > 内置字体族 fallback > 系统 sans fallback

- 当 `MediaTailor.toml` 未显式声明 family 路径时，MediaTailor 会尝试解析上述内置字体族名，并优先从项目根 `fonts/windows/` 目录加载
- 这是快速原型方案，后续可再扩展对齐、容器背景、显式尺寸与更精细排版
