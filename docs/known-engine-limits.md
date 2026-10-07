# Inherited semantics to qualify next

Recorded for concat-song on **2026-10-07**; see [FORK-NOTICE.md](../FORK-NOTICE.md).
These are source-visible limits inherited from upstream
`f1f2f3eab310a3220b2fcdb47531d3d3b85d6713`, not failures reproduced with rendered
media in the GUI-removal slice. The relevant command files remain unchanged in
that slice. Source inspection and runtime fidelity are different evidence.

## Nonlinear retiming during surgery

`SplitClips` in [clips.rs](../src/crates/concat-project/src/commands/clips.rs)
clears a clip's speed curve before splitting and uses constant speed for the
resulting source boundaries. Freeze insertion also clears the curve; merge
refuses clips with curves. Keeping an aggregate source span is not the same as
keeping the source moment at each output time.

The trim path's constant-speed source shift likewise deserves a curve-fidelity
check. Do not infer that a valid saved document or successful edit preserved a
nonlinear time map. A source-preserving interface should preserve that map or
explicitly reject an unsupported operation before mutation.

Next discriminator: compare resolved source times throughout a nonconstant
curve before/after split and freeze, then compare actual rendered intervals.
Include trims, reverse/holds, keyframes, transitions and late source offsets.
Keep a constant-speed case that must still work.

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
