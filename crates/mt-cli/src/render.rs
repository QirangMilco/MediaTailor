use std::cmp::max;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use image::imageops::{self, FilterType};
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};

use crate::config::ProjectConfig;
use crate::font::{px_scale, resolve_text_font};
use crate::parser::{
    CanvasDoc, Color, FitMode, ImageNode, LayoutNode, Node, RectNode, ResolvedTextStyle, TextNode,
    TextStyle,
};

pub fn render_canvas(doc: &CanvasDoc, config: &ProjectConfig) -> Result<RgbaImage> {
    let mut canvas = RgbaImage::from_pixel(doc.width, doc.height, to_rgba(doc.background));
    let base_dir = doc
        .source_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    for node in &doc.nodes {
        render_node(&mut canvas, node, 0, 0, &base_dir, doc, config)?;
    }

    Ok(canvas)
}

fn resolve_text_node_style(
    doc: &CanvasDoc,
    node: &TextNode,
    config: &ProjectConfig,
) -> Result<ResolvedTextStyle> {
    let mut merged = TextStyle::default();

    if let Some(style_name) = &node.style_name {
        if let Some(project_style) = config.text_styles.get(style_name) {
            merged.merge_from(project_style);
        }
        if let Some(file_style) = doc.text_styles.get(style_name) {
            merged.merge_from(file_style);
        }
        if !config.text_styles.contains_key(style_name) && !doc.text_styles.contains_key(style_name) {
            return Err(anyhow!("unknown text style: {style_name}"));
        }
    }

    merged.merge_from(&node.inline_style());
    Ok(merged.resolve())
}

fn render_node(
    canvas: &mut RgbaImage,
    node: &Node,
    origin_x: i32,
    origin_y: i32,
    base_dir: &Path,
    doc: &CanvasDoc,
    config: &ProjectConfig,
) -> Result<()> {
    match node {
        Node::Rect(rect) => {
            draw_rect(canvas, rect, origin_x, origin_y);
            Ok(())
        }
        Node::Image(image) => draw_image(canvas, image, origin_x, origin_y, base_dir),
        Node::Text(text) => {
            let style = resolve_text_node_style(doc, text, config)?;
            let font = resolve_text_font(&style, config, base_dir)?;
            draw_text_node(canvas, text, &style, origin_x, origin_y, &font);
            Ok(())
        }
        Node::Row(layout) => draw_layout(canvas, layout, true, origin_x, origin_y, base_dir, doc, config),
        Node::Column(layout) => {
            draw_layout(canvas, layout, false, origin_x, origin_y, base_dir, doc, config)
        }
    }
}

fn draw_layout(
    canvas: &mut RgbaImage,
    layout: &LayoutNode,
    is_row: bool,
    origin_x: i32,
    origin_y: i32,
    base_dir: &Path,
    doc: &CanvasDoc,
    config: &ProjectConfig,
) -> Result<()> {
    let layout_origin_x = origin_x + layout.x;
    let layout_origin_y = origin_y + layout.y;
    let mut cursor_x = layout.padding_left.max(0);
    let mut cursor_y = layout.padding_top.max(0);
    let gap = layout.gap.max(0);

    for child in &layout.children {
        let child_origin_x = layout_origin_x + cursor_x;
        let child_origin_y = layout_origin_y + cursor_y;
        render_node(canvas, child, child_origin_x, child_origin_y, base_dir, doc, config)?;

        let (child_width, child_height) = measure_node(child, base_dir, doc, config)?;
        let occupied_width = child_width + node_offset_x(child).max(0) as u32;
        let occupied_height = child_height + node_offset_y(child).max(0) as u32;

        if is_row {
            cursor_x += occupied_width as i32 + gap;
        } else {
            cursor_y += occupied_height as i32 + gap;
        }
    }

    Ok(())
}

fn measure_node(node: &Node, base_dir: &Path, doc: &CanvasDoc, config: &ProjectConfig) -> Result<(u32, u32)> {
    match node {
        Node::Rect(rect) => Ok((rect.width, rect.height)),
        Node::Image(image) => measure_image(image, base_dir),
        Node::Text(text) => measure_text(text, base_dir, doc, config),
        Node::Row(layout) => measure_layout(layout, true, base_dir, doc, config),
        Node::Column(layout) => measure_layout(layout, false, base_dir, doc, config),
    }
}

