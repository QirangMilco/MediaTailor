use std::fs;
use std::path::{Path, PathBuf};

use ab_glyph::{FontArc, PxScale};
use anyhow::{bail, Context, Result};

use crate::config::ProjectConfig;
use crate::parser::ResolvedTextStyle;

pub fn load_font_from_path(path: &Path) -> Result<FontArc> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read font file: {}", path.display()))?;
    FontArc::try_from_vec(bytes)
        .with_context(|| format!("failed to parse font file: {}", path.display()))
}

fn load_first_existing_font(candidates: &[PathBuf]) -> Result<FontArc> {
    for path in candidates {
        if path.exists() {
            return load_font_from_path(path);
        }
    }

    let joined = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    bail!("no usable font found in candidate paths: {joined}")
}

fn normalize_family_name(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-' && *ch != '_')
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn builtin_font_candidates(root_dir: &Path, family: &str) -> Option<Vec<PathBuf>> {
    let windows_dir = root_dir.join("fonts").join("windows");
    let normalized = normalize_family_name(family);

    let candidates = match normalized.as_str() {
        "timesnewroman" | "timesroman" | "新罗马" | "times" => vec![
            windows_dir.join("times.ttf"),
            PathBuf::from("/usr/share/fonts/truetype/msttcorefonts/Times_New_Roman.ttf"),
            PathBuf::from("/usr/share/fonts/truetype/liberation2/LiberationSerif-Regular.ttf"),
            PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf"),
            PathBuf::from("/usr/share/fonts/truetype/noto/NotoSerif-Regular.ttf"),
        ],
        "simsun" | "songti" | "song" | "宋体" => vec![
            windows_dir.join("simsun.ttc"),
            PathBuf::from("/usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc"),
            PathBuf::from("/usr/share/fonts/truetype/noto/NotoSerif-Regular.ttf"),
        ],
        "simhei" | "heiti" | "黑体" => vec![
            windows_dir.join("simhei.ttf"),
            PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
            PathBuf::from("/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf"),
        ],
        "kaiti" | "simkai" | "楷体" => vec![
            windows_dir.join("simkai.ttf"),
            PathBuf::from("/usr/share/fonts/truetype/noto/NotoSansKaithi-Regular.ttf"),
            PathBuf::from("/usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc"),
        ],
        "microsoftyahei" | "yahei" | "微软雅黑" => vec![
            windows_dir.join("msyh.ttc"),
            PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
            PathBuf::from("/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf"),
        ],
        _ => return None,
    };

    Some(candidates)
}

fn builtin_default_family_for_language(language: &str) -> Option<&'static str> {
    let normalized = language.to_ascii_lowercase();

    if normalized.starts_with("zh") {
        return Some("宋体");
    }
    if normalized.starts_with("en") {
        return Some("Times New Roman");
    }

    None
}

fn resolve_named_font_family(font_family: &str, config: &ProjectConfig) -> Result<FontArc> {
    if let Some(path) = config.font_families.get(font_family) {
        return load_font_from_path(path);
    }

    if let Some(candidates) = builtin_font_candidates(&config.root_dir, font_family) {
        return load_first_existing_font(&candidates).with_context(|| {
            format!(
                "builtin font family `{font_family}` not found; place the matching font file under {}",
                config.root_dir.join("fonts/windows").display()
            )
        });
    }

    bail!("unknown font family: {font_family}")
}

pub fn load_default_font() -> Result<FontArc> {
    let candidates = [
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
    ];

    for candidate in candidates {
        let path = Path::new(candidate);
        if !path.exists() {
            continue;
        }
        return load_font_from_path(path);
    }

    anyhow::bail!("no usable system font found for text rendering")
}

pub fn resolve_text_font(
    style: &ResolvedTextStyle,
    config: &ProjectConfig,
    base_dir: &Path,
) -> Result<FontArc> {
    if let Some(font_path) = &style.font_path {
        let path = if font_path.is_absolute() {
            font_path.clone()
        } else {
            base_dir.join(font_path)
        };
        return load_font_from_path(&path);
    }

    if let Some(font_family) = &style.font_family {
        return resolve_named_font_family(font_family, config);
    }

    if let Some(language) = &style.language {
        if let Some(font_family) = config.language_defaults.get(language) {
            return resolve_named_font_family(font_family, config).with_context(|| {
                format!("font family `{font_family}` referenced by language `{language}` is not defined")
            });
        }

        if let Some(font_family) = builtin_default_family_for_language(language) {
            if let Ok(font) = resolve_named_font_family(font_family, config) {
                return Ok(font);
            }
        }
    }

    if let Some(font_family) = &config.default_font_family {
        return resolve_named_font_family(font_family, config).with_context(|| {
            format!("default font family `{font_family}` is not defined in MediaTailor.toml")
        });
    }

    if let Ok(font) = resolve_named_font_family("Times New Roman", config) {
        return Ok(font);
    }

    load_default_font()
}

fn ensure_known_font_family(config: &ProjectConfig, family: &str) -> Result<()> {
    if config.font_families.contains_key(family) {
        return Ok(());
    }
    if builtin_font_candidates(&config.root_dir, family).is_some() {
        return Ok(());
    }

    bail!("unknown font family: {family}")
}

pub fn validate_project_fonts(config: &ProjectConfig) -> Result<()> {
    for (family, path) in &config.font_families {
        if !path.exists() {
            bail!(
                "font family `{family}` points to missing file: {}",
                path.display()
            );
        }
    }

    for (language, family) in &config.language_defaults {
        ensure_known_font_family(config, family).with_context(|| {
            format!("font family `{family}` referenced by language `{language}` is not defined")
        })?;
    }

    if let Some(family) = &config.default_font_family {
        ensure_known_font_family(config, family).with_context(|| {
            format!("default font family `{family}` is not defined in MediaTailor.toml")
        })?;
    }

    for (style_name, style) in &config.text_styles {
        if let Some(path) = &style.font_path {
            if !path.exists() {
                bail!(
                    "style `{style_name}` font-path points to missing file: {}",
                    path.display()
                );
            }
        }
        if let Some(family) = &style.font_family {
            ensure_known_font_family(config, family)
                .with_context(|| format!("style `{style_name}` references unknown font family `{family}`"))?;
        }
    }

    Ok(())
}

pub fn px_scale(font_size: f32) -> PxScale {
    PxScale::from(font_size.max(1.0))
}
