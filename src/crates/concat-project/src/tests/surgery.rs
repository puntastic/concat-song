// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 concat-song contributors
// Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.

//! Finite regressions for source-preserving interval edits, without media IO.

use super::*;
use crate::model::{Clip, KeyEase, KeyProperty, SpeedPoint};
use crate::speed::source_at;

fn curved() -> (Editor, String, String) {
    let (mut editor, media, id) = fixture();
    editor
        .apply(Command::TrimClip {
            clip_id: id.clone(),
            edge: TrimEdge::Start,
            delta: 2.0,
            ripple: false,
        })
        .unwrap();
    editor
        .apply(Command::SetClipSpeedCurve {
            clip_id: id.clone(),
            curve: Some(vec![
                SpeedPoint {
                    at: 0.0,
                    speed: 0.5,
                },
                SpeedPoint {
                    at: 0.4,
                    speed: 3.0,
                },
                SpeedPoint {
                    at: 1.0,
                    speed: 1.0,
                },
            ]),
        })
        .unwrap();
    for (at, value) in [(0.0, -30.0), (1.0, 180.0)] {
        editor
            .apply(Command::SetClipKey {
                clip_id: id.clone(),
                property: KeyProperty::Rotation,
                at,
                value,
                ease: KeyEase::IN_OUT,
            })
            .unwrap();
    }
    (editor, media, id)
}

fn same_window(old: &Clip, piece: &Clip, from: f64) {
    for step in 0..=200 {
        let t = f64::from(step) * piece.duration / 200.0;
        let expected = source_at(old, from + t).unwrap();
        let actual = source_at(piece, t).unwrap();
        assert!(
            (actual - expected).abs() < 1e-10,
            "source at {t}: {actual} != {expected}"
        );
        let expected = old.value_at(KeyProperty::Rotation, (from + t) / old.duration);
        let actual = piece.value_at(KeyProperty::Rotation, t / piece.duration);
        assert!(
            (actual - expected).abs() < 1e-8,
            "key at {t}: {actual} != {expected}"
        );
    }
    if let Some(points) = &piece.speed_curve {
        assert!((piece.speed - crate::speed::mean_of(points)).abs() < 1e-12);
    }
}

#[test]
fn repeated_nonlinear_splits_preserve_every_retained_interior() {
    let (mut editor, _, id) = curved();
    let whole = editor.project().active().clip(&id).unwrap().clone();
    let cut = whole.start + whole.duration * 0.37;
    let tail = editor
        .apply(Command::SplitClips {
            clip_ids: vec![id.clone()],
            time: cut,
        })
        .unwrap()
        .created_id
        .unwrap();
    editor
        .apply(Command::SplitClips {
            clip_ids: vec![tail],
            time: whole.start + whole.duration * 0.73,
        })
        .unwrap();
    for piece in &editor.project().active().clips {
        same_window(&whole, piece, piece.start - whole.start);
    }
    let document = editor.to_document(&settings());
    let reopened = Editor::from_document(&document).unwrap();
    for piece in &reopened.project().active().clips {
        same_window(&whole, piece, piece.start - whole.start);
    }
    assert!(editor.undo());
    assert!(editor.undo());
    assert_eq!(editor.project().active().clip(&id).unwrap(), &whole);
    assert!(editor.redo());
    for piece in &editor.project().active().clips {
        same_window(&whole, piece, piece.start - whole.start);
    }
}

#[test]
fn nonlinear_trims_and_extensions_preserve_the_old_interval_for_both_ripple_modes() {
    for edge in [TrimEdge::Start, TrimEdge::End] {
        for delta in [-0.25, 0.35] {
            for ripple in [false, true] {
                let (mut editor, _, id) = curved();
                let old = editor.project().active().clip(&id).unwrap().clone();
                editor
                    .apply(Command::TrimClip {
                        clip_id: id.clone(),
                        edge,
                        delta,
                        ripple,
                    })
                    .unwrap();
                let piece = editor.project().active().clip(&id).unwrap();
                let from = match edge {
                    TrimEdge::Start => old.duration - piece.duration,
                    TrimEdge::End => 0.0,
                };
                same_window(&old, piece, from);
                if ripple {
                    assert_eq!(piece.start, old.start);
                }
            }
        }
    }
}

#[test]
fn extending_a_curved_head_stops_at_source_zero_using_its_boundary_speed() {
    let (mut editor, _, id) = curved();
    let old = editor.project().active().clip(&id).unwrap().clone();
    let earliest = crate::speed::earliest_local_time(&old);
    editor
        .apply(Command::TrimClip {
            clip_id: id.clone(),
            edge: TrimEdge::Start,
            delta: -1000.0,
            ripple: true,
        })
        .unwrap();
    let piece = editor.project().active().clip(&id).unwrap();
    assert!(piece.source_start.abs() < 1e-12);
    assert!((piece.duration - (old.duration - earliest)).abs() < 1e-12);
    same_window(&old, piece, earliest);
}

