// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors
// Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.

//! Reusable title presets and their optional fonts.
//!
//! Built-in TOML presets are embedded from this crate's `text-presets/`
//! directory. Additional presets live under `AppDirs::config/text-presets`
//! as individual TOML files or folders containing `preset.toml`.
//! A user preset with a built-in ID replaces that preset.
//!
//! Presets preserve a title's style, vertical position, language and sample
//! text. Optional font paths are resolved beside the preset file; reading a
//! preset does not install a font. Call [`install_font`] when selecting it,
//! then register the returned family/path on the project.
//!
//! ```toml
//! id = "studio.big-red"
//! name = "Big Red"
//! font = "BigRed.ttf"
//! offsetY = 0.3
//! language = "Latin"
//! order = 10
//! sample = "Aa"
//!
//! [style]
//! fontFamily = "Big Red"
//! fontSize = 0.1
//! fontWeight = 700
//! color = "#ff0000"
//! align = "center"
//! ```

use std::path::{Path, PathBuf};

use crate::AppDirs;
use concat_project::model::{TextAlign, TextStyle};
use serde::Deserialize;

/// One look a title can be given.
pub struct TextPreset {
    /// Stable for ever; "default" is the plain title.
    pub id: String,
    /// Human-readable name supplied by the preset author.
    pub name: String,
    /// The look, with `content` as the preset's initial words.
    pub style: TextStyle,
    /// Where the title sits, as a frame-height fraction from the centre,
    /// when the preset has an opinion - a lower third does.
    pub offset_y: Option<f64>,
    /// A font file the preset brings with it, resolved to a path.
    pub font: Option<PathBuf>,
    /// The writing its face is made for, as the font libraries name it:
    /// "Latin", "Japanese", "Hebrew". Available for callers to filter and sort
    /// presets by; a preset file that says nothing is Latin.
    pub language: String,
    /// Sample text for previewing the look: the file's `sample`, else "Aa" for
    /// Latin and the first two characters of its words for any other
    /// writing, so a Japanese preview shows Japanese.
    pub sample: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PresetFile {
    id: String,
    name: String,
    #[serde(default)]
    font: Option<String>,
    #[serde(default)]
    offset_y: Option<f64>,
    #[serde(default)]
    language: Option<String>,
    /// Where it sorts among its neighbours; absent sorts last, by name.
    #[serde(default)]
    order: Option<i64>,
    /// Sample text for previews; see [`TextPreset::sample`].
    #[serde(default)]
    sample: Option<String>,
    #[serde(default)]
    style: PresetStyle,
}

/// A title's fields, every one optional, laid over the default style.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PresetStyle {
    content: Option<String>,
    font_family: Option<String>,
    font_size: Option<f64>,
    font_weight: Option<f64>,
    italic: Option<bool>,
    color: Option<String>,
    align: Option<String>,
    opacity: Option<f64>,
    stroke_width: Option<f64>,
    stroke_color: Option<String>,
    shadow: Option<bool>,
    background: Option<String>,
    background_radius: Option<f64>,
    background_padding_x: Option<f64>,
    background_padding_y: Option<f64>,
    line_height: Option<f64>,
    tracking: Option<f64>,
    max_width: Option<f64>,
    max_height: Option<f64>,
}

impl PresetStyle {
    fn over(self, name: &str) -> TextStyle {
        let base = TextStyle::default();
        TextStyle {
            content: self.content.unwrap_or_else(|| name.to_owned()),
            font_family: self
                .font_family
                .unwrap_or_else(|| "Hanken Grotesk".to_owned()),
            font_size: self.font_size.unwrap_or(base.font_size).clamp(0.005, 1.0),
            font_weight: self
                .font_weight
                .unwrap_or(base.font_weight)
                .clamp(100.0, 900.0),
            italic: self.italic.unwrap_or(false),
            color: self.color.unwrap_or(base.color),
            align: match self.align.as_deref() {
                Some("left") => TextAlign::Left,
                Some("right") => TextAlign::Right,
                _ => TextAlign::Center,
            },
            opacity: self.opacity.unwrap_or(1.0).clamp(0.0, 1.0),
            stroke_width: self.stroke_width.unwrap_or(0.0).max(0.0),
            stroke_color: self.stroke_color.unwrap_or(base.stroke_color),
            shadow: self.shadow.unwrap_or(true),
            background: self.background.unwrap_or_default(),
            background_radius: self
                .background_radius
                .unwrap_or(base.background_radius)
                .max(0.0),
            background_padding_x: self
                .background_padding_x
                .unwrap_or(base.background_padding_x)
                .max(0.0),
            background_padding_y: self
                .background_padding_y
                .unwrap_or(base.background_padding_y)
                .max(0.0),
            line_height: self.line_height.unwrap_or(base.line_height).max(0.5),
            tracking: self.tracking.unwrap_or(0.0),
            max_width: self.max_width.unwrap_or(0.0).max(0.0),
            max_height: self.max_height.unwrap_or(0.0).max(0.0),
        }
    }
}

