// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors
// Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.

//! Placement that changes over a clip.
//!
//! A clip's transform and opacity are one value each. An *animation* is a
//! set of keys on top of them: for each of scale, the two offsets, rotation
//! and opacity, a list of `(at, value)` with `at` a fraction of the clip's
//! length, eased between neighbours. The values are relative to the clip's
//! own - a scale key is a factor, an offset key an addition, an opacity key
//! a factor - so the same animation means the same motion on any clip, and
//! a preset never has to know where the clip sits.
//!
//! This is the one place the per-frame arithmetic lives; the plan asks the
//! clip, and the clip asks here.

use crate::timeline::Transform;

/// How a key is approached from the one before it: a CSS timing function,
/// as its two control points.
///
/// Four numbers rather than a handful of named shapes, because the named
/// shapes are four numbers each and a curve editor is not - the panel hands
/// people a bezier to drag, and the presets below are what its chips write.
/// The endpoints are pinned at (0,0) and (1,1), so only the middle two
/// points are stored, and `x` is clamped into `0..=1` where a legal timing
/// function keeps it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ease {
    /// First control point's x, `0..=1`.
    pub x1: f64,
    /// First control point's y; may overshoot.
    pub y1: f64,
    /// Second control point's x, `0..=1`.
    pub x2: f64,
    /// Second control point's y; may overshoot.
    pub y2: f64,
}

impl Default for Ease {
    fn default() -> Self {
        Ease::LINEAR
    }
}

impl Ease {
    /// A straight line.
    pub const LINEAR: Ease = Ease::new(0.0, 0.0, 1.0, 1.0);
    /// Starts slow, arrives fast. CSS `ease-in`.
    pub const IN: Ease = Ease::new(0.42, 0.0, 1.0, 1.0);
    /// Starts fast, arrives slow. CSS `ease-out`.
    pub const OUT: Ease = Ease::new(0.0, 0.0, 0.58, 1.0);
    /// Slow at both ends. CSS `ease-in-out`.
    pub const IN_OUT: Ease = Ease::new(0.42, 0.0, 0.58, 1.0);

    /// The four control-point numbers, as given.
    pub const fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Ease {
        Ease { x1, y1, x2, y2 }
    }

    /// Whether this is the straight line. What lets the
    /// mixer skip resampling a ride that has nothing to resample.
    pub fn is_linear(self) -> bool {
        self.x1 == self.y1 && self.x2 == self.y2
    }

    /// The eased fraction for a linear one.
    pub fn apply(self, t: f64) -> f64 {
        if self.is_linear() {
            return t.clamp(0.0, 1.0);
        }
        bezier_y_at_x(self.x1, self.y1, self.x2, self.y2, t)
    }
}

/// x of a cubic bezier with endpoints pinned at 0 and 1, at parameter `t`.
fn bezier_axis(p1: f64, p2: f64, t: f64) -> f64 {
    let u = 1.0 - t;
    3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t
}

fn bezier_axis_slope(p1: f64, p2: f64, t: f64) -> f64 {
    let u = 1.0 - t;
    3.0 * u * u * p1 + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}

/// Solve a CSS cubic-bezier for y at a given x: Newton first, bisection as
/// the fallback where the curve is flat enough that Newton stalls.
///
/// The one solver. The window's `Curves.ease` global reaches it through
/// `format::bezier_y_at_x`, because Slint's expression language has no loops
/// and so cannot do this itself; the engine reaches it through `Ease::apply`
/// on every frame of every keyed property. Two callers, one definition -
/// which matters here more than most, because a preview whose easing
/// disagreed with the export's would disagree invisibly.
pub fn bezier_y_at_x(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    bezier_axis(y1, y2, parameter_at_x(x1, x2, x))
}