fn measure_layout(
    layout: &LayoutNode,
    is_row: bool,
    base_dir: &Path,
    doc: &CanvasDoc,
    config: &ProjectConfig,
) -> Result<(u32, u32)> {
    let mut main = 0u32;
    let mut cross = 0u32;
    let gap = layout.gap.max(0) as u32;

    for (index, child) in layout.children.iter().enumerate() {
        let (child_width, child_height) = measure_node(child, base_dir, doc, config)?;
        let occupied_width = child_width + node_offset_x(child).max(0) as u32;
        let occupied_height = child_height + node_offset_y(child).max(0) as u32;

        if is_row {
            if index > 0 {
                main += gap;
            }
            main += occupied_width;
            cross = max(cross, occupied_height);
        } else {
            if index > 0 {
                main += gap;
            }
            main += occupied_height;
            cross = max(cross, occupied_width);
        }
    }

    let padded_width = if is_row {
        main + layout.padding_left.max(0) as u32 + layout.padding_right.max(0) as u32
    } else {
        cross + layout.padding_left.max(0) as u32 + layout.padding_right.max(0) as u32
    };
    let padded_height = if is_row {
        cross + layout.padding_top.max(0) as u32 + layout.padding_bottom.max(0) as u32
    } else {
        main + layout.padding_top.max(0) as u32 + layout.padding_bottom.max(0) as u32
    };

    Ok((padded_width, padded_height))
}

fn node_offset_x(node: &Node) -> i32 {
    match node {
        Node::Rect(rect) => rect.x,
        Node::Image(image) => image.x.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        Node::Text(text) => text.x,
        Node::Row(layout) | Node::Column(layout) => layout.x,
    }
}

fn node_offset_y(node: &Node) -> i32 {
    match node {
        Node::Rect(rect) => rect.y,
        Node::Image(image) => image.y.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        Node::Text(text) => text.y,
        Node::Row(layout) | Node::Column(layout) => layout.y,
    }
}

fn measure_image(node: &ImageNode, base_dir: &Path) -> Result<(u32, u32)> {
    let image_path = base_dir.join(&node.path);
    let (src_w, src_h) = image::image_dimensions(&image_path)
        .with_context(|| format!("failed to open image asset: {}", image_path.display()))?;
    Ok((node.width.unwrap_or(src_w), node.height.unwrap_or(src_h)))
}

fn measure_text(
    node: &TextNode,
    base_dir: &Path,
    doc: &CanvasDoc,
    config: &ProjectConfig,
) -> Result<(u32, u32)> {
    let style = resolve_text_node_style(doc, node, config)?;
    let font = resolve_text_font(&style, config, base_dir)?;
    let scale = px_scale(style.font);
    let lines = wrap_text(node, &style, &font, scale);
    let width = lines
        .iter()
        .map(|line| measure_text_line(line, &style, &font, scale))
        .fold(0.0_f32, f32::max)
        .ceil() as u32;
    let height = measure_text_block_height(lines.len(), &style);
    Ok((width, height))
}

fn draw_rect(canvas: &mut RgbaImage, rect: &RectNode, origin_x: i32, origin_y: i32) {
    for dy in 0..rect.height {
        for dx in 0..rect.width {
            let x = origin_x + rect.x + dx as i32;
            let y = origin_y + rect.y + dy as i32;
            if x < 0 || y < 0 {
                continue;
            }
            let (x, y) = (x as u32, y as u32);
            if x >= canvas.width() || y >= canvas.height() {
                continue;
            }
            canvas.put_pixel(x, y, to_rgba(rect.fill));
        }
    }
}

fn draw_image(
    canvas: &mut RgbaImage,
    node: &ImageNode,
    origin_x: i32,
    origin_y: i32,
    base_dir: &Path,
) -> Result<()> {
    let image_path = base_dir.join(&node.path);
    let source = image::open(&image_path)
        .with_context(|| format!("failed to open image asset: {}", image_path.display()))?;

    let prepared = prepare_image(source, node);
    overlay_with_opacity(
        canvas,
        &prepared,
        origin_x as i64 + node.x,
        origin_y as i64 + node.y,
        node.opacity,
    );
    Ok(())
}

