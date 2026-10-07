// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors
// Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.

//! Caption construction without a pane, controller or transcription runtime.
//!
//! [`script_captions`] supplies heuristic reading durations for untimed text;
//! [`caption_clip`] also accepts source-aligned timing from a caller.
//! These helpers return data and commands; the caller decides when to apply them.

use crate::Command;
use crate::model::TextStyle;

/// A rough reading/speaking rate for the untimed script heuristic.
pub const CHARS_PER_SECOND: f32 = 14.0;

/// One caption as a title clip: its words, when and for how long, and
/// its look.
pub fn caption_clip(
    text: String,
    start: f64,
    duration: f64,
    look: (f64, f64),
    base: Option<&TextStyle>,
) -> Command {
    let (offset_y, font_size) = look;
    // The kept look where there is one - family, weight, colours, stroke,
    // shadow, background - at the caption's own size and place; the
    // bundled face otherwise.
    let style = match base {
        Some(base) => TextStyle {
            content: text,
            font_size,
            max_width: 0.0,
            max_height: 0.0,
            ..base.clone()
        },
        None => TextStyle {
            content: text,
            font_family: "Hanken Grotesk".to_owned(),
            font_size,
            font_weight: 600.0,
            ..TextStyle::default()
        },
    };
    Command::AddTextClip {
        track_id: None,
        above: true,
        start,
        style: Some(style),
        duration: Some(duration),
        offset_y: Some(offset_y),
    }
}

/// Longest a caption line gets before it is wrapped: about what two lines
/// of broadcast subtitle hold, and what a reader takes in at a glance.
const CAPTION_CHARS: usize = 42;

/// A script as caption lines, each with how long it stays up: a line's
/// reading time at [`CHARS_PER_SECOND`], held to one second at least so
/// a short word is not a flicker, and seven at most so a long line does
/// not hang. A line break in the script is a break the author asked for;
/// within a paragraph a sentence is a caption, and a long sentence wraps
/// at its words.
pub fn script_captions(text: &str) -> Vec<(String, f64)> {
    text.lines()
        .flat_map(sentences)
        .flat_map(|sentence| wrap_caption(&sentence))
        .map(|line| {
            let seconds =
                (line.chars().count() as f64 / f64::from(CHARS_PER_SECOND)).clamp(1.0, 7.0);
            (line, seconds)
        })
        .collect()
}

/// A paragraph's sentences. A full stop, question or exclamation mark ends
/// one when it is followed by space or by the end - so "3.5" and "e.g." hold
/// together - and the CJK marks end one on their own.
fn sentences(paragraph: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = paragraph.chars().peekable();
    while let Some(ch) = chars.next() {
        current.push(ch);
        let ends = match ch {
            '。' | '！' | '？' => true,
            '.' | '!' | '?' => chars.peek().is_none_or(|next| next.is_whitespace()),
            _ => false,
        };
        if ends {
            let sentence = current.trim();
            if !sentence.is_empty() {
                out.push(sentence.to_owned());
            }
            current.clear();
        }
    }
    let rest = current.trim();
    if !rest.is_empty() {
        out.push(rest.to_owned());
    }
    out
}

