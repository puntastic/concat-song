// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors
// Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.

//! Speed curves: the arithmetic the commands share.
//!
//! A curve is a handful of `(at, speed)` points over the clip, joined by
//! straight lines; the engine's [`SpeedCurve`] turns them into a time map.

use concat_core::SpeedCurve;

use crate::model::{Clip, SpeedPoint};

/// The engine's curve for these points, or None when they make no curve.
pub fn curve_of(points: &[SpeedPoint]) -> Option<SpeedCurve> {
    let raw: Vec<(f64, f64)> = points
        .iter()
        .map(|point: &SpeedPoint| (point.at, point.speed))
        .collect();
    SpeedCurve::new(&raw)
}

/// Source seconds per timeline second over a whole clip with these points;
/// the constant rate's equivalent for a curve.
pub fn mean_of(points: &[SpeedPoint]) -> f64 {
    curve_of(points).map(|curve| curve.mean()).unwrap_or(1.0)
}

/// The clip's source position at a local timeline time, with constant
/// endpoint-speed continuation outside its current span. None means a
/// non-finite input/result; a negative result lies before the source.
pub fn source_at(clip: &Clip, local: f64) -> Option<f64> {
    if !local.is_finite() || !clip.duration.is_finite() || clip.duration <= 0.0 {
        return None;
    }
    let consumed = match clip.speed_curve.as_deref().and_then(curve_of) {
        Some(curve) => clip.duration * curve.consumed_extended(local / clip.duration),
        None => local * clip.speed,
    };
    let source = clip.source_start + consumed;
    source.is_finite().then_some(source)
}

/// The earliest local time available under endpoint-speed continuation.
pub fn earliest_local_time(clip: &Clip) -> f64 {
    let speed = clip
        .speed_curve
        .as_deref()
        .and_then(curve_of)
        .map_or(clip.speed, |curve| curve.start_speed());
    -clip.source_start / speed
}

/// A source- and keyframe-preserving piece of a clip's old local interval.
/// The caller owns its new timeline placement, identity, fades and transition.
/// No input is mutated if its interval is not finitely representable.
pub fn window_clip(clip: &Clip, from: f64, to: f64) -> Result<Clip, &'static str> {
    let duration = to - from;
    if !duration.is_finite() || duration <= 0.0 {
        return Err("The retained interval must have a finite, positive duration.");
    }
    let mut source = source_at(clip, from).ok_or("The retained source position is not finite.")?;
    let roundoff = 64.0 * f64::EPSILON * clip.source_start.abs().max(1.0);
    if source < -roundoff {
        return Err("The retained interval begins before the source.");
    }
    source = source.max(0.0);
    let mut piece = clip.clone();
    piece.rewindow_keys(clip.duration, from, to)?;
    if let Some(curve) = clip.speed_curve.as_deref().and_then(curve_of) {
        let window = curve
            .window(from / clip.duration, to / clip.duration)
            .ok_or(
                "The retained speed interval cannot be represented with finite, distinct points.",
            )?;
        piece.speed = window.mean();
        piece.speed_curve = Some(
            window
                .points()
                .iter()
                .map(|&(at, speed)| SpeedPoint { at, speed })
                .collect(),
        );
    }
    piece.source_start = source;
    piece.duration = duration;
    Ok(piece)
}