// Bracket Newton steps so a flat derivative cannot send the solve outside
// the segment. The same parameter solve drives evaluation and interval cuts.
fn parameter_at_x(x1: f64, x2: f64, x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    if x == 0.0 || x == 1.0 {
        return x;
    }
    let (mut lo, mut hi, mut t) = (0.0, 1.0, x);
    for _ in 0..64 {
        let at = bezier_axis(x1, x2, t);
        if at == x {
            return t;
        }
        if at < x {
            lo = t;
        } else {
            hi = t;
        }
        let candidate = t - (at - x) / bezier_axis_slope(x1, x2, t);
        let next = if candidate > lo && candidate < hi {
            candidate
        } else {
            lo + (hi - lo) / 2.0
        };
        if next == t {
            break;
        }
        t = next;
    }
    t
}

type Point = (f64, f64);
type Cubic = [Point; 4];

fn split_cubic(p: Cubic, t: f64) -> (Cubic, Cubic) {
    let mix = |a: Point, b: Point| (a.0 * (1.0 - t) + b.0 * t, a.1 * (1.0 - t) + b.1 * t);
    let (a, b, c) = (mix(p[0], p[1]), mix(p[1], p[2]), mix(p[2], p[3]));
    let (d, e) = (mix(a, b), mix(b, c));
    let f = mix(d, e);
    ([p[0], a, d, f], [f, e, c, p[3]])
}

fn roots(a: f64, b: f64, c: f64) -> Vec<f64> {
    let mut out = Vec::new();
    if a == 0.0 {
        if b != 0.0 {
            out.push(-c / b);
        }
    } else {
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            let q = -0.5 * (b + disc.sqrt().copysign(b));
            if q == 0.0 {
                out.push(-b / (2.0 * a));
            } else {
                out.extend([q / a, c / q]);
            }
        }
    }
    out.retain(|t| t.is_finite() && *t > 0.0 && *t < 1.0);
    out.sort_by(f64::total_cmp);
    out
}

// Encode an actual (time,value) cubic in the existing CSS-ease key format.
// Restriction is exact subdivision, never a sampled/polyline replacement.
// Split at a derivative extremum when a restricted x control falls outside
// its endpoints, and at a value extremum when equal endpoints hide motion.
fn append_cubic(p: Cubic, from: f64, span: f64, out: &mut Vec<Key>, depth: usize) -> Option<()> {
    if depth > 32 || p.iter().any(|p| !p.0.is_finite() || !p.1.is_finite()) {
        return None;
    }
    let (dx, dy) = (p[3].0 - p[0].0, p[3].1 - p[0].1);
    if !dx.is_finite() || dx <= 0.0 || !dy.is_finite() {
        return None;
    }
    let flat = p.iter().all(|point| point.1 == p[0].1);
    let xs = [(p[1].0 - p[0].0) / dx, (p[2].0 - p[0].0) / dx];
    let margin = 8.0 * f64::EPSILON;
    let valid_x = xs.iter().all(|x| *x >= -margin && *x <= 1.0 + margin);
    if valid_x && (dy != 0.0 || flat) {
        let ease = if flat {
            Ease::LINEAR
        } else {
            Ease::new(
                xs[0].clamp(0.0, 1.0),
                (p[1].1 - p[0].1) / dy,
                xs[1].clamp(0.0, 1.0),
                (p[2].1 - p[0].1) / dy,
            )
        };
        if !ease.y1.is_finite() || !ease.y2.is_finite() {
            return None;
        }
        let at = (p[3].0 - from) / span;
        if !at.is_finite() || at <= out.last()?.at {
            return None;
        }
        out.push(Key {
            at,
            value: p[3].1,
            ease,
        });
        return Some(());
    }
    let axis = if !valid_x {
        p.map(|p| p.0)
    } else {
        p.map(|p| p.1)
    };
    let a = -axis[0] + 3.0 * axis[1] - 3.0 * axis[2] + axis[3];
    let b = 2.0 * (axis[0] - 2.0 * axis[1] + axis[2]);
    let c = axis[1] - axis[0];
    let split = if !valid_x {
        let vertex = -b / (2.0 * a);
        (vertex.is_finite() && vertex > 0.0 && vertex < 1.0).then_some(vertex)
    } else {
        roots(a, b, c).first().copied()
    }
    .unwrap_or(0.5);
    let (left, right) = split_cubic(p, split);
    append_cubic(left, from, span, out, depth + 1)?;
    append_cubic(right, from, span, out, depth + 1)
}

