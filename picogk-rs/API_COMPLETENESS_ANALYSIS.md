# PicoGK Rust 迁移 - API/行为对齐现状

**更新时间**: 2026-01-20  
**结论**: Rust 绑定已具备核心可用性，并已通过质量门禁（fmt/test/clippy）。`PicoGK_Test/AdvancedExamples.cs`
可在 Rust 侧复现并进行对照验证；剩余工作主要集中在文档收敛、CI/发布形态、线程安全/回调契约的“定稿化”，以及少量扩展 API。

---

## 1) 验收口径（推荐）

- **API 覆盖**：以 C# 公共 API 为参考，Rust 提供等价能力（命名可 Rust 化，行为/输出一致）
- **行为对齐**：以 `PicoGK_Test/AdvancedExamples.cs` 作为端到端回归基线
- **工程门禁**：`cargo fmt --check` + `cargo test` + `cargo clippy --all-targets -- -D warnings`
- **平台**：macOS arm64 + Windows x64；Linux 需用户提供原生 `.so`（见下文）

---

## 2) 当前完成度（以清单为准）

仓库内的实现覆盖以 `FEATURE_CHECKLIST.md` 为准：`picogk-rs/FEATURE_CHECKLIST.md`。

额外说明：
- Rust 侧 `Voxels/ScalarField/VectorField` 不再实现 `Clone`（避免隐藏的 panic）；请使用 `duplicate()` / `try_clone()`
- Mesh ABI 已复核并修正为 `Vector3f*`（与 C# `in/ref Vector3` 对齐）
- 已新增 C# public API 的反射 dump：`picogk-rs/CSHARP_PUBLIC_API.md`，并新增 FFI 符号覆盖回归测试：`picogk-rs/tests/ffi_symbol_parity_test.rs`

---

## 3) C# 示例对照验证（已落地）

已新增一个 **opt-in**（`#[ignore]`）的对照测试：
- `picogk-rs/tests/csharp_advanced_examples_parity.rs`

对照策略：
- 二进制 STL 的 **顶点/attribute 字节**必须一致
- STL **normal** 字节不做强一致要求（跨语言浮点末位差异会导致 SHA256 不一致，但几何顶点一致）

运行方式：
```bash
cd picogk-rs
cargo test --test csharp_advanced_examples_parity -- --ignored
```

如需在 macOS 上重新生成 C# 基线 STL（本仓库 `native/osx-arm64`）：
```bash
PICOGK_TEST_OUTPUT_DIR=picogk-rs/target/csharp_advanced_examples_baseline \
  dotnet run --project PicoGK_Test -c Release
```

---

## 4) 平台/链接策略（Rust）

默认链接使用仓库自带的原生库：
- macOS arm64：`native/osx-arm64`
- Windows x64：`native/win-x64`

可用环境变量覆盖：
- `PICOGK_LIB_DIR`：原生库所在目录
- `PICOGK_LIB_NAME`：库名（默认 `picogk.1.7`）

macOS / Windows：
- build.rs 会把运行时需要的动态库复制到 `target/{profile}/deps` / `target/{profile}/examples`，以便 `cargo test` / `cargo run`
  开箱即用（不需要额外设置 `DYLD_LIBRARY_PATH` / `PATH`）。

Linux 不自带预编译原生库：可设置 `PICOGK_LIB_DIR` 指向 `.so`，或将 `.so` 放到 `native/linux-x64` / `native/linux-arm64`，或安装到系统 linker 搜索路径（由链接阶段决定是否可用）。

如仅需在无原生库环境（例如 Linux CI）跑 `cargo check/clippy/doc`，可设置 `PICOGK_NO_NATIVE=1`（或 `DOCS_RS=1`）跳过原生链接。

---

## 5) 已知风险/待定稿项

- **线程安全/重入契约**：部分 `unsafe impl Send/Sync` 的正确性依赖原生库是否可并发调用；回调桥接使用全局指针并假设同步回调  
  建议阅读：`picogk-rs/SAFETY.md`
- **文档收敛**：仓库内仍保留部分早期“验证/报告”文档，已在文件顶部标注过期或阶段性说明（以免误导）
