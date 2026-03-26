use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

#[derive(Debug, Clone)]
pub struct CanvasDoc {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub background: Color,
    pub nodes: Vec<Node>,
    pub text_styles: HashMap<String, TextStyle>,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum Node {
    Image(ImageNode),
    Rect(RectNode),
    Text(TextNode),
    Row(LayoutNode),
    Column(LayoutNode),
}

#[derive(Debug, Clone)]
pub struct ImageNode {
    pub path: PathBuf,
    pub x: i64,
    pub y: i64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fit: FitMode,
    pub opacity: f32,
}

#[derive(Debug, Clone)]
pub struct RectNode {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub fill: Color,
}

#[derive(Debug, Clone)]
pub struct TextNode {
    pub value: String,
    pub x: i32,
    pub y: i32,
    pub width: Option<u32>,
    pub style_name: Option<String>,
    pub font: Option<f32>,
    pub color: Option<Color>,
    pub line_height: Option<f32>,
    pub weight: Option<u32>,
    pub letter_spacing: Option<f32>,
    pub align: Option<TextAlign>,
    pub font_family: Option<String>,
    pub font_path: Option<PathBuf>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TextStyle {
    pub font: Option<f32>,
    pub color: Option<Color>,
    pub line_height: Option<f32>,
    pub weight: Option<u32>,
    pub letter_spacing: Option<f32>,
    pub align: Option<TextAlign>,
    pub font_family: Option<String>,
    pub font_path: Option<PathBuf>,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedTextStyle {
    pub font: f32,
    pub color: Color,
    pub line_height: f32,
    pub weight: u32,
    pub letter_spacing: f32,
    pub align: TextAlign,
    pub font_family: Option<String>,
    pub font_path: Option<PathBuf>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl TextAlign {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "left" | "start" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" | "end" => Ok(Self::Right),
            other => bail!("unsupported text align: {other}"),
        }
    }
}

impl TextStyle {
    pub fn merge_from(&mut self, other: &TextStyle) {
        if let Some(font) = other.font {
            self.font = Some(font);
        }
        if let Some(color) = other.color {
            self.color = Some(color);
        }
        if let Some(line_height) = other.line_height {
            self.line_height = Some(line_height);
        }
        if let Some(weight) = other.weight {
            self.weight = Some(weight);
        }
        if let Some(letter_spacing) = other.letter_spacing {
            self.letter_spacing = Some(letter_spacing);
        }
        if let Some(align) = other.align {
            self.align = Some(align);
        }
        if let Some(font_family) = &other.font_family {
            self.font_family = Some(font_family.clone());
        }
        if let Some(font_path) = &other.font_path {
            self.font_path = Some(font_path.clone());
        }
        if let Some(language) = &other.language {
            self.language = Some(language.clone());
        }
    }