/// One key of one property.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Key {
    /// Where in the clip, `0..=1`.
    pub at: f64,
    /// The value there, in the property's relative terms.
    pub value: f64,
    /// How this key is approached from the previous one.
    pub ease: Ease,
}

/// The keys of one property, sorted by `at`.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Track {
    keys: Vec<Key>,
}

impl Track {
    /// A track through these keys, sorted.
    pub fn new(mut keys: Vec<Key>) -> Track {
        keys.retain(|key| key.at.is_finite() && key.value.is_finite());
        for key in &mut keys {
            key.at = key.at.clamp(0.0, 1.0);
        }
        keys.sort_by(|a, b| a.at.total_cmp(&b.at));
        Track { keys }
    }

    /// The keys, sorted.
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    /// True with no keys: the property is the clip's own.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Restrict/rebase this ride to `[from, to]`, with constant endpoint
    /// continuation outside its keys. Eased segments are subdivided rather
    /// than assigned their old easing over a shorter duration. No selection
    /// tolerance is used to discard nearby keys. None means an invalid span
    /// or a result that cannot be represented with finite, distinct keys.
    pub fn window(&self, from: f64, to: f64) -> Option<Track> {
        let span = to - from;
        if !from.is_finite() || !to.is_finite() || !span.is_finite() || span <= 0.0 {
            return None;
        }
        if self.is_empty() {
            return Some(self.clone());
        }
        let first = self.keys[0].at;
        // Retain an existing first key's time during extension: an extra
        // constant key at the new edge would be redundant, and could replace
        // a neighbouring piece's boundary key when the pieces are merged.
        let anchor = if first > from && first <= to {
            (first - from) / span
        } else {
            0.0
        };
        let mut out = vec![Key {
            at: anchor,
            value: self.value_at(from, 0.0),
            ease: Ease::LINEAR,
        }];
        for pair in self.keys.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if b.at == a.at {
                if a.at >= from && a.at < to {
                    out.push(Key {
                        at: (a.at - from) / span,
                        value: b.value,
                        ease: Ease::LINEAR,
                    });
                }
                continue;
            }
            let (lo, hi) = (from.max(a.at), to.min(b.at));
            if hi <= lo {
                continue;
            }
            let ease = b.ease;
            if [ease.x1, ease.y1, ease.x2, ease.y2]
                .iter()
                .any(|x| !x.is_finite())
                || !(0.0..=1.0).contains(&ease.x1)
                || !(0.0..=1.0).contains(&ease.x2)
            {
                return None;
            }
            let (dx, dy) = (b.at - a.at, b.value - a.value);
            let mut p = if ease.is_linear() {
                [
                    (a.at, a.value),
                    (a.at, a.value),
                    (b.at, b.value),
                    (b.at, b.value),
                ]
            } else {
                [
                    (a.at, a.value),
                    (a.at + dx * ease.x1, a.value + dy * ease.y1),
                    (a.at + dx * ease.x2, a.value + dy * ease.y2),
                    (b.at, b.value),
                ]
            };
            let solve = |x| {
                if ease.is_linear() {
                    parameter_at_x(0.0, 1.0, x)
                } else {
                    parameter_at_x(ease.x1, ease.x2, x)
                }
            };
            let (u, v) = (solve((lo - a.at) / dx), solve((hi - a.at) / dx));
            if v < 1.0 {
                p = split_cubic(p, v).0;
            }
            if u > 0.0 {
                p = split_cubic(p, u / v).1;
            }
            p[0].0 = lo;
            p[3].0 = hi;
            let start = Key {
                at: (lo - from) / span,
                value: p[0].1,
                ease: Ease::LINEAR,
            };
            // Explicit zero-width key pairs carry jumps above. A segment
            // beginning at an already emitted boundary needs no duplicate
            // key merely because the two evaluation paths round differently.
            if out.last().is_none_or(|last| last.at != start.at) {
                out.push(start);
            }
            append_cubic(p, from, span, &mut out, 0)?;
        }
        if out.last()?.at < 1.0 && to <= self.keys.last()?.at {
            out.push(Key {
                at: 1.0,
                value: self.value_at(to, 0.0),
                ease: Ease::LINEAR,
            });
        }
        if out
            .iter()
            .any(|key| !key.value.is_finite() || !(0.0..=1.0).contains(&key.at))
        {
            return None;
        }
        Some(Track { keys: out })
    }

    /// The value at `x`: `rest` with no keys, the first key's before it,
    /// the last key's after it, and eased between neighbours.
    pub fn value_at(&self, x: f64, rest: f64) -> f64 {
        let Some(first) = self.keys.first() else {
            return rest;
        };
        if x <= first.at {
            return first.value;
        }
        for pair in self.keys.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if x <= b.at {
                if b.at <= a.at {
                    return b.value;
                }
                let t = b.ease.apply((x - a.at) / (b.at - a.at));
                return a.value + (b.value - a.value) * t;
            }
        }
        self.keys[self.keys.len() - 1].value
    }

    /// The same ride, with every eased segment replaced by `steps` straight
    /// ones through the same points.
    ///
    /// For callers that can interpolate but cannot ease. The mixer is the
    /// one that needs it: a clip's gain ride becomes an FFmpeg `volume`
    /// expression, and an expression can lerp but cannot run the Newton
    /// solve a bezier wants. Sampling here rather than there keeps the
    /// easing in the one place that understands it, and leaves the
    /// expression generator with nothing but straight lines to write.
    ///
    /// Linear segments are passed through untouched, so a ride nobody has
    /// eased costs nothing and reads identically on both sides.
    pub fn resample(&self, steps: usize) -> Track {
        let steps = steps.max(1);
        if self.keys.len() < 2 || self.keys.iter().all(|key| key.ease.is_linear()) {
            return self.clone();
        }
        let mut out = vec![self.keys[0]];
        for pair in self.keys.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if b.ease.is_linear() || b.at <= a.at {
                out.push(Key {
                    ease: Ease::LINEAR,
                    ..b
                });
                continue;
            }
            for step in 1..=steps {
                let t = step as f64 / steps as f64;
                out.push(Key {
                    at: a.at + (b.at - a.at) * t,
                    value: a.value + (b.value - a.value) * b.ease.apply(t),
                    ease: Ease::LINEAR,
                });
            }
        }
        Track { keys: out }
    }
}