#[test]
fn an_explicit_constant_curve_and_nearby_keys_do_not_need_rejection() {
    let (mut editor, _, id) = fixture();
    editor
        .apply(Command::SetClipSpeedCurve {
            clip_id: id.clone(),
            curve: Some(vec![
                SpeedPoint {
                    at: 0.0,
                    speed: 2.0,
                },
                SpeedPoint {
                    at: 1.0,
                    speed: 2.0,
                },
            ]),
        })
        .unwrap();
    for (at, value) in [(0.0, 0.0), (0.5001, 25.0), (1.0, 40.0)] {
        editor
            .apply(Command::SetClipKey {
                clip_id: id.clone(),
                property: KeyProperty::Rotation,
                at,
                value,
                ease: KeyEase::LINEAR,
            })
            .unwrap();
    }
    let old = editor.project().active().clip(&id).unwrap().clone();
    editor
        .apply(Command::SplitClips {
            clip_ids: vec![id],
            time: 2.5,
        })
        .unwrap();
    for piece in &editor.project().active().clips {
        same_window(&old, piece, piece.start - old.start);
    }
}

#[test]
fn an_unrepresentable_key_boundary_leaves_all_selected_clips_and_ids_untouched() {
    let (mut editor, media_id, bad) = fixture();
    for (at, value) in [(0.0, 0.0), (1.0, 1.0)] {
        editor
            .apply(Command::SetClipKey {
                clip_id: bad.clone(),
                property: KeyProperty::Opacity,
                at,
                value,
                ease: KeyEase([0.3, 3.0, 0.7, 3.0]),
            })
            .unwrap();
    }
    let track_id = editor.project().active().tracks[1].id.clone();
    let good = editor
        .apply(Command::AddClip {
            media_id,
            track_id,
            start: 0.0,
            ripple: false,
        })
        .unwrap()
        .created_id
        .unwrap();
    let mut project = editor.project().clone();
    let before = project.clone();
    let mut mint = crate::commands::IdMint::default();
    mint.adopt_project(&project);
    let mut expected_mint = mint.clone();
    let error = crate::commands::apply(
        &mut project,
        &mut mint,
        Command::SplitClips {
            clip_ids: vec![good, bad],
            time: 5.0,
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        crate::CommandError::CannotPreserveTiming { .. }
    ));
    assert_eq!(project, before);
    assert_eq!(mint.next("c"), expected_mint.next("c"));
}

#[test]
fn no_op_and_nonfinite_cuts_do_not_rewrite_curves_or_history() {
    let (mut editor, _, id) = curved();
    let before = editor.to_document(&settings());
    let start = editor.project().active().clip(&id).unwrap().start;
    assert!(
        !editor
            .apply(Command::SplitClips {
                clip_ids: vec![id.clone()],
                time: start
            })
            .unwrap()
            .applied
    );
    assert!(
        editor
            .apply(Command::SplitClips {
                clip_ids: vec![id],
                time: f64::NAN
            })
            .is_err()
    );
    assert_eq!(editor.to_document(&settings()), before);
}

#[test]
fn nonlinear_ripple_and_freeze_move_only_the_declared_lane() {
    let (mut editor, media_id, id) = curved();
    let old = editor.project().active().clip(&id).unwrap().clone();
    let next_start = old.start + old.duration;
    let other_track = editor.project().active().tracks[1].id.clone();
    let add = |track_id| Command::AddClip {
        media_id: media_id.clone(),
        track_id,
        start: next_start,
        ripple: false,
    };
    let next = editor
        .apply(add(old.track_id.clone()))
        .unwrap()
        .created_id
        .unwrap();
    let unrelated = editor.apply(add(other_track)).unwrap().created_id.unwrap();
    editor
        .apply(Command::TrimClip {
            clip_id: id.clone(),
            edge: TrimEdge::Start,
            delta: 0.25,
            ripple: true,
        })
        .unwrap();
    assert_eq!(
        editor.project().active().clip(&next).unwrap().start,
        next_start - 0.25
    );
    assert_eq!(
        editor.project().active().clip(&unrelated).unwrap().start,
        next_start
    );
    let moving = editor.project().active().clip(&id).unwrap().clone();
    let Command::AddMedia { mut item } = media("/freeze-fixture.jpg", 1.0, false) else {
        unreachable!()
    };
    item.kind = MediaKind::Image;
    item.duration = None;
    editor
        .apply(Command::FreezeFrame {
            clip_id: id,
            time: moving.start + moving.duration / 2.0,
            duration: Some(1.0),
            still: Some(item),
        })
        .unwrap();
    assert_eq!(
        editor.project().active().clip(&next).unwrap().start,
        next_start - 0.25 + 1.0
    );
    assert_eq!(
        editor.project().active().clip(&unrelated).unwrap().start,
        next_start
    );
}

