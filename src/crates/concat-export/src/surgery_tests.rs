// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 concat-song contributors
// Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.

//! Timing-plan tests: no decode, GPU or rendered-fidelity claim.

use super::*;

fn clip(start: f64, duration: f64, source_start: f64) -> ExportClip {
    ExportClip {
        path: "timing-fixture.mp4".to_owned(),
        source_start,
        media_width: Some(64),
        media_height: Some(64),
        ..ExportClip::blank(ClipKind::Video, start, duration, 0)
    }
}

fn source(clip: &ExportClip, local: f64) -> f64 {
    clip.source_start
        + SpeedCurve::new(&clip.speed_curve).map_or(local * clip.speed, |curve| {
            clip.duration * curve.consumed(local / clip.duration)
        })
}

fn keyed(clip: &mut ExportClip, property: &str) {
    clip.animation = vec![
        ExportKey {
            property: property.to_owned(),
            at: 0.0,
            value: 0.2,
            ease: linear_ease(),
        },
        ExportKey {
            property: property.to_owned(),
            at: 1.0,
            value: 0.9,
            ease: EASE_IN_OUT,
        },
    ];
}

#[test]
fn surgery_pre_roll_preserves_nonlinear_source_and_eased_key_interiors() {
    let mut incoming = clip(4.0, 4.0, 3.0);
    incoming.speed_curve = vec![(0.0, 0.5), (1.0, 2.0)];
    incoming.speed = 1.25;
    keyed(&mut incoming, "rotation");
    let old = incoming.clone();
    let mut clips = vec![clip(0.0, 4.0, 0.0), incoming];
    let cut = Cut {
        incoming: 1,
        outgoing: 0,
        kind: "cross-fade".to_owned(),
        duration: 1.0,
    };
    assert_eq!(
        pre_roll(&mut clips, &cut, 1.0 / 30.0).unwrap(),
        Some((1, 1.0))
    );
    let extended = &clips[1];
    assert_eq!(extended.source_start, 2.5);
    let old_track = animation_of(&old.animation).unwrap().rotation;
    let new_track = animation_of(&extended.animation).unwrap().rotation;
    for step in 0..=100 {
        let t = f64::from(step) * old.duration / 100.0;
        assert!((source(extended, t + 1.0) - source(&old, t)).abs() < 1e-10);
        assert!(
            (new_track.value_at((t + 1.0) / extended.duration, 0.0)
                - old_track.value_at(t / old.duration, 0.0))
            .abs()
                < 1e-10
        );
    }
}

#[test]
fn surgery_source_positions_are_not_rounded_to_the_output_frame_grid() {
    let mut clip = clip(0.0, 2.0, 1.234567);
    clip.speed = 1.5;
    let plan = preview_plan(&[clip], 64, 64, 30, 1, ColorSpace::Sdr);
    assert_eq!(plan.error(), None);
    let built = plan.built.as_ref().unwrap();
    for frame in 0..50 {
        let time = FrameRate::THIRTY.time_of_frame(frame);
        let layers = plan_frame(&built.timeline, time).layers;
        assert_eq!(layers.len(), 1);
        let expected = 1.234567 + time.as_f64() * 1.5;
        assert!((layers[0].source_time.as_f64() - expected).abs() < 1e-6);
    }
}

#[test]
fn surgery_fractional_split_keeps_the_same_render_plan_source_at_every_output_instant() {
    for curved in [false, true] {
        let mut whole = clip(0.0, 2.0, 1.234567);
        whole.speed = 1.5;
        if curved {
            whole.speed_curve = vec![(0.0, 0.5), (0.4, 3.0), (1.0, 1.0)];
        }
        let cut = 0.37337;
        let mut head = whole.clone();
        head.duration = cut;
        let mut tail = whole.clone();
        tail.start = cut;
        tail.duration = whole.duration - cut;
        tail.source_start = source(&whole, cut);
        if let Some(curve) = SpeedCurve::new(&whole.speed_curve) {
            for (piece, a, b) in [
                (&mut head, 0.0, cut / whole.duration),
                (&mut tail, cut / whole.duration, 1.0),
            ] {
                let window = curve.window(a, b).unwrap();
                piece.speed = window.mean();
                piece.speed_curve = window.points().to_vec();
            }
        }
        let before = preview_plan(&[whole], 64, 64, 30, 1, ColorSpace::Sdr);
        let after = preview_plan(&[head, tail], 64, 64, 30, 1, ColorSpace::Sdr);
        for frame in 0..60 {
            let time = FrameRate::THIRTY.time_of_frame(frame);
            let old = plan_frame(&before.built.as_ref().unwrap().timeline, time);
            let new = plan_frame(&after.built.as_ref().unwrap().timeline, time);
            assert_eq!(new.layers.len(), 1);
            // The f64 -> Rational boundary has microsecond resolution.
            assert!(
                (old.layers[0].source_time.as_f64() - new.layers[0].source_time.as_f64()).abs()
                    <= 2e-6
            );
        }
    }
}