/// Every animatable property's keys.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Animation {
    /// A factor on the clip's scale; 1 is as placed.
    pub scale: Track,
    /// Added to the clip's horizontal offset, in frame widths.
    pub offset_x: Track,
    /// Added to the clip's vertical offset, in frame heights.
    pub offset_y: Track,
    /// Added to the clip's rotation, in degrees.
    pub rotation: Track,
    /// A factor on the clip's opacity; 1 is as set.
    pub opacity: Track,
    /// A factor on the clip's gain; 1 is as mixed. Sound rather than
    /// picture, but the same shape of fact - a value that changes over the
    /// clip - so it is a track here and not a second mechanism somewhere
    /// else. Nothing in the compositor reads it; the mixer does.
    pub volume: Track,
}

impl Animation {
    /// True when no property has a key.
    pub fn is_empty(&self) -> bool {
        self.scale.is_empty()
            && self.offset_x.is_empty()
            && self.offset_y.is_empty()
            && self.rotation.is_empty()
            && self.opacity.is_empty()
            && self.volume.is_empty()
    }

    /// The clip's transform at `x`, a fraction of its length.
    pub fn transform_at(&self, base: Transform, x: f64) -> Transform {
        Transform {
            scale: (base.scale * self.scale.value_at(x, 1.0)).max(0.001),
            offset_x: base.offset_x + self.offset_x.value_at(x, 0.0),
            offset_y: base.offset_y + self.offset_y.value_at(x, 0.0),
            rotation: base.rotation + self.rotation.value_at(x, 0.0),
            stretch_x: base.stretch_x,
            stretch_y: base.stretch_y,
        }
    }

    /// The clip's opacity at `x`.
    pub fn opacity_at(&self, base: f32, x: f64) -> f32 {
        (f64::from(base) * self.opacity.value_at(x, 1.0)).clamp(0.0, 1.0) as f32
    }