#[test]
fn a_video_freeze_without_a_still_remains_a_no_op() {
    let (mut editor, _, id) = fixture();
    for (at, value) in [(0.0, 0.0), (1.0, 1.0)] {
        editor
            .apply(Command::SetClipKey {
                clip_id: id.clone(),
                property: KeyProperty::Opacity,
                at,
                value,
                ease: KeyEase([0.3, 3.0, 0.7, 3.0]),
            })
            .unwrap();
    }
    let before = editor.to_document(&settings());
    assert!(
        !editor
            .apply(Command::FreezeFrame {
                clip_id: id,
                time: 5.0,
                duration: Some(1.0),
                still: None
            })
            .unwrap()
            .applied
    );
    assert_eq!(editor.to_document(&settings()), before);
}

#[test]
fn split_then_merge_retains_incoming_easing_for_clip_and_effect_keys() {
    use crate::model::AppliedFilter;
    let (mut editor, _, id) = fixture();
    editor
        .apply(Command::UpdateClip {
            clip_id: id.clone(),
            patch: ClipPatch {
                video_effects: Some(vec![AppliedFilter::new("concat.vignette")]),
                ..Default::default()
            },
        })
        .unwrap();
    for (at, value) in [(0.0, 0.0), (0.4001, 120.0), (1.0, 80.0)] {
        editor
            .apply(Command::SetClipKey {
                clip_id: id.clone(),
                property: KeyProperty::Rotation,
                at,
                value,
                ease: KeyEase::IN_OUT,
            })
            .unwrap();
        editor
            .apply(Command::SetEffectKey {
                clip_id: id.clone(),
                entry: 0,
                key: "strength".to_owned(),
                at,
                value,
                ease: KeyEase::IN,
            })
            .unwrap();
    }
    let whole = editor.project().active().clip(&id).unwrap().clone();
    let tail = editor
        .apply(Command::SplitClips {
            clip_ids: vec![id.clone()],
            time: 4.0,
        })
        .unwrap()
        .created_id
        .unwrap();
    editor
        .apply(Command::MergeClips {
            clip_ids: vec![id.clone(), tail],
        })
        .unwrap();
    let joined = editor.project().active().clip(&id).unwrap();
    for step in 0..=400 {
        let x = f64::from(step) / 400.0;
        assert!(
            (whole.value_at(KeyProperty::Rotation, x) - joined.value_at(KeyProperty::Rotation, x))
                .abs()
                < 1e-8,
            "clip easing changed at {x}"
        );
        assert!(
            (whole.video_effects[0].value_at("strength", x, 0.0)
                - joined.video_effects[0].value_at("strength", x, 0.0))
            .abs()
                < 1e-8,
            "effect easing changed at {x}"
        );
    }
    assert!(
        joined
            .keys_on(KeyProperty::Rotation)
            .any(|key| (key.at - 0.4001).abs() < 1e-12)
    );
}

#[test]
fn incompatible_shared_key_values_refuse_the_join_without_mutation() {
    let (mut editor, _, id) = fixture();
    for (at, value) in [(0.0, 0.0), (1.0, 120.0)] {
        editor
            .apply(Command::SetClipKey {
                clip_id: id.clone(),
                property: KeyProperty::Rotation,
                at,
                value,
                ease: KeyEase::IN_OUT,
            })
            .unwrap();
    }
    let tail = editor
        .apply(Command::SplitClips {
            clip_ids: vec![id.clone()],
            time: 4.0,
        })
        .unwrap()
        .created_id
        .unwrap();
    editor
        .apply(Command::SetClipKey {
            clip_id: tail.clone(),
            property: KeyProperty::Rotation,
            at: 0.0,
            value: 90.0,
            ease: KeyEase::LINEAR,
        })
        .unwrap();
    let before = editor.to_document(&settings());
    let error = editor
        .apply(Command::MergeClips {
            clip_ids: vec![id, tail],
        })
        .unwrap_err();
    assert!(matches!(
        error,
        crate::CommandError::CannotPreserveTiming { .. }
    ));
    assert_eq!(editor.to_document(&settings()), before);
}
