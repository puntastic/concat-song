# Presentation can come later

Introduced for concat-song on **2026-10-07**; see [FORK-NOTICE.md](../FORK-NOTICE.md).

The current target is an instance-operated engine. A future human presentation
layer may display a result, comparison or selected state. Keep useful interfaces,
not the old manual editor and its untested dependencies.

| Need | Existing connection point | Boundary |
| --- | --- | --- |
| Identify build | `version`, capabilities | Declaration is not a behavior test |
| Inspect edit | `project.get`, `project.document` | Bind views to actual project/timeline/live or saved state |
| Request change | `edit.apply`, `edit.undo`, `edit.redo` | Use established session owner |
| Save/reopen | Project lifecycle API | Explicit save; undo does not survive reopening |
| Show picture | `preview.frame` | One composited still, not motion/audio |
| Show sequence | `export.run`, export events | Whole-timeline export; range review is future work |
| Observe progress | Cutout/export job events tagged with project paths | No project-change stream or durable replay/status guarantee |

The typed entry is `concat-api::Api::dispatch`; socket clients use
`concat-server`. `OpenProjects` and export-slot coordination remain shared
in-process hooks, not cross-process locking or a complete multi-writer protocol.
Future clients preserve authentication, root limits, source identity and
meaningful failure handling; concurrent mutation needs its own checked design.

A future layer consumes documented state/events and media artifacts, and routes
edits through the existing command path. Window types/event loops stay outside
engine crates. Source-relative time, retiming and rendering keep their current
owners rather than becoming presentation arithmetic.

These are real seams, not a promise of plug-and-play GUI support. Revision-bound
views, event recovery, source/time explanations and AV range previews must be
implemented/checked before reliance. Partial/stale views must remain distinguishable.

Human creative control can occur through media and dialogue; it is not manual
timeline operation. Add presentation for a concrete wanted use with its own
testing/maintenance scope.

Independently implemented clients follow the actual [licence exception](../LICENSE-EXCEPTIONS.md).
Internal-crate linkage/copied engine code is not independent merely because it
lives in another directory.
