# PicoGK - Rust Edition

**PicoGK** ("peacock") is a compact, open-source geometry kernel developed by [LEAP 71](https://leap71.com/). This repository contains the **pure Rust implementation**, migrated from the original C# version.

## Overview

PicoGK serves as the foundation of a broader technology stack for [Computational Engineering](https://leap71.com/computationalengineering/), a new paradigm pioneered by LEAP 71.

The name stands for **Pico** (tiny) **G**eometry **K**ernel, and the library offers [a deliberately reduced](https://jlk.ae/2023/12/06/the-power-of-reduced-instruction-sets/) yet powerful instruction set designed to create computational geometry for engineering applications.

While it may appear minimal on the surface, PicoGK is used to generate some of the most advanced physical components imaginable: from electric motors and heat exchangers to [3D-printed rocket engines](https://leap71.com/rp/) and bio-inspired structures.

Explore more at [PicoGK.org](https://picogk.org).

## Features

- **Voxel-based geometry** - Robust boolean operations (union, subtract, intersect)
- **Implicit surfaces** - TPMS structures (Gyroid, Schwarz, etc.)
- **Mesh operations** - STL import/export, mesh transformations
- **Lattice structures** - Beam-based lattice generation
- **Scalar/Vector fields** - Field-driven geometry manipulation
- **OpenVDB support** - VDB file I/O for interoperability

## Project Structure

```
PicoGK/
├── Cargo.toml          # Rust project configuration
├── build.rs            # Build script for native library linking
├── src/                # Rust source code
│   ├── lib.rs          # Library entry point
│   ├── voxels.rs       # Voxel operations
│   ├── mesh.rs         # Mesh operations
│   ├── implicit.rs     # Implicit surfaces (TPMS)
│   ├── lattice.rs      # Lattice structures
│   └── ...
├── examples/           # Example programs
├── tests/              # Integration tests
├── native/             # Pre-compiled native libraries
│   ├── osx-arm64/      # macOS Apple Silicon
│   ├── win-x64/        # Windows x64
│   ├── linux-x64/      # Linux x64
│   └── linux-arm64/    # Linux ARM64
├── doc/                # Documentation
└── tools/              # Build and verification scripts
```

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Platform: macOS (ARM64), Windows (x64), or Linux (x64/ARM64)

### Installation

Add PicoGK to your `Cargo.toml`:

```toml
[dependencies]
picogk = { git = "https://github.com/jiaqiwang969/PicoGK.git" }
```

### Basic Example

```rust
use nalgebra::Vector3;
use picogk::{Library, Result, Voxels};

fn main() -> Result<()> {
    // Initialize PicoGK with 0.5mm voxel size
    let _lib = Library::init(0.5)?;

    println!("PicoGK {}", Library::version());

    // Create a sphere at origin with radius 20mm
    let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;

    // Convert to mesh and save
    let mesh = sphere.as_mesh()?;
    println!("Generated mesh with {} vertices", mesh.vertex_count());

    mesh.save_stl("sphere.stl")?;
    println!("Saved sphere.stl");

    Ok(())
}
```

### Gyroid TPMS Example

```rust
use nalgebra::Vector3;
use picogk::{BBox3, GyroidImplicit, Implicit, Library, Result, Voxels};

fn main() -> Result<()> {
    let _lib = Library::init(0.3)?;

    // Define bounding box (60mm cube)
    let bounds = BBox3::new(
        Vector3::new(-30.0, -30.0, -30.0),
        Vector3::new(30.0, 30.0, 30.0),
    );

    // Create Gyroid with 10mm period and 1.5mm wall thickness
    let gyroid = GyroidImplicit::new(10.0, 1.5, bounds);

    // Voxelize and export
    let vox = Voxels::from_implicit(&gyroid)?;
    let mesh = vox.as_mesh()?;
    mesh.save_stl("gyroid.stl")?;

    Ok(())
}
```

## Running Examples

```bash
# Build the project
cargo build --release

# Run examples
cargo run --release --example basic      # Creates sphere.stl
cargo run --release --example gyroid     # Creates gyroid.stl
cargo run --release --example load_stl   # Load and process STL
```

## Available Examples

| Example | Description |
|---------|-------------|
| `basic` | Create a simple sphere |
| `gyroid` | Generate Gyroid TPMS structure |
| `load_stl` | Load and process STL files |
| `save_stl` | Export geometry to STL |
| `voxels_query` | Query voxel properties |
| `voxels_advanced_offset` | Advanced offset operations |
| `vdb_io_demo` | OpenVDB file I/O |
| `comprehensive_demo` | Full feature demonstration |

## Running Tests

```bash
cargo test
```

## Documentation

| Document | Description |
|----------|-------------|
| [RUST_API_DESIGN.md](doc/RUST_API_DESIGN.md) | Rust API design principles |
| [MIGRATION_GUIDE.md](doc/MIGRATION_GUIDE.md) | C# to Rust migration guide |
| [BUILD_FROM_SOURCE.md](doc/BUILD_FROM_SOURCE.md) | Building native libraries |
| [SAFETY.md](doc/SAFETY.md) | Safety considerations |

## Tools

```bash
# Verify repository integrity
./tools/verify.sh

# Clean build artifacts
./tools/clean.sh        # Basic cleanup
./tools/clean.sh --all  # Full cleanup including large outputs
```

## Platform Support

| Platform | Architecture | Status |
|----------|--------------|--------|
| macOS | ARM64 (Apple Silicon) | Supported |
| Windows | x64 | Supported |
| Linux | x64 | Supported |
| Linux | ARM64 | Supported |

## Migration from C#

This repository has been fully migrated from C# to Rust. The original C# implementation has been removed. Key changes:

- All 36 C# source files removed
- Rust source moved from `picogk-rs/` to root
- Updated project configuration for pure Rust
- Maintained API compatibility where possible

See [MIGRATION_GUIDE.md](doc/MIGRATION_GUIDE.md) for details on migrating existing C# code.

## License

Apache 2.0 - see [LICENSE](LICENSE) for details.

## Credits

- Original PicoGK by [LEAP 71](https://leap71.com/)
- Rust port and migration

## Links

- [PicoGK.org](https://picogk.org) - Official website
- [LEAP 71](https://leap71.com/) - Company behind PicoGK
- [Computational Engineering](https://leap71.com/computationalengineering/) - The paradigm