/// Where the user's presets live.
pub fn dir(dirs: &AppDirs) -> PathBuf {
    dirs.config.join("text-presets")
}

/// The presets that ship with the engine: the files under text-presets/,
/// embedded at build time (see build.rs), read exactly as a user's are.
/// In the order their files ask for, then by name.
pub fn builtin() -> Vec<TextPreset> {
    mod embedded {
        include!(concat!(env!("OUT_DIR"), "/text_presets.rs"));
    }
    let mut presets: Vec<(i64, TextPreset)> = embedded::BUILTIN
        .iter()
        .filter_map(|text| parse(text, None))
        .collect();
    presets.sort_by(|(a_order, a), (b_order, b)| {
        a_order.cmp(b_order).then_with(|| a.name.cmp(&b.name))
    });
    presets.into_iter().map(|(_, preset)| preset).collect()
}

/// The presets in the user's folder. A file that does not parse is
/// skipped without discarding the other presets.
pub fn user(dirs: &AppDirs) -> Vec<TextPreset> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir(dirs)) else {
        return found;
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter_map(|path| {
            if path.is_dir() {
                let inner = path.join("preset.toml");
                inner.is_file().then_some(inner)
            } else {
                (path.extension().and_then(|e| e.to_str()) == Some("toml")).then_some(path)
            }
        })
        .collect();
    paths.sort();
    for path in paths {
        if let Some(preset) = read(&path) {
            found.push(preset);
        }
    }
    found
}

fn read(path: &Path) -> Option<TextPreset> {
    let text = std::fs::read_to_string(path).ok()?;
    parse(&text, path.parent()).map(|(_, preset)| preset)
}

/// One preset file's text, and where it sorts. `folder` is where a font
/// it names is looked for; a built-in preset has none and brings no font.
fn parse(text: &str, folder: Option<&Path>) -> Option<(i64, TextPreset)> {
    let file: PresetFile = toml::from_str(text).ok()?;
    let id = file.id.trim().to_owned();
    if id.is_empty() {
        return None;
    }
    let font = file
        .font
        .filter(|name| !name.trim().is_empty())
        .and_then(|name| folder.map(|folder| folder.join(name.trim())));
    let order = file.order.unwrap_or(i64::MAX);
    let language = file
        .language
        .map(|language| language.trim().to_owned())
        .filter(|language| !language.is_empty())
        .unwrap_or_else(|| "Latin".to_owned());
    let style = file.style.over(&file.name);
    let sample = file
        .sample
        .map(|sample| sample.trim().to_owned())
        .filter(|sample| !sample.is_empty())
        .unwrap_or_else(|| {
            if language.eq_ignore_ascii_case("latin") {
                "Aa".to_owned()
            } else {
                style
                    .content
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .take(2)
                    .collect()
            }
        });
    Some((
        order,
        TextPreset {
            name: if file.name.trim().is_empty() {
                id.clone()
            } else {
                file.name.trim().to_owned()
            },
            id,
            style,
            offset_y: file.offset_y,
            font,
            language,
            sample,
        },
    ))
}

/// Every available preset: the built-in ones, then the user's. A
/// user preset with a built-in id replaces it, which is how a look can be
/// overridden without a duplicate entry.
pub fn all(dirs: &AppDirs) -> Vec<TextPreset> {
    let mut presets = builtin();
    for preset in user(dirs) {
        match presets.iter().position(|held| held.id == preset.id) {
            Some(index) => presets[index] = preset,
            None => presets.push(preset),
        }
    }
    presets
}

/// Where the host keeps fonts: the ones presets brought, and imported font
/// files. A font here outlives the project
/// it was imported into - it is offered to every title, and registered on
/// a project the moment a title in it is set in the face.
pub fn fonts_dir(dirs: &AppDirs) -> PathBuf {
    dirs.config.join("fonts")
}

