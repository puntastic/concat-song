# Agent-engine foundation arc

Started **2026-10-07**, from upstream
`f1f2f3eab310a3220b2fcdb47531d3d3b85d6713`.
This is the current implementation slice, not a whole-editor roadmap.

## Result sought

A buildable, testable engine/API workspace with the conventional GUI removed,
useful media functions preserved, future presentation seams documented, and
modification notices part of ordinary work.

Repeatable mechanics belong in code. Instances supply intent, interpretation
and judgments the tools cannot establish without repeatedly reconstructing
timing, source identity or state bookkeeping.

## Sequence and checks

1. Bind fork/base; preserve upstream history and legal notices.
2. Extract useful title/caption functions, then remove GUI crates, updater,
   branding assets, application packaging and GUI workflows.
3. Make the surviving Cargo graph explicit; retain dependency versions where
   possible and pin the Rust patch version.
4. Run static/notice checks and pure tests, then native engine/API tests with
   actual required libraries. An unavailable native input limits that check;
   static success is not a build.
5. Check the supported workspace has no GUI dependencies/paths. Keep the media
   renderer and font rasterizer, which are not GUI code.
6. Review the diff, record modifications and preserve exact source/check results.

Preset payloads and attribution must survive extraction. The lockfile should
lose unused GUI packages without an unrelated version refresh. Documentation
must distinguish actual API behavior, clamping/no-op outcomes, explicit saves
and process-local undo. The notice checker is not legal certification or proof
of a complete source offer.

## Next frontier

After the foundation works, prioritize strict outcomes/preconditions, faithful
short AV review, engine-owned source/time explanations and recoverable
checkpoints/retries. A concrete editing encounter should guide final interfaces.
No range export, durable retry or multimodal-runtime capability is claimed by
its appearance here.

Core fidelity, preservation or editability failures may change the approach.
Missing GUI convenience is not failure of this target. A future presentation
need can activate the retained seams without restoring a second product by default.

The first source-priorities received during this slice are nonlinear-retime
surgery and multistream reattachment. Their exact inherited behaviors and
discriminating cases are retained in [known-engine-limits.md](known-engine-limits.md),
not claimed repaired by removing the GUI.
