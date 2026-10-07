# concat-song architecture

Modified for concat-song on **2026-10-07**; see [FORK-NOTICE.md](FORK-NOTICE.md).
The inherited engine remains; the conventional graphical/mobile application
and its update/packaging paths are removed.

## Dependency direction

```text
CLI / future protocol clients
            |
       server transports
            |
      concat-api dispatcher
            |
     concat-host sessions/jobs
         /       |        |
     project   export    media
        |      / | \       |
        |  effects render   |
        |     |     |       |
        +-----+-----+-------+-- core
```

`concat-speech` remains an optional consumer of host/media. `concat-text`
rasterizes titles; `concat-vision` supports image treatments. Neither requires
the removed Slint window. `concat-perf` measures selected engine paths.

The exact members are explicit in [src/Cargo.toml](src/Cargo.toml).
There is no GUI member, mobile activity or UI build profile.

## The edit and the rendered result

The saved document (`concat-project::model`) has string IDs and seconds.
Commands operate through `Editor`, which owns in-memory undo snapshots.
`Command::Batch` stages project changes and commits them together on success;
this does not guarantee strict requested-value acceptance, complete ID-mint
rollback or persistent retry safety.

Export flattens the document, resolves the core timeline and plans a frame at a
rational timeline instant. The plan carries contributing media, source times,
geometry and effects. The compositor draws it and the media layer encodes it.
GPU rendering is currently part of the retained engine, not a Slint dependency;
offscreen operation needs a usable hardware or software graphics adapter.

The internal `FramePlan` is a future provenance seam, not an exposed public
explanation endpoint. Source-time conversion belongs in the engine rather than
being reconstructed by each client.

## State, ownership and jobs

`concat-api` owns sessions. The socket `Hub` serializes requests. The host
supplies persistence, previews, exports and job slots. Explicit save is required;
closing/reopening does not preserve undo.

`OpenProjects` and export-slot coordination remain shared in-process hooks.
Each API starts with its own register; even a shared register does not provide
cross-process or complete multi-API exclusion. The current single-dispatcher
route serializes its own calls. A presentation layer should use that owner,
not assume a second mutable document is coordinated. Multi-writer protection,
revision-bound changes and durable idempotency remain follow-on work.

## Useful functions rescued from the GUI

- `concat_project::captions` builds caption commands without a window or native
  media library.
- `concat_host::text_presets` preserves title styles/custom-font loading and
  embeds the preset data. Font rasterization and licences remain in `concat-text`.

These preserve capabilities, not panes, settings dialogs, platform launchers,
menus or application updating.

## Boundaries

The request/response contract is `concat-api/src/message.rs`; edit semantics
live in `concat-project/src/commands/`. Transports carry that API rather than
inventing another edit model. Future agent receipts should report actual state
and material limits, not equate a successful envelope with a correct edit.

Authentication and configured write roots stay in the API/server. Socket
defaults are loopback and still authenticated. Off-machine deployment needs
appropriate secure transport and a source offer for the modified version.
Removing a GUI removes neither duty.

[Presentation hooks](docs/presentation-hooks.md) names retained entry points;
[the development arc](docs/agent-engine-foundation.md) separates the foundation
from unimplemented improvements.