/// Copies a font file into [`fonts_dir`], unless one of that name is there
/// already, and says where it landed. None when it cannot be copied.
pub fn install_font_file(dirs: &AppDirs, source: &Path) -> Option<PathBuf> {
    let name = source.file_name()?;
    let fonts = fonts_dir(dirs);
    let installed = fonts.join(name);
    if !installed.is_file() {
        std::fs::create_dir_all(&fonts).ok()?;
        std::fs::copy(source, &installed).ok()?;
    }
    Some(installed)
}

/// Every family in [`fonts_dir`], with the file that carries it: available
/// alongside the bundled face and the system's, in the
/// order the files are found. A file that is not a font is passed over.
pub fn installed_fonts(dirs: &AppDirs) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(fonts_dir(dirs)) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let file = path.to_string_lossy().into_owned();
        for family in concat_text::families_in(&path) {
            out.push((family, file.clone()));
        }
    }
    out
}

/// Puts the preset's font where the host keeps fonts, if it is not there
/// already, and says how to register it: the family the style names and
/// the installed file. None for a preset that brings no font, or whose
/// file cannot be read.
pub fn install_font(dirs: &AppDirs, preset: &TextPreset) -> Option<(String, String)> {
    let source = preset.font.as_ref().filter(|path| path.is_file())?;
    let installed = install_font_file(dirs, source)?;
    let family = preset.style.font_family.trim().trim_matches('"').to_owned();
    if family.is_empty() {
        return None;
    }
    Some((family, installed.to_string_lossy().into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_position_and_relative_font_survive_loading() {
        let folder = Path::new("presets/custom");
        let (_, preset) = parse(
            r#"
            id = "custom.japanese"
            name = "Japanese title"
            language = "Japanese"
            font = "face.ttf"
            offsetY = 0.3
            [style]
            content = "日本語"
            fontFamily = "Custom Face"
            "#,
            Some(folder),
        )
        .expect("valid preset");
        assert_eq!(preset.language, "Japanese");
        assert_eq!(preset.sample, "日本");
        assert_eq!(preset.offset_y, Some(0.3));
        assert_eq!(preset.font, Some(folder.join("face.ttf")));
        assert_eq!(preset.style.font_family, "Custom Face");
    }

    #[test]
    fn invalid_presets_are_rejected_and_explicit_samples_are_preserved() {
        assert!(parse("not valid TOML", None).is_none());
        assert!(parse("id = '  '\nname = 'Empty'", None).is_none());
        let (_, preset) =
            parse("id = 'test'\nname = 'Test'\nsample = 'Custom'", None).expect("valid preset");
        assert_eq!(preset.sample, "Custom");
        assert_eq!(preset.language, "Latin");
        assert_eq!(preset.font, None);
    }

    #[test]
    fn built_in_ids_are_unique_and_the_plain_title_is_first() {
        let presets = builtin();
        assert_eq!(presets[0].id, "default");
        for (index, preset) in presets.iter().enumerate() {
            assert!(
                !presets[..index].iter().any(|held| held.id == preset.id),
                "{} twice",
                preset.id
            );
        }
    }

    /// A file under text-presets/ that does not parse would be skipped
    /// without a word, and its entry would simply be missing.
    #[test]
    fn every_shipped_preset_file_parses() {
        mod embedded {
            include!(concat!(env!("OUT_DIR"), "/text_presets.rs"));
        }
        for text in embedded::BUILTIN {
            assert!(parse(text, None).is_some(), "does not parse:\n{text}");
        }
        assert_eq!(builtin().len(), embedded::BUILTIN.len());
    }

    #[test]
    fn a_partial_style_lays_over_the_default() {
        let file: PresetFile = toml::from_str(
            r##"
            id = "t.red"
            name = "Red"
            offsetY = 0.3
            [style]
            color = "#ff0000"
            align = "left"
            "##,
        )
        .expect("parses");
        let style = file.style.over(&file.name);
        assert_eq!(style.color, "#ff0000");
        assert_eq!(style.align, TextAlign::Left);
        assert_eq!(style.content, "Red");
        assert_eq!(style.font_weight, TextStyle::default().font_weight);
        assert_eq!(file.offset_y, Some(0.3));
    }
}
