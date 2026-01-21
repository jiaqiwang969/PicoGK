use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn extract_csharp_entrypoints(src: &str) -> BTreeSet<String> {
    // We only need the raw exported symbol strings, so a simple substring scan is enough.
    let marker = "EntryPoint = \"";
    let mut out = BTreeSet::new();
    let mut pos = 0usize;
    while let Some(idx) = src[pos..].find(marker) {
        let start = pos + idx + marker.len();
        if let Some(end_rel) = src[start..].find('"') {
            let end = start + end_rel;
            out.insert(src[start..end].to_string());
            pos = end + 1;
        } else {
            break;
        }
    }
    out
}

fn extract_rust_ffi_fns(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix("pub fn ") {
            // `pub fn Name(` ... ; take the identifier.
            let name = rest
                .split(|c: char| c == '(' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .trim();
            if !name.is_empty() {
                out.insert(name.to_string());
            }
        }
    }
    out
}

#[test]
fn rust_ffi_covers_csharp_entrypoints() {
    let csharp_path = Path::new("../PicoGK__Interop.cs");
    if !csharp_path.exists() {
        eprintln!(
            "Skipping: {} not found (repo-only parity test)",
            csharp_path.display()
        );
        return;
    }

    let rust_path = Path::new("src/ffi.rs");
    let csharp = fs::read_to_string(csharp_path).expect("read PicoGK__Interop.cs");
    let rust = fs::read_to_string(rust_path).expect("read src/ffi.rs");

    let csharp_eps = extract_csharp_entrypoints(&csharp);
    let rust_fns = extract_rust_ffi_fns(&rust);

    let missing: Vec<String> = csharp_eps.difference(&rust_fns).cloned().collect();

    assert!(
        missing.is_empty(),
        "Rust FFI missing {} C# entrypoints:\n{}",
        missing.len(),
        missing.join("\n")
    );
}