    /// The clip's gain at `x`. Unclamped at the top: a key above unity is a
    /// clip being lifted, which is the whole point of keying gain, and the
    /// mixer is where clipping is anyone's business.
    pub fn volume_at(&self, base: f64, x: f64) -> f64 {
        (base * self.volume.value_at(x, 1.0)).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_preserve_eased_interiors_and_endpoint_holds() {
        for ease in [
            Ease::LINEAR,
            Ease::IN,
            Ease::OUT,
            Ease::IN_OUT,
            Ease::new(1.0, 2.0, 0.0, -1.0),
            Ease::new(0.2, -2.0, 0.8, 3.0),
        ] {
            let track = Track::new(vec![key(0.1, -2.0, Ease::LINEAR), key(0.9, 4.0, ease)]);
            for (a, b) in [(0.0, 0.5), (0.333, 0.8), (0.5, 1.0), (-0.4, 1.2)] {
                let window = track
                    .window(a, b)
                    .unwrap_or_else(|| panic!("{ease:?}, {a}..{b}"));
                for step in 0..=200 {
                    let x = f64::from(step) / 200.0;
                    let want = track.value_at(a + (b - a) * x, 0.0);
                    let got = window.value_at(x, 0.0);
                    assert!(
                        (got - want).abs() < 1e-9,
                        "{ease:?}, {a}..{b}, at {x}: {got} != {want}"
                    );
                }
            }
        }
    }

    #[test]
    fn equal_endpoint_excursions_are_subdivided_not_flattened() {
        let p = [(0.0, 1.0), (1.0 / 3.0, 2.0), (2.0 / 3.0, 0.0), (1.0, 1.0)];
        let mut keys = vec![key(0.0, 1.0, Ease::LINEAR)];
        append_cubic(p, 0.0, 1.0, &mut keys, 0).unwrap();
        let track = Track::new(keys);
        assert!(track.keys().len() > 2);
        for step in 0..=100 {
            let t = f64::from(step) / 100.0;
            let want = split_cubic(p, t).0[3].1;
            assert!((track.value_at(t, 0.0) - want).abs() < 1e-10);
        }
    }

    #[test]
    fn window_keeps_steps_and_keys_near_a_cut() {
        let track = Track::new(vec![
            key(0.0, 0.0, Ease::LINEAR),
            key(0.5001, 2.0, Ease::LINEAR),
            key(0.5001, 3.0, Ease::LINEAR),
            key(1.0, 4.0, Ease::LINEAR),
        ]);
        for (a, b) in [(0.0, 0.5), (0.5001, 1.0), (0.4, 0.8)] {
            let window = track.window(a, b).unwrap();
            for step in 0..=100 {
                let x = f64::from(step) / 100.0;
                assert!(
                    (window.value_at(x, 0.0) - track.value_at(a + (b - a) * x, 0.0)).abs() < 1e-10
                );
            }
        }
        assert!(track.window(1.0, 0.0).is_none());
        assert!(track.window(f64::NAN, 1.0).is_none());
    }

    fn key(at: f64, value: f64, ease: Ease) -> Key {
        Key { at, value, ease }
    }

    #[test]
    fn a_track_holds_its_ends_and_eases_between() {
        let track = Track::new(vec![
            key(0.5, 1.0, Ease::LINEAR),
            key(0.0, 0.0, Ease::LINEAR),
        ]);
        assert_eq!(track.value_at(-1.0, 7.0), 0.0);
        assert_eq!(track.value_at(0.25, 7.0), 0.5);
        assert_eq!(track.value_at(0.9, 7.0), 1.0);
        assert_eq!(Track::default().value_at(0.5, 7.0), 7.0);

        let eased = Track::new(vec![key(0.0, 0.0, Ease::LINEAR), key(1.0, 1.0, Ease::OUT)]);
        assert!(eased.value_at(0.5, 0.0) > 0.5);
        let eased = Track::new(vec![key(0.0, 0.0, Ease::LINEAR), key(1.0, 1.0, Ease::IN)]);
        assert!(eased.value_at(0.5, 0.0) < 0.5);
    }

    #[test]
    fn an_animation_is_relative_to_the_clip() {
        let animation = Animation {
            scale: Track::new(vec![
                key(0.0, 0.5, Ease::LINEAR),
                key(1.0, 1.0, Ease::LINEAR),
            ]),
            offset_x: Track::new(vec![
                key(0.0, 0.5, Ease::LINEAR),
                key(1.0, 0.0, Ease::LINEAR),
            ]),
            opacity: Track::new(vec![
                key(0.0, 0.0, Ease::LINEAR),
                key(0.5, 1.0, Ease::LINEAR),
            ]),
            ..Animation::default()
        };
        let base = Transform {
            scale: 2.0,
            offset_x: 0.1,
            offset_y: -0.2,
            rotation: 10.0,
            stretch_x: 1.0,
            stretch_y: 1.0,
        };
        let mid = animation.transform_at(base, 0.5);
        assert!((mid.scale - 1.5).abs() < 1e-12);
        assert!((mid.offset_x - 0.35).abs() < 1e-12);
        assert_eq!(mid.offset_y, -0.2);
        assert_eq!(mid.rotation, 10.0);
        assert!((animation.opacity_at(0.8, 0.25) - 0.4).abs() < 1e-6);
        assert_eq!(animation.opacity_at(0.8, 0.9), 0.8);
        assert!(!animation.is_empty());
        assert!(Animation::default().is_empty());
    }

    #[test]
    fn the_eases_bend_the_way_their_names_say() {
        // The endpoints are pinned whatever the control points are.
        for ease in [Ease::LINEAR, Ease::IN, Ease::OUT, Ease::IN_OUT] {
            assert!((ease.apply(0.0) - 0.0).abs() < 1e-6, "{ease:?} at 0");
            assert!((ease.apply(1.0) - 1.0).abs() < 1e-6, "{ease:?} at 1");
        }
        // Only the straight line is straight, and it costs no solve.
        assert!(Ease::LINEAR.is_linear());
        assert_eq!(Ease::LINEAR.apply(0.37), 0.37);
        assert!(!Ease::IN.is_linear());

        // In starts slow, out starts fast, and in-out is symmetric about
        // the middle.
        assert!(Ease::IN.apply(0.5) < 0.5);
        assert!(Ease::OUT.apply(0.5) > 0.5);
        assert!((Ease::IN_OUT.apply(0.5) - 0.5).abs() < 1e-6);
        assert!(Ease::IN_OUT.apply(0.25) < 0.25);
        assert!(Ease::IN_OUT.apply(0.75) > 0.75);

        // Out of range is held, not extrapolated.
        assert_eq!(Ease::IN.apply(-1.0), 0.0);
        assert_eq!(Ease::IN.apply(2.0), 1.0);
    }

    #[test]
    fn resampling_follows_the_curve_it_flattens() {
        let track = Track::new(vec![
            key(0.0, 0.0, Ease::LINEAR),
            key(1.0, 4.0, Ease::IN_OUT),
        ]);
        let flat = track.resample(12);

        // Every segment of the flattened ride is straight, so a caller that
        // can only lerp - the mixer's filtergraph - reads it correctly.
        assert!(flat.keys().iter().all(|key| key.ease.is_linear()));
        assert!(flat.keys().len() > track.keys().len());

        // And it is the same ride, to a few thousandths of its range -
        // which on a gain ramp is a hundredth of a decibel.
        let range = 4.0;
        for step in 0..=40 {
            let x = f64::from(step) / 40.0;
            let error = (flat.value_at(x, 0.0) - track.value_at(x, 0.0)).abs();
            assert!(error < range * 0.005, "at {x}: off by {error}");
        }

        // A ride with nothing to flatten is handed back untouched, so the
        // common case costs neither keys nor expression length.
        let straight = Track::new(vec![
            key(0.0, 0.0, Ease::LINEAR),
            key(1.0, 1.0, Ease::LINEAR),
        ]);
        assert_eq!(straight.resample(12), straight);
    }
}
