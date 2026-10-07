# concat-song

**An independently modified, agent-operated editing engine derived from
[Concat](https://github.com/jub0t/concat). Not an official Concat release.**

Modifications began **2026-10-07**. Read [FORK-NOTICE.md](FORK-NOTICE.md), the
dated [modification index](MODIFICATIONS.json), and the retained
[AGPL-3.0-or-later licence](LICENSE) with its [plugin exception](LICENSE-EXCEPTIONS.md).

## The target

Give software agents useful editing operations, inspectable state, media evidence
and recovery, while offloading repeatable mechanics into code. The output is for
people; the editing machinery is operated programmatically.

This fork removes the conventional desktop/mobile editor, its GUI-only
dependencies, installers and updater. It does not maintain a second human-operated
editor or promise GUI testing. [Presentation connection points](docs/presentation-hooks.md)
remain available for a future viewer or review surface without carrying that
application now.

## What is here

- A project document, typed edit commands, grouped operations and undo/redo.
- FFmpeg-backed media handling, an offscreen GPU compositor, effects, text,
  audio and export.
- A JSON-RPC API over stdio/TCP/Unix sockets; optional gRPC.
- Reusable title presets and caption-command helpers extracted from the former
  GUI. These are library capabilities, not automatically new API verbs.
- Optional speech/model capabilities retained for deliberate integration.

The first slice is a **headless foundation**, not a completed autonomous editor.
Inherited API success can still include clamped/no-op edits; undo is process-local;
a preview frame is not sound/motion review. Native interval export, strict edit
receipts and durable retries remain development targets, not completed features.

## Start here

1. [Engine layout and build](src/README.md)
2. [Architecture](ARCHITECTURE.md)
3. [Future presentation boundary](docs/presentation-hooks.md)
4. [Contributing and checks](CONTRIBUTING.md)
5. [Modification notices and source delivery](docs/licensing-process.md)
6. [Current development arc](docs/agent-engine-foundation.md)
7. [Inherited editing-semantic limits](docs/known-engine-limits.md)

The executable/package name currently remains `concat-cli` for source compatibility:

```sh
cd src
cargo run --locked -p concat-cli -- api '{"method":"version"}'
cargo run --locked -p concat-cli -- probe /path/to/input.mp4
```

Building needs Rust **1.93.0**, FFmpeg development libraries and the native
dependencies in the build guide/CI. No prebuilt fork release is promised.
Upstream GUI installers do not install this fork.

## Scope and evidence

Source traces to upstream commit
`f1f2f3eab310a3220b2fcdb47531d3d3b85d6713`. History remains available for
attribution and comparison. Source code, passed tests, exposed API operations,
media quality and long-recording performance are different evidence.

Human creative direction, media review and feedback belong in the surrounding
collaboration; they do not require a manual timeline editor here.
Independently implemented integrations use documented interfaces. Copied engine
code remains part of the covered fork.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [TRADEMARK.md](TRADEMARK.md).
Repository naming is not upstream endorsement or clearance for a future product
brand. Binary distribution and remote deployment need the matching source and
notice arrangements in the licensing process.