/// A sentence in lines of at most [`CAPTION_CHARS`], broken between words;
/// a word longer than a line, or a run of CJK with no spaces, is broken
/// where it must be.
fn wrap_caption(sentence: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_chars = 0;
    for word in sentence.split_whitespace() {
        let word_chars = word.chars().count();
        if line_chars > 0 && line_chars + 1 + word_chars > CAPTION_CHARS {
            lines.push(std::mem::take(&mut line));
            line_chars = 0;
        }
        if word_chars > CAPTION_CHARS {
            let mut piece = String::new();
            for ch in word.chars() {
                piece.push(ch);
                if piece.chars().count() == CAPTION_CHARS {
                    lines.push(std::mem::take(&mut piece));
                }
            }
            line = piece;
            line_chars = line.chars().count();
            continue;
        }
        if line_chars > 0 {
            line.push(' ');
            line_chars += 1;
        }
        line.push_str(word);
        line_chars += word_chars;
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caption_command_keeps_timing_position_and_the_selected_style() {
        let base = TextStyle {
            font_family: "Custom Face".to_owned(),
            color: "#123456".to_owned(),
            max_width: 0.4,
            max_height: 0.2,
            ..TextStyle::default()
        };
        let command = caption_clip("Words".to_owned(), 2.5, 1.75, (0.35, 0.05), Some(&base));
        let Command::AddTextClip {
            track_id,
            above,
            start,
            style: Some(style),
            duration,
            offset_y,
        } = command
        else {
            panic!("caption must produce an AddTextClip command");
        };
        assert_eq!(track_id, None);
        assert!(above);
        assert_eq!(start, 2.5);
        assert_eq!(duration, Some(1.75));
        assert_eq!(offset_y, Some(0.35));
        assert_eq!(style.content, "Words");
        assert_eq!(style.font_size, 0.05);
        assert_eq!(style.font_family, "Custom Face");
        assert_eq!(style.color, "#123456");
        assert_eq!(style.max_width, 0.0);
        assert_eq!(style.max_height, 0.0);
        assert_eq!(base.max_width, 0.4);
    }

    #[test]
    fn a_caption_without_a_selected_style_uses_the_bundled_face() {
        let Command::AddTextClip {
            style: Some(style), ..
        } = caption_clip("Hello".to_owned(), 0.0, 2.0, (0.0, 0.04), None)
        else {
            panic!("caption must carry a style");
        };
        assert_eq!(style.font_family, "Hanken Grotesk");
        assert_eq!(style.font_weight, 600.0);
        assert_eq!(style.font_size, 0.04);
    }

    #[test]
    fn long_unicode_runs_wrap_without_losing_characters() {
        let text = "字".repeat(90);
        let lines = script_captions(&text);
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines
                .iter()
                .map(|(line, _)| line.as_str())
                .collect::<String>(),
            text
        );
        assert!(
            lines
                .iter()
                .all(|(line, _)| line.chars().count() <= CAPTION_CHARS)
        );
    }

    #[test]
    fn script_captions_apply_as_one_undoable_engine_batch() {
        let mut start = 3.0;
        let commands = script_captions("First. Second!")
            .into_iter()
            .map(|(text, duration)| {
                let command = caption_clip(text, start, duration, (0.35, 0.05), None);
                start += duration;
                command
            })
            .collect();
        let mut editor = crate::Editor::new();
        editor
            .apply(Command::Batch { commands })
            .expect("valid caption commands");
        let clips = &editor.project().active().clips;
        assert_eq!(clips.len(), 2);
        assert!(
            clips
                .iter()
                .all(|clip| clip.kind == crate::model::ClipKind::Text)
        );
        assert_eq!(
            clips[0].text.as_ref().expect("title style").content,
            "First."
        );
        assert_eq!(
            clips[1].text.as_ref().expect("title style").content,
            "Second!"
        );
        assert!(editor.undo());
        assert!(editor.project().active().clips.is_empty());
        assert!(!editor.undo());
    }

    /// A script becomes one caption per sentence, a hand line break is
    /// kept, a long sentence wraps at its words, and each line is held for
    /// its reading time within one to seven seconds.
    #[test]
    fn a_script_is_cut_into_readable_lines() {
        let lines = script_captions(
            "Hello there. This is version 3.5, mind!\n\nA sentence that runs on for far \
             longer than a caption line has any business running on for. Ok?",
        );
        let text: Vec<&str> = lines.iter().map(|(line, _)| line.as_str()).collect();
        assert_eq!(
            text,
            [
                "Hello there.",
                "This is version 3.5, mind!",
                "A sentence that runs on for far longer",
                "than a caption line has any business",
                "running on for.",
                "Ok?",
            ]
        );
        assert!(
            lines
                .iter()
                .all(|(_, seconds)| (1.0..=7.0).contains(seconds))
        );
        assert_eq!(lines[0].1, 1.0);
        assert!(lines[2].1 > lines[0].1);
        assert!(script_captions("  \n ").is_empty());
        assert_eq!(script_captions("你好。再见！").len(), 2);
    }
}
