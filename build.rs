// Build script for PicoGK Rust bindings
//
// This script handles linking to the PicoGK native library.
//
// This repo ships pre-compiled native binaries for macOS/Windows. On macOS and Windows we also
// copy the required runtime libraries next to Cargo-produced binaries so `cargo test` / `cargo run`
// work without extra environment variables.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_root = manifest_dir
        .parent()
        .expect("CARGO_MANIFEST_DIR should have a parent")
        .to_path_buf();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // docs.rs builds the docs on Linux without access to platform-specific native binaries.
    // Skip native linking in that environment so `cargo doc` can succeed there.
    println!("cargo:rerun-if-env-changed=PICOGK_NO_NATIVE");
    if env::var_os("PICOGK_NO_NATIVE").is_some() {
        println!("cargo:warning=PicoGK: PICOGK_NO_NATIVE=1, skipping native linking");
        return;
    }

    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if env::var_os("DOCS_RS").is_some() {
        println!("cargo:warning=PicoGK: DOCS_RS=1, skipping native linking");
        return;
    }

    // Determine platform and library path
    //
    // Override for all platforms:
    // - PICOGK_LIB_DIR: directory containing the native library
    // - PICOGK_LIB_NAME: base name without lib-prefix / extension (default: picogk.1.7)
    println!("cargo:rerun-if-env-changed=PICOGK_LIB_DIR");
    println!("cargo:rerun-if-env-changed=PICOGK_LIB_NAME");

    let lib_name = env::var("PICOGK_LIB_NAME").unwrap_or_else(|_| "picogk.1.7".to_string());
    let lib_path = if let Ok(dir) = env::var("PICOGK_LIB_DIR") {
        let p = PathBuf::from(dir);
        Some(if p.is_absolute() {
            p
        } else {
            project_root.join(p)
        })
    } else if target_os == "linux" {
        // Linux: we don't ship a native binary by default, but allow a repo-local drop-in
        // at `native/linux-x64` / `native/linux-arm64` before falling back to system paths.
        fn linux_dir_has_lib(dir: &PathBuf, lib_name: &str) -> bool {
            let expected = format!("lib{lib_name}.so");
            let entries = match fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => return false,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if name == expected {
                    return true;
                }
                if let Some(rest) = name.strip_prefix(&expected) {
                    if rest.starts_with('.') {
                        return true;
                    }
                }
            }
            false
        }

        let rel = match target_arch.as_str() {
            "x86_64" => Some(PathBuf::from("native/linux-x64")),
            "aarch64" => Some(PathBuf::from("native/linux-arm64")),
            _ => None,
        };

        let found = rel.and_then(|rel| {
            let candidate_project = project_root.join(&rel);
            if candidate_project.exists() && linux_dir_has_lib(&candidate_project, &lib_name) {
                return Some(candidate_project);
            }
            let candidate_manifest = manifest_dir.join(&rel);
            if candidate_manifest.exists() && linux_dir_has_lib(&candidate_manifest, &lib_name) {
                return Some(candidate_manifest);
            }
            None
        });

        if found.is_none() {
            println!("cargo:warning=PicoGK: Linux: PICOGK_LIB_DIR not set; relying on system linker search path for lib{lib_name}.so");
        }
        found
    } else {
        // Default search: this repo keeps `native/` at the project root, but allow using
        // the crate standalone by also checking for `native/` next to `Cargo.toml`.
        let rel = match (target_os.as_str(), target_arch.as_str()) {
            ("macos", "aarch64") => PathBuf::from("native/osx-arm64"),
            ("windows", "x86_64") => PathBuf::from("native/win-x64"),
            _ => {
                println!(
                    "cargo:warning=PicoGK: unsupported target {}-{} (set PICOGK_LIB_DIR/PICOGK_LIB_NAME to override)",
                    target_os, target_arch
                );
                std::process::exit(1);
            }
        };

        let candidate_project = project_root.join(&rel);
        Some(if candidate_project.exists() {
            candidate_project
        } else {
            manifest_dir.join(rel)
        })
    };

    println!("cargo:rerun-if-changed=build.rs");
    if let Some(ref lib_path) = lib_path {
        println!("cargo:rerun-if-changed={}", lib_path.display());
    }

    // Helper: locate `target/{profile}` from `OUT_DIR` (`.../target/{profile}/build/.../out`).
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("OUT_DIR should have at least 3 parents")
        .to_path_buf();

    // Platform-specific setup.
    if target_os == "macos" {
        let lib_path = lib_path.as_ref().expect("macOS should have a lib_path");
        // We do NOT patch the vendor dylib in-place. Modifying a signed dylib breaks its code
        // signature and can cause macOS to kill the process at runtime.
        //
        // Instead, we:
        // 1) Copy the dylibs into a build-local link directory (so we can provide the `lib*.dylib`
        //    name without touching the repo's `native/` folder).
        // 2) Copy the runtime dylibs next to Cargo-produced binaries (tests/examples) so the
        //    vendor install-name `@loader_path/...` resolves without requiring DYLD_LIBRARY_PATH.

        let link_dir = out_dir.join("picogk_native_link");
        if let Err(err) = fs::create_dir_all(&link_dir) {
            println!(
                "cargo:warning=PicoGK: failed to create link dir {}: {}",
                link_dir.display(),
                err
            );
        }

        let dylib_main = lib_path.join(format!("{}.dylib", lib_name));
        let dylib_liblzma = lib_path.join(format!("{}_liblzma.5.dylib", lib_name));
        let dylib_libzstd = lib_path.join(format!("{}_libzstd.1.dylib", lib_name));

        for src in [&dylib_main, &dylib_liblzma, &dylib_libzstd] {
            if !src.exists() {
                println!(
                    "cargo:warning=PicoGK: missing dylib {} (set PICOGK_LIB_DIR?)",
                    src.display()
                );
                continue;
            }

            let dst = link_dir.join(src.file_name().expect("dylib should have a file name"));
            // Best-effort copy; we avoid failing the build for transient filesystem issues.
            if !dst.exists() {
                if let Err(err) = fs::copy(src, &dst) {
                    println!(
                        "cargo:warning=PicoGK: failed to copy {} -> {}: {}",
                        src.display(),
                        dst.display(),
                        err
                    );
                }
            }
        }

        // For `-l{lib_name}`, the linker expects `lib{lib_name}.dylib`.
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let expected = link_dir.join(format!("lib{}.dylib", lib_name));
            if !expected.exists() {
                let target = PathBuf::from(format!("{}.dylib", lib_name));
                if let Err(err) = symlink(&target, &expected) {
                    println!(
                        "cargo:warning=PicoGK: failed to create symlink {} -> {}: {}",
                        expected.display(),
                        target.display(),
                        err
                    );
                }
            }
        }

        // Add library search path for linking.
        println!("cargo:rustc-link-search=native={}", link_dir.display());

        // Copy runtime dylibs next to Cargo-produced binaries.
        for rel in ["deps", "examples"] {
            let dir = profile_dir.join(rel);
            if let Err(err) = fs::create_dir_all(&dir) {
                println!(
                    "cargo:warning=PicoGK: failed to create runtime dir {}: {}",
                    dir.display(),
                    err
                );
                continue;
            }
            for src in [&dylib_main, &dylib_liblzma, &dylib_libzstd] {
                if !src.exists() {
                    continue;
                }
                let dst = dir.join(src.file_name().expect("dylib should have a file name"));
                if !dst.exists() {
                    if let Err(err) = fs::copy(src, &dst) {
                        println!(
                            "cargo:warning=PicoGK: failed to copy {} -> {}: {}",
                            src.display(),
                            dst.display(),
                            err
                        );
                    }
                }
            }
        }
    }

    if target_os == "windows" {
        let lib_path = lib_path.as_ref().expect("windows should have a lib_path");
        // Make `cargo test` / `cargo run --example ...` work out of the box by copying the
        // required DLLs next to Cargo-produced binaries. Windows searches the executable
        // directory first when resolving DLLs.
        for rel in ["deps", "examples"] {
            let dir = profile_dir.join(rel);
            if let Err(err) = fs::create_dir_all(&dir) {
                println!(
                    "cargo:warning=PicoGK: failed to create runtime dir {}: {}",
                    dir.display(),
                    err
                );
                continue;
            }

            let entries = match fs::read_dir(lib_path) {
                Ok(e) => e,
                Err(err) => {
                    println!(
                        "cargo:warning=PicoGK: failed to read native dir {}: {}",
                        lib_path.display(),
                        err
                    );
                    continue;
                }
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("dll"))
                {
                    let dst = dir.join(path.file_name().expect("dll should have a file name"));
                    if !dst.exists() {
                        if let Err(err) = fs::copy(&path, &dst) {
                            println!(
                                "cargo:warning=PicoGK: failed to copy {} -> {}: {}",
                                path.display(),
                                dst.display(),
                                err
                            );
                        }
                    }
                }
            }
        }
    }

    if target_os == "linux" {
        // Best-effort: if the user provides a `.so` via `PICOGK_LIB_DIR` (or a repo-local
        // `native/linux-*` drop-in), copy runtime libraries next to Cargo-produced binaries so
        // `cargo test` / `cargo run --example ...` can work without requiring `LD_LIBRARY_PATH`.
        if let Some(ref lib_path) = lib_path {
            for rel in ["deps", "examples"] {
                let dir = profile_dir.join(rel);
                if let Err(err) = fs::create_dir_all(&dir) {
                    println!(
                        "cargo:warning=PicoGK: failed to create runtime dir {}: {}",
                        dir.display(),
                        err
                    );
                    continue;
                }

                let entries = match fs::read_dir(lib_path) {
                    Ok(e) => e,
                    Err(err) => {
                        println!(
                            "cargo:warning=PicoGK: failed to read native dir {}: {}",
                            lib_path.display(),
                            err
                        );
                        continue;
                    }
                };

                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Match both `libfoo.so` and versioned `libfoo.so.1.2`.
                    let is_so = name.ends_with(".so") || name.contains(".so.");
                    if !is_so {
                        continue;
                    }
                    let dst = dir.join(path.file_name().expect(".so should have a file name"));
                    if !dst.exists() {
                        if let Err(err) = fs::copy(&path, &dst) {
                            println!(
                                "cargo:warning=PicoGK: failed to copy {} -> {}: {}",
                                path.display(),
                                dst.display(),
                                err
                            );
                        }
                    }
                }
            }
        }
    }

    if target_os != "macos" {
        // Add library search path for linking on other platforms when we have one.
        if let Some(ref lib_path) = lib_path {
            println!("cargo:rustc-link-search=native={}", lib_path.display());
        }
    }

    // Link to the library
    if target_os == "windows" {
        // Avoid requiring an import library (.lib). The DLL must be discoverable at runtime.
        println!("cargo:rustc-link-lib=raw-dylib={}", lib_name);
    } else {
        println!("cargo:rustc-link-lib=dylib={}", lib_name);
    }

    // Set rpath for macOS
    if target_os == "macos" {
        let lib_path = lib_path.as_ref().expect("macOS should have a lib_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
    }

    // On Linux, prefer discovering runtime `.so`s next to the produced binaries.
    if target_os == "linux" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }

    if let Some(ref lib_path) = lib_path {
        println!(
            "cargo:warning=Linking to PicoGK library at: {}",
            lib_path.display()
        );
    } else {
        println!("cargo:warning=Linking to PicoGK library via system linker search paths");
    }
}
