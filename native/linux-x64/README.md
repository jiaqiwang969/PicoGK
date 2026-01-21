# Linux x64 Native Library (Optional)

This repository does not currently ship a prebuilt PicoGK `.so` for Linux.

If you have a compatible PicoGK native build, you can place it in this folder so the Rust
bindings can discover it automatically on Linux x64:

- Expected file name (default): `libpicogk.1.7.so`
- Or set `PICOGK_LIB_NAME` to match your library name (without the `lib` prefix / extension)

Alternatively, set `PICOGK_LIB_DIR` to the directory containing your `.so`, or install the
library into the system linker search path (e.g. `/usr/lib`).

