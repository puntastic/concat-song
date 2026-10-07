# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 concat-song contributors
# Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.
<#
.SYNOPSIS
Run Cargo with existing, isolated Windows engine-build dependencies.
.DESCRIPTION
This does not download dependencies, install software, edit global PATH or
launch a GUI. Run in a fresh PowerShell process; environment changes are
process-local. See docs/windows-build.md for the expected cache layout.
#>
[CmdletBinding()]
param(
    [string]$BuildRoot = (Join-Path $env:LOCALAPPDATA 'concat-song-build'),
    [string]$FfmpegDir,
    [string]$LibclangDir,
    [switch]$CoreOnly,
    [string[]]$CargoArgs = @('check', '--locked', '--workspace', '--exclude', 'concat-speech', '--all-targets')
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$env:CARGO_HOME = Join-Path $BuildRoot 'cargo'
$env:RUSTUP_HOME = Join-Path $BuildRoot 'rustup'
$env:CARGO_TARGET_DIR = Join-Path $BuildRoot 'target'
$env:CARGO_BUILD_JOBS = '2'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$cargo = Join-Path $env:CARGO_HOME 'bin\cargo.exe'
if (-not (Test-Path -LiteralPath $cargo -PathType Leaf)) {
    throw "Missing isolated Cargo: $cargo. No installation was attempted."
}
$env:PATH = (Join-Path $env:CARGO_HOME 'bin') + ';' + $env:PATH

if (-not $CoreOnly) {
    if (-not $FfmpegDir) {
        $FfmpegDir = Join-Path $BuildRoot 'vendor\ffmpeg\ffmpeg-n8.1.3-14-g330caae0c1-win64-gpl-shared-8.1'
    }
    if (-not $LibclangDir) {
        $LibclangDir = Join-Path $BuildRoot 'vendor\libclang-18.1.1\libclang-18.1.1.data\platlib\clang\native'
    }
    foreach ($required in @((Join-Path $FfmpegDir 'include\libavcodec\avcodec.h'), (Join-Path $LibclangDir 'libclang.dll'))) {
        if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
            throw "Missing native input: $required. No download or installation was attempted."
        }
    }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) {
        throw 'Visual Studio discovery is unavailable; install/select a C++ toolchain separately.'
    }
    $vs = (& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | Select-Object -First 1)
    if (-not $vs) { throw 'No usable Visual Studio C++ toolchain was discovered.' }
    Import-Module (Join-Path $vs 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll')
    Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64' | Out-Null
    $env:FFMPEG_DIR = $FfmpegDir
    $env:LIBCLANG_PATH = $LibclangDir
    $env:PATH = (Join-Path $env:CARGO_HOME 'bin') + ';' + (Join-Path $FfmpegDir 'bin') + ';' + (Join-Path $vs 'Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin') + ';' + $env:PATH
}

Push-Location (Join-Path $repo 'src')
try {
    & $cargo +1.93.0 @CargoArgs
    $code = $LASTEXITCODE
    if ($code -ne 0) { throw "Cargo exited $code." }
} finally {
    Pop-Location
}
