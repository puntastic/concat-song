<!-- Modified for concat-song on 2026-10-07; see FORK-NOTICE.md. -->
# Contributing to concat-song

This fork develops the headless editing engine, JSON API, transports, and CLI.
The upstream Slint editor and Android application have been removed. A future
presentation layer can call the engine/API; GUI maintenance is not part of the
current project. Preserve that separation when proposing changes.

Open an issue in this fork for substantial changes before investing in an
implementation. Useful reports include the exact revision, OS/architecture,
command or API requests, native-library versions, expected result, and observed
result. Share media only when you have permission, and prefer a small synthetic
reproducer over private footage or project paths.

## Build prerequisites

Read [the engine guide](src/README.md) for crate roles and native dependencies.
The workspace is under `src/`; `src/rust-toolchain.toml` pins Rust 1.93.
Keep `Cargo.lock` and use `--locked` for validation. Linux and Windows x86-64
MSVC are the current native CI targets. Builds need FFmpeg 7+ development
libraries and libclang; the host also requires ONNX Runtime and Linux ALSA.
Rendering is GPU-backed even without a window; Mesa lavapipe is used in Linux
CI. Optional speech adds CMake/C++ and whisper/sherpa native dependencies.

The exact CI setup and fixed native-download digests are in
[engine-native.yml](.github/workflows/engine-native.yml). Fixed URLs and digests
make input drift fail visibly; they do not guarantee upstream asset retention
or bit-for-bit builds. Windows ONNX/DirectML acquisition remains the locked
`ort-sys` crate's responsibility. No Nix package, installer, release upload,
model publication, or automatic distribution is currently provided.

From a configured environment:

```sh
cd src
cargo build --locked -p concat-cli
cargo run --locked -p concat-cli -- api '{"jsonrpc":"2.0","id":1,"method":"version"}'
```

Do not point API write roots or export commands at important work while
developing; use temporary project folders and synthetic media.

## Checks

Run the headless scope before opening a pull request:

```sh
cd src
cargo fmt --all --check
cargo clippy --locked --workspace --exclude concat-speech --all-targets -- -D warnings
cargo test --locked --workspace --exclude concat-speech -- --show-output
cargo clippy --locked -p concat-server -p concat-cli --features grpc --all-targets -- -D warnings
cargo test --locked -p concat-server --features grpc -- --show-output
cargo run --locked -p concat-perf --profile quick -- --check --quick
```

CI requires the Linux rendering tests to find an adapter by setting
`CONCAT_REQUIRE_GPU=1`. On other machines the suite can report adapter or codec
skips; read them, and do not describe a skipped path as tested. `--show-output`
keeps those messages visible. The quick performance check excludes the full
media benchmarking scenarios, which need a controlled machine.

The native-library-free portability subset remains checked with:

```sh
cargo clippy --locked -p concat-core -p concat-project -p concat-effects -p concat-render -p concat-text --features concat-render/gpu --target wasm32-unknown-unknown -- -D warnings
```

Speech remains in the workspace but is excluded explicitly from the routine
native lane to avoid its heavy dependencies on every engine change. For speech
changes, run the **Engine CI** workflow manually with `speech` enabled, or use
the equivalent checks in an environment with its native libraries configured:

```sh
cargo clippy --locked -p concat-speech --all-features --all-targets -- -D warnings
cargo test --locked -p concat-speech --all-features -- --show-output
```

That lane includes Chatterbox compilation/tests, not live model inference or
quality evaluation. Real audio devices, hardware encoders, downloaded model
weights, and macOS/mobile targets need separate, clearly reported validation.
The remaining `src/scripts/*-mobile.sh` and `mobile-env.sh` are unvalidated
engine-dependency utilities, not supported mobile builds or packaging targets.

From the repository root, also run:

```sh
python scripts/models.py --check
python -m unittest discover -s scripts/tests -p 'test_modification_notices.py'
python scripts/check_modification_notices.py check
```

Model changes must keep `models/manifest.toml` and their Rust tables consistent.
The read-only check does not download models or establish their availability.
The explicit `scripts/models.py mirror --out <directory>` utility downloads
models and may record missing digests in those tables; inspect all resulting
changes and update modification notices before committing. There is no model
release publisher in this fork.

Use the `log` facade for engine diagnostics. CLI/API standard output is a
machine-readable protocol surface, not a place for incidental diagnostics.

## Attribution and modification notices

The inherited code remains AGPL-3.0-or-later; retain [LICENSE](LICENSE),
[LICENSE-EXCEPTIONS.md](LICENSE-EXCEPTIONS.md), and existing attribution. Follow
[the fork modification process](docs/licensing-process.md), including dated
notices for changed commentable files and the central modification inventory.
Keep existing upstream SPDX and copyright lines unchanged; add the fork notice
after them rather than replacing them. Record third-party origins and licences
when introducing code or dependencies.

[CLA.md](CLA.md) and [TRADEMARK.md](TRADEMARK.md) are retained upstream records,
not a claim that this fork is operated by the upstream maintainers. The fork's
name does not transfer upstream branding or trademark rights. See
[FORK-NOTICE.md](FORK-NOTICE.md) for the fork's provenance and limits.

[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) remains the conduct reference.
