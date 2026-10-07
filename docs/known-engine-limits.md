# Inherited semantics to qualify next

Recorded for concat-song on **2026-10-07**; see [FORK-NOTICE.md](../FORK-NOTICE.md).
These are source-visible limits inherited from upstream
`f1f2f3eab310a3220b2fcdb47531d3d3b85d6713`, with scoped repair status below.
Source inspection, plan-level regressions and rendered-media fidelity are
different evidence.

## Nonlinear retiming during surgery

The inherited `SplitClips` and freeze paths cleared speed curves and used
constant means for source boundaries. Keeping an aggregate source span did not
keep the source moment at each output time. The current
[source-preserving surgery repair](source-preserving-surgery.md) restricts the
existing time map and eased keys across split, trim, moving freeze pieces and
transition pre-roll. Merge still refuses varying-speed clips.

Pure and export-plan regressions exercise those mappings and explicit atomic
refusals. Do not infer whole-media fidelity from a valid document or these tests:
microsecond export clocks, stepped audio tempo and caller-supplied freeze stills
retain separate limits.

Next discriminator: compare actual rendered intervals and audio across the
tested operations, including boundaries, transitions and source offsets. Keep
constant-speed controls. The map/plan tests are already present; rendered-media
equivalence remains a separate check.

## Multistream reattachment

`ReattachAudio` in [audio.rs](../src/crates/concat-project/src/commands/audio.rs)
removes the detached sound clips, unmutes the picture clip and copies the first
sound clip's filters. It does not reconstruct every independently edited stream
or mix. Common provenance is not a complete synchronization/reattachment contract.

Next discriminator: detach two distinct streams, give them different gains,
filters and timing, then inspect reattach and undo. Compare both structure and
audible output; a simple unchanged one-stream case should remain supported.

## Other current boundaries

- Many edit commands tolerate absent IDs or clamp values; RPC success alone is
  not requested-value success.
- Batch stages the project, not all ID-mint state, and returns incomplete
  per-child creation information.
- Preview is one composited PNG; there is no native AV range-export request yet.
- Undo/jobs are process-local; request IDs do not establish durable idempotency.
- Existing shared ownership helpers are not cross-process/multi-API locking.

The next engine work should make these outcomes explicit and test the desired
invariants. This list is neither an exhaustive defect census nor a commitment
to implement every surrounding feature before another useful experiment.
