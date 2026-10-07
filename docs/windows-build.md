# Isolated Windows development

Introduced for concat-song on **2026-10-07**; see [FORK-NOTICE.md](../FORK-NOTICE.md).

`scripts/with-engine-env.ps1` invokes an existing local toolchain without a GUI,
system PATH edits, downloads or installers. Its default cache is
`$env:LOCALAPPDATA/concat-song-build`; `-BuildRoot`, `-FfmpegDir` and
`-LibclangDir` can select another prepared location. Use a fresh PowerShell
process rather than dot-sourcing it into a long-lived shell.

Compiler discovery and runtime DLL search are kept separate. The wrapper takes
the VS development environment's INCLUDE/LIB/toolchain variables but restores
the caller's PATH plus the explicitly selected Rust, FFmpeg and CMake paths.
In the first local validation, carrying the full VS/SDK PATH into rendering
caused access violations; the same render binary passed all 86 tests with the
runtime path restored. Several graphics-compiler DLL versions were present in
those SDK paths, so this is an environment repair, not evidence of a fixed
renderer algorithm or an isolated defect in one named DLL. Never edit the
machine-wide PATH to make this helper work.

A separate software-renderer defect was reproduced after the native build was
working: the original Gaussian shader completed at 16x16 but stalled at 18x18
and at the 480x270 effect-card size on Microsoft Basic Render Driver (WARP).
Its column loop now gets the unchanged across-pass height from the frame
uniform, retaining the reciprocal arithmetic, kernel, sampling and alpha
handling. The manifest's full-height assumption is tested. Software and default
adapter numerical oracles passed, and the entire software effect-card test
completed locally. This supports the scoped workaround, not a proven root cause
inside a particular Windows driver or compiler.

The same full-height/query pattern was then reproduced in Glow and Bloom Pulse
at tiny intermediate sizes. Their equivalent substitutions preserve the screen,
pulse, alpha and sampling formulas; manifest invariants and fixture/HDR/alpha
checks cover both software and default adapters. The complete package-defaults
software sweep now passes. The long scripted editing/export scenario also
completed locally on WARP with all output assertions retained. These local
results do not turn an earlier hosted timeout into a pass.

Windows CI runs the full effect-card test as a required, logged preflight.
Only a verified pass permits the remaining workspace suite to filter that
already-run test. A timeout, unavailable required adapter or failed preflight
leaves validation incomplete; it does not turn into a skip or a green remainder.

Hardware decoding is a separate capability from the hardware/software graphics
adapter. `HwDevice::platform_default()` selects a backend by operating system;
it does not probe whether a particular device can decode a particular stream.
The export preference test retains its complete picture/sound checks when the
decoder takes its documented software fallback, and reports the actual route.
Set `CONCAT_REQUIRE_HARDWARE_DECODE=1` for a run that must demonstrate hardware
decoding. `CONCAT_REQUIRE_GPU` still requires rendering capability; it does not
turn a software CI runner into a hardware-codec qualification machine.

Test-only `CONCAT_TEST_SOFTWARE_GPU`, `CONCAT_TEST_SOFTWARE_CARDS` and
`CONCAT_TEST_SOFTWARE_EXPORT` selectors allow the renderer, cached effect cards
and scripted export cases to be qualified on a software adapter even when the
host also has a physical GPU. Explicit software requests fail when no adapter
is available. They do not change production adapter selection. Export scenarios
log preparation, rendering and readback boundaries; Windows CI exposes output
as it happens so an unrelated long-running case cannot hide a completed failure.

The first local check used:

- Rust 1.93.0 MSVC under `cargo/` and `rustup/`; official rustup installer SHA256
  `6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`.
- The existing Visual Studio C++ toolchain, discovered through `vswhere`.
- FFmpeg `ffmpeg-n8.1.3-14-g330caae0c1-win64-gpl-shared-8.1.zip` from BtbN
  release `autobuild-2026-10-06-13-06`, SHA256
  `751c56e0b63426426487ab4048031b0166281c59c0a7e33ef7dd7428495e1d8a`.
  Extract under `vendor/ffmpeg/` preserving the archive's enclosing directory.
- libclang 18.1.1 Windows wheel from PyPI, SHA256
  `4dd2d3b82fab35e2bf9ca717d7b63ac990a3519c7e312f19fa8e86dcc712f7fb`.
  Extract under `vendor/libclang-18.1.1/`; no global Python installation needed.
- Lockfile-bound native packages downloaded by Cargo/build scripts, including
  the Windows ORT/DirectML provider. This is not an offline build guarantee.

Checksums here identify inspected downloads; the wrapper checks required input
paths but does not independently authenticate an arbitrary replacement cache.
Use the source release's checksum and inspect changed build inputs. Pinned
upstream archives may cease to be hosted; recover the same verified artifact or
deliberately qualify a replacement rather than silently switching to latest.

From the repository root in PowerShell:

```powershell
& ./scripts/with-engine-env.ps1 -CoreOnly -CargoArgs @('test', '--locked', '-p', 'concat-core', '-p', 'concat-project')
& ./scripts/with-engine-env.ps1 -CargoArgs @('check', '--locked', '--workspace', '--exclude', 'concat-speech', '--all-targets')
& ./scripts/with-engine-env.ps1 -CargoArgs @('test', '--locked', '-p', 'concat-host', 'text_presets::tests')
```

`CoreOnly` skips native-library setup, not Cargo's dependency checking. Do not
use it to make a media check appear supported without its actual dependencies.
The wrapper leaves build caches outside the source tree and caps parallel jobs
at two. Optional speech needs additional inputs and is not covered by these
examples. See the CI workflow for its separately selected lane.

For a second worktree, pass Cargo's `--target-dir` in `-CargoArgs` to keep its
build outputs separate. The wrapper sets `CARGO_TARGET_DIR` from `-BuildRoot`,
so a value inherited from the parent shell is not a separate-target override.