    pub fn resolve(&self) -> ResolvedTextStyle {
        ResolvedTextStyle {
            font: self.font.unwrap_or(32.0).max(1.0),
            color: self.color.unwrap_or(Color::WHITE),
            line_height: self.line_height.unwrap_or(1.25).max(0.5),
            weight: self.weight.unwrap_or(400),
            letter_spacing: self.letter_spacing.unwrap_or(0.0),
            align: self.align.unwrap_or(TextAlign::Left),
            font_family: self.font_family.clone(),
            font_path: self.font_path.clone(),
            language: self.language.clone(),
        }
    }
}

impl TextNode {
    pub fn inline_style(&self) -> TextStyle {
        TextStyle {
            font: self.font,
            color: self.color,
            line_height: self.line_height,
            weight: self.weight,
            letter_spacing: self.letter_spacing,
            align: self.align,
            font_family: self.font_family.clone(),
            font_path: self.font_path.clone(),
            language: self.language.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub x: i32,
    pub y: i32,
    pub gap: i32,
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Copy)]
pub enum FitMode {
    None,
    Fill,
    Contain,
    Cover,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub fn parse(input: &str) -> Result<Self> {
        let value = input.trim();
        match value {
            "white" => Ok(Self::WHITE),
            "black" => Ok(Self {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            _ if value.starts_with('#') => Self::parse_hex(value),
            _ => bail!("unsupported color: {value}"),
        }
    }

    fn parse_hex(value: &str) -> Result<Self> {
        let hex = value.trim_start_matches('#');
        let parse = |slice: &str| {
            u8::from_str_radix(slice, 16).map_err(|_| anyhow!("invalid hex color: {value}"))
        };

        match hex.len() {
            6 => Ok(Self {
                r: parse(&hex[0..2])?,
                g: parse(&hex[2..4])?,
                b: parse(&hex[4..6])?,
                a: 255,
            }),
            8 => Ok(Self {
                r: parse(&hex[0..2])?,
                g: parse(&hex[2..4])?,
                b: parse(&hex[4..6])?,
                a: parse(&hex[6..8])?,
            }),
            _ => bail!("invalid hex color length: {value}"),
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedLine {
    indent: usize,
    content: String,
    line_no: usize,
}

pub fn parse_mtc_file(path: &Path) -> Result<CanvasDoc> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read MTC file: {}", path.display()))?;
    parse_mtc(&source, path)
}

pub fn parse_mtc(source: &str, path: &Path) -> Result<CanvasDoc> {
    let lines = collect_lines(source)?;
    if lines.is_empty() {
        bail!("MTC file is empty");
    }

    let mut text_styles = HashMap::new();
    let mut index = 0;

    while index < lines.len() {
        let line = &lines[index];
        if line.indent != 0 || !line.content.starts_with("style ") {
            break;
        }
        parse_style_definition(&lines, &mut index, &mut text_styles)?;
    }

    let canvas_line = lines
        .get(index)
        .ok_or_else(|| anyhow!("MTC file is empty"))?;

    let name = canvas_line
        .content
        .strip_prefix("canvas ")
        .ok_or_else(|| anyhow!("line {}: first statement must be `canvas <name>`", canvas_line.line_no))?
        .trim()
        .to_string();

    if name.is_empty() {
        bail!("line {}: canvas name cannot be empty", canvas_line.line_no);
    }

    let mut width = None;
    let mut height = None;
    let mut background = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    let mut nodes = Vec::new();
    index += 1;

    while index < lines.len() {
        let line = &lines[index];
        if line.indent != 2 {
            bail!("line {}: expected 2-space indent for canvas body", line.line_no);
        }

        if let Some(value) = line.content.strip_prefix("size ") {
            let (w, h) = parse_size(value, line.line_no)?;
            width = Some(w);
            height = Some(h);
            index += 1;
            continue;
        }

        if let Some(value) = line.content.strip_prefix("width ") {
            width = Some(parse_u32(value, line.line_no, "width")?);
            index += 1;
            continue;
        }

        if let Some(value) = line.content.strip_prefix("height ") {
            height = Some(parse_u32(value, line.line_no, "height")?);
            index += 1;
            continue;
        }

        if let Some(value) = line.content.strip_prefix("background ") {
            background = Color::parse(value)
                .with_context(|| format!("line {}: invalid background", line.line_no))?;
            index += 1;
            continue;
        }

        nodes.push(parse_node(&lines, &mut index, 2)?);
    }

    Ok(CanvasDoc {
        name,
        width: width.ok_or_else(|| anyhow!("canvas width is required"))?,
        height: height.ok_or_else(|| anyhow!("canvas height is required"))?,
        background,
        nodes,
        text_styles,
        source_path: path.to_path_buf(),
    })
}

fn parse_style_definition(
    lines: &[ParsedLine],
    index: &mut usize,
    styles: &mut HashMap<String, TextStyle>,
) -> Result<()> {
    let line = &lines[*index];
    let name = line
        .content
        .strip_prefix("style ")
        .ok_or_else(|| anyhow!("line {}: invalid style declaration", line.line_no))?
        .trim();

    if name.is_empty() {
        bail!("line {}: style name cannot be empty", line.line_no);
    }

    let mut style = TextStyle::default();
    *index += 1;

    while *index < lines.len() {
        let prop = &lines[*index];
        if prop.indent == 0 {
            break;
        }
        if prop.indent != 2 {
            bail!(
                "line {}: expected 2-space indent for style properties",
                prop.line_no
            );
        }
        apply_text_style_property(&mut style, prop)?;
        *index += 1;
    }

    styles.insert(name.to_string(), style);
    Ok(())
}

fn parse_node(lines: &[ParsedLine], index: &mut usize, indent: usize) -> Result<Node> {
    let line = &lines[*index];
    if line.indent != indent {
        bail!("line {}: unexpected indentation for node `{}`", line.line_no, line.content);
    }

    if let Some(path_value) = parse_quoted_command(&line.content, "image")? {
        let mut node = ImageNode {
            path: PathBuf::from(path_value),
            x: 0,
            y: 0,
            width: None,
            height: None,
            fit: FitMode::None,
            opacity: 1.0,
        };
        *index += 1;
        while *index < lines.len() && lines[*index].indent > indent {
            let prop = &lines[*index];
            if prop.indent != indent + 2 {
                bail!(
                    "line {}: expected {}-space indent for image properties",
                    prop.line_no,
                    indent + 2
                );
            }
            apply_image_property(&mut node, prop)?;
            *index += 1;
        }
        return Ok(Node::Image(node));
    }

    if line.content == "rect" {
        let mut node = RectNode {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            fill: Color::WHITE,
        };
        *index += 1;
        while *index < lines.len() && lines[*index].indent > indent {
            let prop = &lines[*index];
            if prop.indent != indent + 2 {
                bail!(
                    "line {}: expected {}-space indent for rect properties",
                    prop.line_no,
                    indent + 2
                );
            }
            apply_rect_property(&mut node, prop)?;
            *index += 1;
        }
        if node.width == 0 || node.height == 0 {
            bail!("rect node requires non-zero width and height");
        }
        return Ok(Node::Rect(node));
    }

    if let Some((text_value, inline_style)) = parse_text_statement(&line.content)? {
        let mut node = TextNode {
            value: text_value.to_string(),
            x: 0,
            y: 0,
            width: None,
            style_name: inline_style.map(str::to_string),
            font: None,
            color: None,
            line_height: None,
            weight: None,
            letter_spacing: None,
            align: None,
            font_family: None,
            font_path: None,
            language: None,
        };

        while *index < lines.len() && lines[*index].indent > indent {
            let prop = &lines[*index];
            if prop.indent != indent + 2 {
                bail!(
                    "line {}: expected {}-space indent for text properties",
                    prop.line_no,
                    indent + 2
                );
            }
            apply_text_property(&mut node, prop)?;
            *index += 1;
        }
        return Ok(Node::Text(node));
    }

    if line.content == "row" {
        return parse_layout_node(lines, index, indent, "row");
    }

    if line.content == "column" {
        return parse_layout_node(lines, index, indent, "column");
    }

    bail!("line {}: unsupported statement `{}`", line.line_no, line.content)
}

fn parse_layout_node(lines: &[ParsedLine], index: &mut usize, indent: usize, kind: &str) -> Result<Node> {
    let mut node = LayoutNode {
        x: 0,
        y: 0,
        gap: 0,
        padding_top: 0,
        padding_right: 0,
        padding_bottom: 0,
        padding_left: 0,
        children: Vec::new(),
    };

    *index += 1;
    while *index < lines.len() {
        let line = &lines[*index];
        if line.indent <= indent {
            break;
        }
        if line.indent != indent + 2 {
            bail!(
                "line {}: expected {}-space indent for {kind} properties or children",
                line.line_no,
                indent + 2
            );
        }

        if is_layout_property(&line.content) {
            apply_layout_property(&mut node, line)?;
            *index += 1;
            continue;
        }

        node.children.push(parse_node(lines, index, indent + 2)?);
    }

    if node.children.is_empty() {
        bail!("{kind} node requires at least one child node");
    }

    Ok(match kind {
        "row" => Node::Row(node),
        "column" => Node::Column(node),
        _ => unreachable!(),
    })
}

fn collect_lines(source: &str) -> Result<Vec<ParsedLine>> {
    let mut lines = Vec::new();

    for (index, raw_line) in source.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if raw_line.contains('\t') {
            bail!("line {}: tabs are not supported, use spaces only", line_no);
        }
        let indent = raw_line.chars().take_while(|c| *c == ' ').count();
        if indent % 2 != 0 {
            bail!("line {}: indentation must be multiples of 2 spaces", line_no);
        }
        lines.push(ParsedLine {
            indent,
            content: trimmed.to_string(),
            line_no,
        });
    }

    Ok(lines)
}

fn parse_quoted_command<'a>(content: &'a str, command: &str) -> Result<Option<&'a str>> {
    let prefix = format!("{command} ");
    let Some(rest) = content.strip_prefix(&prefix) else {
        return Ok(None);
    };
    Ok(Some(parse_quoted_value(rest.trim(), command)?))
}

fn parse_quoted_value<'a>(value: &'a str, label: &str) -> Result<&'a str> {
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        bail!("{label} path/value must be quoted");
    }
    Ok(&value[1..value.len() - 1])
}

fn parse_text_statement(content: &str) -> Result<Option<(&str, Option<&str>)>> {
    let Some(rest) = content.strip_prefix("text ") else {
        return Ok(None);
    };
    if !rest.starts_with('"') {
        bail!("text path/value must be quoted");
    }

    let Some(relative_end) = rest[1..].find('"') else {
        bail!("text path/value must be quoted");
    };
    let end = relative_end + 1;
    let value = &rest[1..end];
    let trailing = rest[end + 1..].trim();

    if trailing.is_empty() {
        return Ok(Some((value, None)));
    }

    let style_name = trailing
        .strip_prefix("style ")
        .ok_or_else(|| anyhow!("unsupported text statement `{content}`"))?
        .trim();

    if style_name.is_empty() {
        bail!("text style name cannot be empty");
    }

    Ok(Some((value, Some(style_name))))
}

fn parse_size(value: &str, line_no: usize) -> Result<(u32, u32)> {
    let (left, right) = value
        .split_once('x')
        .ok_or_else(|| anyhow!("line {line_no}: size must be like 1200x800"))?;
    Ok((
        parse_u32(left, line_no, "width")?,
        parse_u32(right, line_no, "height")?,
    ))
}

fn parse_u32(value: &str, line_no: usize, label: &str) -> Result<u32> {
    value
        .trim()
        .parse::<u32>()
        .with_context(|| format!("line {line_no}: invalid {label}: {}", value.trim()))
}

fn parse_i32(value: &str, line_no: usize, label: &str) -> Result<i32> {
    value
        .trim()
        .parse::<i32>()
        .with_context(|| format!("line {line_no}: invalid {label}: {}", value.trim()))
}

fn parse_i64(value: &str, line_no: usize, label: &str) -> Result<i64> {
    value
        .trim()
        .parse::<i64>()
        .with_context(|| format!("line {line_no}: invalid {label}: {}", value.trim()))
}

fn parse_f32(value: &str, line_no: usize, label: &str) -> Result<f32> {
    value
        .trim()
        .parse::<f32>()
        .with_context(|| format!("line {line_no}: invalid {label}: {}", value.trim()))
}

fn parse_spacing(value: &str, line_no: usize, label: &str) -> Result<(i32, i32, i32, i32)> {
    let parts = value
        .split_whitespace()
        .map(|part| parse_i32(part, line_no, label))
        .collect::<Result<Vec<_>>>()?;

    match parts.as_slice() {
        [all] => Ok((*all, *all, *all, *all)),
        [vertical, horizontal] => Ok((*vertical, *horizontal, *vertical, *horizontal)),
        [top, right, bottom, left] => Ok((*top, *right, *bottom, *left)),
        _ => bail!("line {line_no}: {label} must have 1, 2, or 4 integer values"),
    }
}

fn is_layout_property(content: &str) -> bool {
    content.starts_with("x ")
        || content.starts_with("y ")
        || content.starts_with("gap ")
        || content.starts_with("padding ")
}

fn apply_layout_property(node: &mut LayoutNode, prop: &ParsedLine) -> Result<()> {
    if let Some(value) = prop.content.strip_prefix("x ") {
        node.x = parse_i32(value, prop.line_no, "x")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("y ") {
        node.y = parse_i32(value, prop.line_no, "y")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("gap ") {
        node.gap = parse_i32(value, prop.line_no, "gap")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("padding ") {
        let (top, right, bottom, left) = parse_spacing(value, prop.line_no, "padding")?;
        node.padding_top = top;
        node.padding_right = right;
        node.padding_bottom = bottom;
        node.padding_left = left;
        return Ok(());
    }

    bail!("line {}: unsupported layout property `{}`", prop.line_no, prop.content)
}

fn apply_image_property(node: &mut ImageNode, prop: &ParsedLine) -> Result<()> {
    if let Some(value) = prop.content.strip_prefix("x ") {
        node.x = parse_i64(value, prop.line_no, "x")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("y ") {
        node.y = parse_i64(value, prop.line_no, "y")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("width ") {
        node.width = Some(parse_u32(value, prop.line_no, "width")?);
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("height ") {
        node.height = Some(parse_u32(value, prop.line_no, "height")?);
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("fit ") {
        node.fit = match value.trim() {
            "none" => FitMode::None,
            "fill" => FitMode::Fill,
            "contain" => FitMode::Contain,
            "cover" => FitMode::Cover,
            other => bail!("line {}: unsupported fit mode `{other}`", prop.line_no),
        };
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("opacity ") {
        node.opacity = parse_f32(value, prop.line_no, "opacity")?.clamp(0.0, 1.0);
        return Ok(());
    }

    bail!("line {}: unsupported image property `{}`", prop.line_no, prop.content)
}

fn apply_rect_property(node: &mut RectNode, prop: &ParsedLine) -> Result<()> {
    if let Some(value) = prop.content.strip_prefix("x ") {
        node.x = parse_i32(value, prop.line_no, "x")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("y ") {
        node.y = parse_i32(value, prop.line_no, "y")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("width ") {
        node.width = parse_u32(value, prop.line_no, "width")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("height ") {
        node.height = parse_u32(value, prop.line_no, "height")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("fill ") {
        node.fill = Color::parse(value)
            .with_context(|| format!("line {}: invalid fill color", prop.line_no))?;
        return Ok(());
    }

    bail!("line {}: unsupported rect property `{}`", prop.line_no, prop.content)
}

fn apply_text_property(node: &mut TextNode, prop: &ParsedLine) -> Result<()> {
    if let Some(value) = prop.content.strip_prefix("x ") {
        node.x = parse_i32(value, prop.line_no, "x")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("y ") {
        node.y = parse_i32(value, prop.line_no, "y")?;
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("width ") {
        node.width = Some(parse_u32(value, prop.line_no, "width")?);
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("style ") {
        let style_name = value.trim();
        if style_name.is_empty() {
            bail!("line {}: text style name cannot be empty", prop.line_no);
        }
        node.style_name = Some(style_name.to_string());
        return Ok(());
    }

    apply_text_style_fields(
        &mut node.font,
        &mut node.color,
        &mut node.line_height,
        &mut node.weight,
        &mut node.letter_spacing,
        &mut node.align,
        &mut node.font_family,
        &mut node.font_path,
        &mut node.language,
        prop,
    )
}

fn apply_text_style_property(style: &mut TextStyle, prop: &ParsedLine) -> Result<()> {
    apply_text_style_fields(
        &mut style.font,
        &mut style.color,
        &mut style.line_height,
        &mut style.weight,
        &mut style.letter_spacing,
        &mut style.align,
        &mut style.font_family,
        &mut style.font_path,
        &mut style.language,
        prop,
    )
}

fn apply_text_style_fields(
    font: &mut Option<f32>,
    color: &mut Option<Color>,
    line_height: &mut Option<f32>,
    weight: &mut Option<u32>,
    letter_spacing: &mut Option<f32>,
    align: &mut Option<TextAlign>,
    font_family: &mut Option<String>,
    font_path: &mut Option<PathBuf>,
    language: &mut Option<String>,
    prop: &ParsedLine,
) -> Result<()> {
    if let Some(value) = prop.content.strip_prefix("font ") {
        *font = Some(parse_f32(value, prop.line_no, "font")?.max(1.0));
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("font-family ") {
        *font_family = Some(parse_quoted_value(value.trim(), "font-family")?.to_string());
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("font-path ") {
        *font_path = Some(PathBuf::from(parse_quoted_value(value.trim(), "font-path")?));
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("language ") {
        *language = Some(value.trim().to_string());
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("color ") {
        *color = Some(
            Color::parse(value)
                .with_context(|| format!("line {}: invalid text color", prop.line_no))?,
        );
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("line-height ") {
        *line_height = Some(parse_f32(value, prop.line_no, "line-height")?.max(0.5));
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("weight ") {
        *weight = Some(parse_u32(value, prop.line_no, "weight")?);
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("letter-spacing ") {
        *letter_spacing = Some(parse_f32(value, prop.line_no, "letter-spacing")?);
        return Ok(());
    }
    if let Some(value) = prop.content.strip_prefix("align ") {
        *align = Some(
            TextAlign::parse(value)
                .with_context(|| format!("line {}: invalid text align", prop.line_no))?,
        );
        return Ok(());
    }

    bail!("line {}: unsupported text property `{}`", prop.line_no, prop.content)
}