#[test]
fn surgery_microsecond_endpoints_share_one_boundary_at_the_reviewers_split() {
    for (start, absolute_cut) in [(0.0000004, 0.0333336), (0.0000006, 0.0333334)] {
        let mut whole = clip(start, 2.0, 1.25);
        whole.speed_curve = vec![(0.0, 0.5), (1.0, 2.0)];
        whole.speed = 1.25;
        keyed(&mut whole, "rotation");
        let offset = absolute_cut - whole.start;
        let curve = SpeedCurve::new(&whole.speed_curve).unwrap();
        let mut head = whole.clone();
        let mut tail = whole.clone();
        head.duration = offset;
        tail.start = absolute_cut;
        tail.duration = whole.duration - offset;
        tail.source_start = source(&whole, offset);
        for (piece, from, to) in [
            (&mut head, 0.0, offset / whole.duration),
            (&mut tail, offset / whole.duration, 1.0),
        ] {
            let window = curve.window(from, to).unwrap();
            piece.speed_curve = window.points().to_vec();
            piece.speed = window.mean();
            piece.animation = window_animation(&whole.animation, from, to).unwrap();
        }
        let old = preview_plan(&[whole], 64, 64, 30, 1, ColorSpace::Sdr);
        let split = preview_plan(&[head, tail], 64, 64, 30, 1, ColorSpace::Sdr);
        for frame in 0..60 {
            let at = FrameRate::THIRTY.time_of_frame(frame);
            let before = plan_frame(&old.built.as_ref().unwrap().timeline, at);
            let after = plan_frame(&split.built.as_ref().unwrap().timeline, at);
            if before.layers.is_empty() {
                assert!(after.layers.is_empty());
                continue;
            }
            assert_eq!(
                after.layers.len(),
                1,
                "gap or duplicate ownership at frame {frame}"
            );
            // Endpoint clocks and source conversion have microsecond precision;
            // this is not equality to every arbitrary f64 timestamp.
            assert!(
                (before.layers[0].source_time.as_f64() - after.layers[0].source_time.as_f64())
                    .abs()
                    < 4e-6
            );
            assert!(
                (before.layers[0].transform.rotation - after.layers[0].transform.rotation).abs()
                    < 1e-5
            );
        }
    }
}

#[test]
fn surgery_absolute_endpoint_ties_ignore_only_floating_arithmetic_residue() {
    let cut: f64 = 0.0333335;
    let expected = Rational::new(33_334, 1_000_000);
    for value in [
        cut,
        f64::from_bits(cut.to_bits() - 1),
        f64::from_bits(cut.to_bits() + 1),
    ] {
        assert_eq!(resolve::endpoint_time(value), Some(expected));
    }
    assert_ne!(resolve::endpoint_time(cut - 0.0000001), Some(expected));
}

#[test]
fn surgery_audio_preserves_gain_shapes_and_exposes_its_tempo_approximation() {
    let mut clip = clip(0.0, 4.0, 1.25);
    clip.kind = ClipKind::Audio;
    clip.speed_curve = vec![(0.0, 0.5), (1.0, 2.0)];
    clip.speed = 1.25;
    clip.audio_stream = Some(3);
    clip.preserve_pitch = true;
    keyed(&mut clip, "volume");
    let gain = volume_track(&clip);
    let pieces = audio_pieces(&clip).unwrap();
    assert_eq!(pieces.len(), 40);
    for piece in pieces {
        assert_eq!(piece.stream, Some(3));
        assert!(piece.preserve_pitch);
        let from = piece.start - clip.start;
        assert!((piece.source_start - source(&clip, from)).abs() < 1e-10);
        for step in 0..=10 {
            let t = f64::from(step) * piece.duration / 10.0;
            let got_gain = piece.volume_curve.value_at(t / piece.duration, 0.0);
            let wanted_gain = gain.value_at((from + t) / clip.duration, 0.0);
            assert!((got_gain - wanted_gain).abs() < 1e-10);
            let linear_source = piece.source_start + t * piece.speed;
            // Exact constant-mean intervals approximate this linear speed
            // ramp with maximum source error slope * interval^2 / 8.
            let bound = (2.0 - 0.5) / clip.duration * piece.duration.powi(2) / 8.0;
            assert!((linear_source - source(&clip, from + t)).abs() <= bound + 1e-10);
        }
    }
}

#[test]
fn surgery_transition_failure_does_not_publish_half_resolved_state() {
    let mut incoming = clip(4.0, 4.0, 3.0);
    incoming.transition = Some(TransitionSpec {
        kind: "cross-fade".to_owned(),
        duration: 1.0,
    });
    incoming.animation = vec![
        ExportKey {
            property: "rotation".to_owned(),
            at: 0.0,
            value: f64::MAX,
            ease: linear_ease(),
        },
        ExportKey {
            property: "rotation".to_owned(),
            at: 1.0,
            value: -f64::MAX,
            ease: EASE_IN_OUT,
        },
    ];
    let mut clips = vec![clip(0.0, 4.0, 0.0), incoming];
    assert!(resolve_transitions(&mut clips, FrameRate::THIRTY).is_err());
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[1].start, 4.0);
    assert_eq!(clips[1].duration, 4.0);
    assert_eq!(clips[1].source_start, 3.0);
    assert_eq!(clips[1].track, 0);
    let plan = preview_plan(&clips, 64, 64, 30, 1, ColorSpace::Sdr);
    assert!(plan.error().is_some());
    assert!(preview_moments(&plan, 4.0, 3, false).is_empty());
}