fn draw_text_node(
    canvas: &mut RgbaImage,
    node: &TextNode,
    style: &ResolvedTextStyle,
    origin_x: i32,
    origin_y: i32,
    font: &ab_glyph::FontArc,
) {
    let scale = px_scale(style.font);
    let lines = wrap_text(node, style, font, scale);
    let line_height = (style.font * style.line_height).max(style.font);
    let color = to_rgba(style.color);
    let text_x = origin_x + node.x;
    let text_y = origin_y + node.y;

    for (index, line) in lines.iter().enumerate() {
        let y = text_y + (index as f32 * line_height).round() as i32;
        let x = aligned_text_x(text_x, node.width, line, style, font, scale);
        if style.weight >= 600 {
            let shadow = Rgba([color[0], color[1], color[2], (color[3] / 2).max(1)]);
            draw_text_line(canvas, line, shadow, x + 1, y + 1, style, font, scale);
        }
        draw_text_line(canvas, line, color, x, y, style, font, scale);
    }
}

fn wrap_text(
    node: &TextNode,
    style: &ResolvedTextStyle,
    font: &ab_glyph::FontArc,
    scale: ab_glyph::PxScale,
) -> Vec<String> {
    let Some(max_width) = node.width else {
        return node.value.lines().map(|line| line.to_string()).collect();
    };

    let mut lines = Vec::new();
    for raw_line in node.value.lines() {
        if raw_line.split_whitespace().count() > 1 {
            wrap_whitespace_line(raw_line, max_width, style, font, scale, &mut lines);
        } else {
            wrap_character_line(raw_line, max_width, style, font, scale, &mut lines);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn wrap_whitespace_line(
    raw_line: &str,
    max_width: u32,
    style: &ResolvedTextStyle,
    font: &ab_glyph::FontArc,
    scale: ab_glyph::PxScale,
    lines: &mut Vec<String>,
) {
    let mut current = String::new();
    for word in raw_line.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let width = measure_text_line(&candidate, style, font, scale);
        if width <= max_width as f32 || current.is_empty() {
            current = candidate;
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if current.is_empty() {
        lines.push(String::new());
    } else {
        lines.push(current);
    }
}

fn wrap_character_line(
    raw_line: &str,
    max_width: u32,
    style: &ResolvedTextStyle,
    font: &ab_glyph::FontArc,
    scale: ab_glyph::PxScale,
    lines: &mut Vec<String>,
) {
    let mut current = String::new();
    for ch in raw_line.chars() {
        let candidate = format!("{current}{ch}");
        let width = measure_text_line(&candidate, style, font, scale);
        if width <= max_width as f32 || current.is_empty() {
            current = candidate;
        } else {
            lines.push(current);
            current = ch.to_string();
        }
    }

    if current.is_empty() {
        lines.push(String::new());
    } else {
        lines.push(current);
    }
}

fn measure_text_line(
    line: &str,
    style: &ResolvedTextStyle,
    font: &ab_glyph::FontArc,
    scale: ab_glyph::PxScale,
) -> f32 {
    let (base_width, _) = text_size(scale, font, line);
    let extra_spacing = style.letter_spacing.max(0.0) * line.chars().count().saturating_sub(1) as f32;
    base_width as f32 + extra_spacing
}

fn measure_text_block_height(line_count: usize, style: &ResolvedTextStyle) -> u32 {
    let line_height = (style.font * style.line_height).max(style.font);
    if line_count == 0 {
        0
    } else {
        ((line_count.saturating_sub(1) as f32) * line_height + style.font).ceil() as u32
    }
}

fn aligned_text_x(
    text_x: i32,
    max_width: Option<u32>,
    line: &str,
    style: &ResolvedTextStyle,
    font: &ab_glyph::FontArc,
    scale: ab_glyph::PxScale,
) -> i32 {
    let Some(container_width) = max_width else {
        return text_x;
    };

    let line_width = measure_text_line(line, style, font, scale);
    let remaining = (container_width as f32 - line_width).max(0.0);
    match style.align {
        crate::parser::TextAlign::Left => text_x,
        crate::parser::TextAlign::Center => text_x + (remaining / 2.0).round() as i32,
        crate::parser::TextAlign::Right => text_x + remaining.round() as i32,
    }
}

fn draw_text_line(
    canvas: &mut RgbaImage,
    line: &str,
    color: Rgba<u8>,
    x: i32,
    y: i32,
    style: &ResolvedTextStyle,
    font: &ab_glyph::FontArc,
    scale: ab_glyph::PxScale,
) {
    if style.letter_spacing <= 0.0 || line.chars().count() <= 1 {
        draw_text_mut(canvas, color, x, y, scale, font, line);
        return;
    }

    let mut cursor_x = x as f32;
    for ch in line.chars() {
        let glyph = ch.to_string();
        draw_text_mut(canvas, color, cursor_x.round() as i32, y, scale, font, &glyph);
        let (glyph_width, _) = text_size(scale, font, &glyph);
        cursor_x += glyph_width as f32 + style.letter_spacing;
    }
}

fn prepare_image(source: DynamicImage, node: &ImageNode) -> RgbaImage {
    let src_w = source.width();
    let src_h = source.height();
    let target_w = node.width.unwrap_or(src_w).max(1);
    let target_h = node.height.unwrap_or(src_h).max(1);

    match node.fit {
        FitMode::None => {
            if node.width.is_some() || node.height.is_some() {
                source
                    .resize(target_w, target_h, FilterType::Lanczos3)
                    .to_rgba8()
            } else {
                source.to_rgba8()
            }
        }
        FitMode::Fill => source
            .resize_exact(target_w, target_h, FilterType::Lanczos3)
            .to_rgba8(),
        FitMode::Contain => source.resize(target_w, target_h, FilterType::Lanczos3).to_rgba8(),
        FitMode::Cover => cover_resize(source.to_rgba8(), target_w, target_h),
    }
}

fn cover_resize(source: RgbaImage, target_w: u32, target_h: u32) -> RgbaImage {
    let src_w = source.width().max(1);
    let src_h = source.height().max(1);

    let width_ratio = target_w as f32 / src_w as f32;
    let height_ratio = target_h as f32 / src_h as f32;
    let scale = width_ratio.max(height_ratio);

    let scaled_w = ((src_w as f32) * scale).ceil().max(target_w as f32) as u32;
    let scaled_h = ((src_h as f32) * scale).ceil().max(target_h as f32) as u32;

    let resized = imageops::resize(&source, scaled_w, scaled_h, FilterType::Lanczos3);
    let crop_x = scaled_w.saturating_sub(target_w) / 2;
    let crop_y = scaled_h.saturating_sub(target_h) / 2;
    imageops::crop_imm(&resized, crop_x, crop_y, target_w, target_h).to_image()
}

fn overlay_with_opacity(canvas: &mut RgbaImage, layer: &RgbaImage, x: i64, y: i64, opacity: f32) {
    let opacity = opacity.clamp(0.0, 1.0);
    for (lx, ly, pixel) in layer.enumerate_pixels() {
        let tx = x + lx as i64;
        let ty = y + ly as i64;
        if tx < 0 || ty < 0 {
            continue;
        }
        let (tx, ty) = (tx as u32, ty as u32);
        if tx >= canvas.width() || ty >= canvas.height() {
            continue;
        }

        let src = *pixel;
        let mut src_alpha = (src[3] as f32 / 255.0) * opacity;
        if src_alpha <= 0.0 {
            continue;
        }
        src_alpha = src_alpha.clamp(0.0, 1.0);

        let dst = canvas.get_pixel(tx, ty);
        let dst_alpha = dst[3] as f32 / 255.0;
        let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
        let blend_channel = |s: u8, d: u8| -> u8 {
            if out_alpha <= f32::EPSILON {
                0
            } else {
                let s = s as f32 / 255.0;
                let d = d as f32 / 255.0;
                let out = (s * src_alpha + d * dst_alpha * (1.0 - src_alpha)) / out_alpha;
                (out * 255.0).round().clamp(0.0, 255.0) as u8
            }
        };

        canvas.put_pixel(
            tx,
            ty,
            Rgba([
                blend_channel(src[0], dst[0]),
                blend_channel(src[1], dst[1]),
                blend_channel(src[2], dst[2]),
                (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8,
            ]),
        );
    }
}

fn to_rgba(color: Color) -> Rgba<u8> {
    Rgba([color.r, color.g, color.b, color.a])
}
