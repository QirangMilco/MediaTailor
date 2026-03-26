use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use crate::parser::{TextAlign, TextStyle};


#[derive(Debug, Clone, Default)]
pub struct ProjectConfig {
    pub root_dir: PathBuf,
    pub font_families: HashMap<String, PathBuf>,
    pub default_font_family: Option<String>,
    pub language_defaults: HashMap<String, String>,
    pub text_styles: HashMap<String, TextStyle>,
}

#[derive(Debug, Deserialize, Default)]
struct RawProjectConfig {
    #[serde(default)]
    fonts: RawFontsConfig,
    #[serde(default)]
    styles: HashMap<String, RawTextStyle>,
}

#[derive(Debug, Deserialize, Default)]
struct RawFontsConfig {
    #[serde(default)]
    families: HashMap<String, String>,
    #[serde(default)]
    defaults: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawTextStyle {
    font: Option<f32>,
    color: Option<String>,
    #[serde(rename = "line-height")]
    line_height: Option<f32>,
    weight: Option<u32>,
    #[serde(rename = "letter-spacing")]
    letter_spacing: Option<f32>,
    align: Option<String>,
    #[serde(rename = "font-family")]
    font_family: Option<String>,
    #[serde(rename = "font-path")]
    font_path: Option<String>,
    language: Option<String>,
}

fn find_project_config_path(input_path: &Path) -> Option<PathBuf> {
    let mut current = input_path.parent()?;

    loop {
        let candidate = current.join("MediaTailor.toml");
        if candidate.exists() {
            return Some(candidate);
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }

    None
}

pub fn load_project_config(input_path: &Path) -> Result<ProjectConfig> {
    let fallback_root_dir = input_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let Some(config_path) = find_project_config_path(input_path) else {
        return Ok(ProjectConfig {
            root_dir: fallback_root_dir,
            ..ProjectConfig::default()
        });
    };

    let root_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let source = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config file: {}", config_path.display()))?;
    let raw: RawProjectConfig = toml::from_str(&source)
        .with_context(|| format!("failed to parse config file: {}", config_path.display()))?;

    let font_families = raw
        .fonts
        .families
        .into_iter()
        .map(|(name, value)| (name, root_dir.join(value)))
        .collect();

    let default_font_family = raw.fonts.defaults.get("default").cloned();
    let language_defaults = raw
        .fonts
        .defaults
        .into_iter()
        .filter(|(key, _)| key != "default")
        .collect();

    let text_styles = raw
        .styles
        .into_iter()
        .map(|(name, style)| {
            let text_style = TextStyle {
                font: style.font.map(|value| value.max(1.0)),
                color: style.color.as_deref().map(crate::parser::Color::parse).transpose()?,
                line_height: style.line_height.map(|value| value.max(0.5)),
                weight: style.weight,
                letter_spacing: style.letter_spacing,
                align: style.align.as_deref().map(TextAlign::parse).transpose()?,
                font_family: style.font_family,
                font_path: style.font_path.map(|value| root_dir.join(value)),
                language: style.language,
            };
            Ok((name, text_style))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    Ok(ProjectConfig {
        root_dir,
        font_families,
        default_font_family,
        language_defaults,
        text_styles,
    })
}
