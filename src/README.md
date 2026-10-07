# concat-song engine

Modified for concat-song on **2026-10-07**; see [FORK-NOTICE.md](../FORK-NOTICE.md).
This is the agent-operated workspace. The conventional GUI is removed.

## Build inputs

- Rust **1.93.0**, pinned in `rust-toolchain.toml`; retain the committed lockfile.
- FFmpeg 7+ development headers/import libraries. CI pins an exact FFmpeg8.1
  build and checksum. Set `FFMPEG_DIR` to its root.
- A native compiler/linker and libclang for bindings (`LIBCLANG_PATH`).
- ONNX Runtime for the enabled vision/inference path. Windows DirectML must
  match the locked `ort` dependency; a CPU-only substitute is not equivalent.
- A hardware/software graphics adapter for offscreen rendering. Linux CI uses
  Mesa/lavapipe and requires GPU tests.
- Audio-device libraries where needed by the host (ALSA on Linux).
- Optional speech also needs CMake/C++ and matching sherpa libraries.

Exact CI inputs are in [engine-native.yml](../.github/workflows/engine-native.yml).
Pinned downloads fail closed if missing or changed; that is not a guarantee of
indefinite upstream hosting. These instructions do not run a system installer.

```sh
cargo test --locked -p concat-core -p concat-project
cargo test --locked --workspace --exclude concat-speech
cargo run --locked -p concat-cli -- api '{"method":"version"}'
cargo run --locked -p concat-cli -- probe input.mp4
cargo run --locked -p concat-cli -- serve
cargo run --locked -p concat-cli --features grpc -- serve --grpc 127.0.0.1:7421
```

`serve` starts an authenticated service, not a prerequisite for stdio `api`.
Off-loopback operation requires an authorized deployment and secure transport.
Narrow write roots to the actual task workspace.

## Crates

| Crate | Responsibility |
| --- | --- |
| `concat-core` | Rational time, handles, frames and timeline; std only |
| `concat-project` | Saved model, commands, batches, undo and caption helpers |
| `concat-media` | FFmpeg probe, decode, samples and encode |
| `concat-render` | Frame planning and offscreen GPU composition |
| `concat-effects` | Effect/shader packages and parameters |
| `concat-text` | Title rasterization and licensed fonts |
| `concat-vision` | Masking/enhancement model support |
| `concat-export` | Document resolution, render and mix/export |
| `concat-host` | Sessions, persistence, jobs, media support and title presets |
| `concat-api` | Structured requests, responses and events |
| `concat-server` | JSON-RPC and optional gRPC |
| `concat-cli` | Programmatic executable |
| `concat-speech` | Optional transcription/synthesis libraries |
| `concat-perf` | Selected engine performance checks |

Retained speech libraries and extracted caption/preset helpers are not
automatically new public API verbs. Inspect the current request enum.

## Protocol and evidence

[Request](crates/concat-api/src/message.rs) is the implementation contract.
JSON-RPC uses one object per line; `project.get`/`project.document` expose state,
`edit.apply` changes it, and `project.save` persists it. Export starts a job with
events. `preview.frame` returns a composited still, not video/audio.

```jsonl
{"jsonrpc":"2.0","id":1,"method":"version"}
{"jsonrpc":"2.0","id":2,"method":"project.open","params":{"path":"/edits/example"}}
{"jsonrpc":"2.0","id":3,"method":"project.get","params":{"path":"/edits/example"}}
```

Inherited commands may clamp values or tolerate absent targets. Inspect the
result and save deliberately. In-memory undo, project save, retry safety and
media-quality verification are different capabilities.

Routine CI covers engine/GPU/gRPC/wasm. Speech is a separately requested lane;
a routine pass does not mean it ran. Mobile dependency scripts remain source
utilities, not supported/tested mobile applications.

[Presentation hooks](../docs/presentation-hooks.md) identifies later viewer
entry points without retaining a GUI. Keep media/edit semantics below that
boundary. [CONTRIBUTING.md](../CONTRIBUTING.md) covers notices and development checks.
