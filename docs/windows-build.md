# Isolated Windows development

Introduced for concat-song on **2026-10-07**; see [FORK-NOTICE.md](../FORK-NOTICE.md).

`scripts/with-engine-env.ps1` invokes an existing local toolchain without a GUI,
system PATH edits, downloads or installers. Its default cache is
`$env:LOCALAPPDATA/concat-song-build`; `-BuildRoot`, `-FfmpegDir` and
`-LibclangDir` can select another prepared location. Use a fresh PowerShell
process rather than dot-sourcing it into a long-lived shell.

The first local check used:

- Rust1.93.0 MSVC under `cargo/` and `rustup/`; official rustup installer SHA256
  `6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`.
- The existing Visual Studio C++ toolchain, discovered through `vswhere`.
- FFmpeg `ffmpeg-n8.1.3-14-g330caae0c1-win64-gpl-shared-8.1.zip` from BtbN
  release `autobuild-2026-10-06-13-06`, SHA256
  `751c56e0b63426426487ab4048031b0166281c59c0a7e33ef7dd7428495e1d8a`.
  Extract under `vendor/ffmpeg/` preserving the archive's enclosing directory.
- libclang18.1.1 Windows wheel from PyPI, SHA256
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
