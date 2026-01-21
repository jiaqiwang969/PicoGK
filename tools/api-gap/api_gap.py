#!/usr/bin/env python3
"""
Generate a rough C# -> Rust API gap report for PicoGK.

Inputs:
  - picogk-rs/CSHARP_PUBLIC_API.md (generated via tools/api-dump)
  - picogk-rs/src/**/*.rs (Rust bindings)

Output:
  - picogk-rs/API_GAP_REPORT.md (by default)

This is a heuristic tool. It normalizes C# method names (Pascal/camel + Hungarian prefixes)
to snake_case and tries to match them against Rust `pub fn` names in inherent `impl Type {}` blocks.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Set, Tuple


IGNORED_CSHARP_METHODS: Set[str] = {
    "Dispose",
    "Equals",
    "GetHashCode",
    "GetType",
    # Callback interface method; Rust exposes traversal via closures instead.
    "InformActiveValue",
    "ToString",
}

# Some C# -> Rust type name differences.
TYPE_MAP: Dict[str, str] = {
    "OpenVdbFile": "VdbFile",
}


def camel_to_snake(name: str) -> str:
    # Reasonable camel/Pascal -> snake conversion, preserving acronym boundaries.
    out: List[str] = []
    for i, ch in enumerate(name):
        if ch.isupper():
            if i > 0:
                prev = name[i - 1]
                nxt = name[i + 1] if i + 1 < len(name) else ""
                if prev.islower() or (prev.isupper() and nxt.islower()):
                    out.append("_")
            out.append(ch.lower())
        else:
            out.append(ch)
    return "".join(out)


def csharp_method_candidates(cs_name: str) -> Set[str]:
    snake = camel_to_snake(cs_name)
    cands: Set[str] = {snake}

    # Hungarian prefixes used in PicoGK C#.
    for pref in (
        "vox_",
        "vec_",
        "msh_",
        "o_",
        "b_",
        "by_",
        "f_",
        "n_",
        "str_",
        "clr_",
        "img_",
    ):
        if snake.startswith(pref):
            cands.add(snake[len(pref) :])

    # Special casing used by some image helpers: `byGetValue` (byte) -> `byte_value`.
    if snake.startswith("by_") and snake.endswith("get_value"):
        cands.add("byte_value")

    # Common verb patterns.
    if snake.startswith("save_to_") and snake.endswith("_file"):
        mid = snake[len("save_to_") : -len("_file")]
        cands.add(f"save_{mid}")

    if snake.startswith("load_from_") and snake.endswith("_file"):
        mid = snake[len("load_from_") : -len("_file")]
        cands.add(f"load_{mid}")

    if "bounding_box" in snake:
        cands.add("bounding_box")

    if snake == "vox_from_vdb_file":
        cands.add("load_vdb")

    return cands


@dataclass(frozen=True)
class CSharpType:
    name: str  # e.g. "Voxels"
    methods: Set[str]


def parse_csharp_api_dump(path: Path) -> Dict[str, CSharpType]:
    text = path.read_text(encoding="utf-8")
    current: Optional[str] = None
    in_methods = False
    methods: Dict[str, Set[str]] = {}

    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith("## "):
            in_methods = False
            full = line[len("## ") :].strip()
            # Only keep PicoGK namespace types.
            if not full.startswith("PicoGK."):
                current = None
                continue
            type_name = full[len("PicoGK.") :]
            # Skip nested types (enums etc).
            if "+" in type_name:
                current = None
                continue
            current = type_name
            methods.setdefault(current, set())
            continue

        if current is None:
            continue

        if line == "### Methods":
            in_methods = True
            continue

        if line.startswith("### "):
            in_methods = False
            continue

        if not in_methods:
            continue

        # Method lines look like: - `Void BoolAdd(Voxels& voxOperand)`
        if not line.startswith("- `") or not line.endswith("`"):
            continue
        sig = line[3:-1]
        # Extract name: last token before '('.
        paren = sig.find("(")
        if paren < 0:
            continue
        before = sig[:paren].strip()
        parts = before.split()
        if not parts:
            continue
        name = parts[-1].strip()
        if name.startswith("_"):
            continue
        if name in IGNORED_CSHARP_METHODS:
            continue
        methods[current].add(name)

    return {k: CSharpType(name=k, methods=v) for k, v in methods.items()}


def strip_rust_comments_and_strings(code: str) -> str:
    # Produces a same-length string where characters inside comments/strings are replaced
    # with spaces, preserving newlines. This allows brace matching without being confused by
    # `{}` inside doc comments or format strings.
    out: List[str] = []
    i = 0
    n = len(code)

    mode = "normal"
    block_depth = 0
    raw_hashes = 0

    def emit(ch: str) -> None:
        out.append(ch)

    while i < n:
        ch = code[i]
        nxt = code[i + 1] if i + 1 < n else ""

        if mode == "line_comment":
            if ch == "\n":
                mode = "normal"
                emit("\n")
            else:
                emit(" ")
            i += 1
            continue

        if mode == "block_comment":
            if ch == "\n":
                emit("\n")
                i += 1
                continue
            if ch == "/" and nxt == "*":
                block_depth += 1
                emit(" ")
                emit(" ")
                i += 2
                continue
            if ch == "*" and nxt == "/":
                block_depth -= 1
                emit(" ")
                emit(" ")
                i += 2
                if block_depth <= 0:
                    mode = "normal"
                    block_depth = 0
                continue
            emit(" ")
            i += 1
            continue

        if mode == "string":
            if ch == "\n":
                emit("\n")
                i += 1
                continue
            if ch == "\\":
                emit(" ")
                if i + 1 < n:
                    emit(" ")
                    i += 2
                else:
                    i += 1
                continue
            if ch == '"':
                mode = "normal"
                emit(" ")
                i += 1
                continue
            emit(" ")
            i += 1
            continue

        if mode == "raw_string":
            if ch == "\n":
                emit("\n")
                i += 1
                continue
            if ch == '"':
                # Potential end: '"' + hashes.
                ok = True
                for k in range(raw_hashes):
                    if i + 1 + k >= n or code[i + 1 + k] != "#":
                        ok = False
                        break
                if ok:
                    emit(" ")
                    for _ in range(raw_hashes):
                        emit(" ")
                    i += 1 + raw_hashes
                    mode = "normal"
                    continue
            emit(" ")
            i += 1
            continue

        if mode == "char":
            if ch == "\n":
                # Should not happen for valid char literals; fall back.
                mode = "normal"
                emit("\n")
                i += 1
                continue
            if ch == "\\":
                emit(" ")
                if i + 1 < n:
                    emit(" ")
                    i += 2
                else:
                    i += 1
                continue
            if ch == "'":
                mode = "normal"
                emit(" ")
                i += 1
                continue
            emit(" ")
            i += 1
            continue

        # normal
        if ch == "/" and nxt == "/":
            mode = "line_comment"
            emit(" ")
            emit(" ")
            i += 2
            continue
        if ch == "/" and nxt == "*":
            mode = "block_comment"
            block_depth = 1
            emit(" ")
            emit(" ")
            i += 2
            continue

        # raw strings: r#"..."# or br#"..."#
        if ch in ("r", "b"):
            start = i
            pref_len = 0
            if ch == "r":
                pref_len = 1
            elif ch == "b" and nxt == "r":
                pref_len = 2
            if pref_len:
                j = i + pref_len
                k = 0
                while j < n and code[j] == "#":
                    k += 1
                    j += 1
                if j < n and code[j] == '"':
                    # start raw string
                    mode = "raw_string"
                    raw_hashes = k
                    for _ in range(j - start + 1):
                        emit(" ")
                    i = j + 1
                    continue

        # normal strings: "..." or b"..."
        if ch == "b" and nxt == '"':
            mode = "string"
            emit(" ")
            emit(" ")
            i += 2
            continue
        if ch == '"':
            mode = "string"
            emit(" ")
            i += 1
            continue

        if ch == "'":
            # Heuristic: treat as char literal only if we can find a closing quote soon.
            # This avoids eating lifetimes like `impl<'a>`.
            limit = 16
            j = i + 1
            escaped = False
            found = False
            while j < n and j - i <= limit and code[j] != "\n":
                cj = code[j]
                if escaped:
                    escaped = False
                else:
                    if cj == "\\":
                        escaped = True
                    elif cj == "'":
                        found = True
                        break
                j += 1
            if found:
                mode = "char"
                emit(" ")
                i += 1
                continue

        emit(ch)
        i += 1

    return "".join(out)


def find_matching_brace(code: str, open_idx: int) -> int:
    depth = 0
    for i in range(open_idx, len(code)):
        ch = code[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i
    return -1


def extract_rust_methods(src_dir: Path) -> Dict[str, Set[str]]:
    methods: Dict[str, Set[str]] = {}
    trait_methods: Dict[str, Set[str]] = {}

    trait_re = re.compile(r"\bpub\s+trait\s+([A-Za-z_]\w*)\b")
    trait_impl_re = re.compile(r"\bimpl\s+([A-Za-z_]\w*)\s+for\s+([A-Za-z_]\w*)\b")
    fn_re = re.compile(r"\bfn\s+([A-Za-z_]\w*)\s*\(")

    # Allow optional generics between the name and `(`, e.g. `pub fn save<P: AsRef<Path>>(...`.
    pub_fn_re = re.compile(r"\bpub\s+fn\s+([A-Za-z_]\w*)\s*(?:<[^(){}]*>)?\s*\(")
    impl_re = re.compile(r"\bimpl\s+([A-Za-z_]\w*)\b")

    # Pass 1: collect methods from `pub trait TraitName { ... }` blocks.
    for path in sorted(src_dir.rglob("*.rs")):
        code = path.read_text(encoding="utf-8")
        clean = strip_rust_comments_and_strings(code)

        pos = 0
        while True:
            m = trait_re.search(clean, pos)
            if not m:
                break
            trait = m.group(1)
            brace = clean.find("{", m.end())
            if brace < 0:
                pos = m.end()
                continue
            end = find_matching_brace(clean, brace)
            if end < 0:
                pos = brace + 1
                continue
            body = clean[brace + 1 : end]
            for fm in fn_re.finditer(body):
                trait_methods.setdefault(trait, set()).add(fm.group(1))
            pos = end + 1

    # Pass 2: collect inherent `pub fn` methods from `impl Type { ... }` blocks.
    for path in sorted(src_dir.rglob("*.rs")):
        code = path.read_text(encoding="utf-8")
        clean = strip_rust_comments_and_strings(code)

        pos = 0
        while True:
            m = impl_re.search(clean, pos)
            if not m:
                break
            typ = m.group(1)
            brace = clean.find("{", m.end())
            if brace < 0:
                pos = m.end()
                continue
            header = clean[m.start() : brace]
            if re.search(r"\bfor\b", header):
                pos = brace + 1
                continue
            end = find_matching_brace(clean, brace)
            if end < 0:
                pos = brace + 1
                continue
            body = clean[brace + 1 : end]
            for fm in pub_fn_re.finditer(body):
                name = fm.group(1)
                methods.setdefault(typ, set()).add(name)
            pos = end + 1

    # Pass 3: add methods from in-crate `pub trait` implementations to implementors.
    for path in sorted(src_dir.rglob("*.rs")):
        code = path.read_text(encoding="utf-8")
        clean = strip_rust_comments_and_strings(code)

        pos = 0
        while True:
            m = trait_impl_re.search(clean, pos)
            if not m:
                break
            trait = m.group(1)
            typ = m.group(2)
            if trait in trait_methods:
                methods.setdefault(typ, set()).update(trait_methods[trait])
            pos = m.end()
    return methods


def generate_report(
    csharp: Dict[str, CSharpType],
    rust: Dict[str, Set[str]],
    out_path: Path,
) -> None:
    now = _dt.datetime.utcnow().replace(microsecond=0).isoformat() + "Z"
    lines: List[str] = []
    lines.append("# PicoGK API Gap Report (C# -> Rust)")
    lines.append("")
    lines.append("Heuristic diff between the C# public API surface and the Rust bindings.")
    lines.append("")
    lines.append(f"- GeneratedAtUtc: `{now}`")
    lines.append(f"- CSharpDump: `picogk-rs/CSHARP_PUBLIC_API.md`")
    lines.append(f"- RustSource: `picogk-rs/src/`")
    lines.append("")

    for cs_type in sorted(csharp.keys()):
        rs_type = TYPE_MAP.get(cs_type, cs_type)
        if rs_type not in rust:
            continue

        cs_methods = sorted(csharp[cs_type].methods)
        rs_methods = rust.get(rs_type, set())

        covered: List[str] = []
        missing: List[Tuple[str, List[str]]] = []

        for m in cs_methods:
            cands = sorted(csharp_method_candidates(m))
            if any(c in rs_methods for c in cands):
                covered.append(m)
            else:
                missing.append((m, cands))

        total = len(cs_methods)
        miss = len(missing)
        lines.append(f"## {cs_type} -> {rs_type}")
        lines.append("")
        lines.append(f"- C#: {total} methods")
        lines.append(
            f"- Rust: {len(rs_methods)} methods (public: inherent `pub fn` + in-crate `pub trait` methods)"
        )
        lines.append(f"- Missing: {miss}")
        lines.append("")

        if missing:
            lines.append("### Missing (C# method -> candidates)")
            for name, cands in missing:
                lines.append(f"- `{name}` -> `{', '.join(cands)}`")
            lines.append("")

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main(argv: Sequence[str]) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--csharp",
        type=Path,
        default=Path("picogk-rs/CSHARP_PUBLIC_API.md"),
        help="Path to C# public API dump markdown",
    )
    ap.add_argument(
        "--rust-src",
        type=Path,
        default=Path("picogk-rs/src"),
        help="Path to Rust source directory",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=Path("picogk-rs/API_GAP_REPORT.md"),
        help="Output path for the gap report markdown",
    )
    ns = ap.parse_args(argv)

    if not ns.csharp.exists():
        print(f"error: missing C# dump: {ns.csharp}", file=os.sys.stderr)
        return 2
    if not ns.rust_src.exists():
        print(f"error: missing Rust src dir: {ns.rust_src}", file=os.sys.stderr)
        return 2

    csharp = parse_csharp_api_dump(ns.csharp)
    rust = extract_rust_methods(ns.rust_src)
    generate_report(csharp, rust, ns.out)
    print(f"Wrote: {ns.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(os.sys.argv[1:]))
