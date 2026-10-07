// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors
// Modified for concat-song on 2026-10-07; see FORK-NOTICE.md.

//! Embed reusable TOML title presets. No window assets or GUI compiler are needed.

fn main() {
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"))
        .join("text-presets");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&root)
        .expect("text-presets/ exists")
        .map(|entry| {
            entry
                .expect("read preset directory")
                .path()
                .join("preset.toml")
        })
        .filter(|file| file.is_file())
        .collect();
    files.sort();
    let mut table = String::from("pub static BUILTIN: &[&str] = &[\n");
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
        table.push_str(&format!(
            "    include_str!({:?}),\n",
            file.display().to_string()
        ));
    }
    table.push_str("];\n");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("out dir"))
        .join("text_presets.rs");
    std::fs::write(out, table).expect("write text_presets.rs");
}
