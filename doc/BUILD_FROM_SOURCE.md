# PicoGK 从源代码编译方案

## 当前情况分析

当前 PicoGK 项目结构：
```
PicoGK/
├── *.cs                    # C# 封装层
├── PicoGK.csproj          # C# 项目文件
└── native/
    ├── osx-arm64/
    │   └── picogk.1.7.dylib    # 预编译的 macOS 库
    └── win-x64/
        └── picogk.1.7.dll      # 预编译的 Windows 库
```

**问题**：没有 C++ 源代码，只有预编译的二进制文件。

## 解决方案

### 方案 1：获取 PicoGK C++ 源代码（推荐）

PicoGK 的 C++ 源代码可能在以下位置：

1. **单独的私有仓库**
   - LEAP 71 可能将 C++ 实现放在单独的仓库中
   - 需要联系 LEAP 71 获取访问权限

2. **GitHub 上的其他仓库**
   ```bash
   # 搜索 LEAP 71 的其他仓库
   https://github.com/leap71?tab=repositories
   ```

3. **作为子模块**
   ```bash
   # 检查是否有 git 子模块
   cd /Users/jqwang/166-leap71/PicoGK
   git submodule status
   ```

### 方案 2：创建自己的 C++ 实现

基于 OpenVDB 从零实现 PicoGK 的核心功能。

#### 2.1 项目结构

```
picogk-native/
├── CMakeLists.txt          # CMake 构建配置
├── include/
│   └── picogk/
│       ├── voxels.h        # Voxels 类
│       ├── mesh.h          # Mesh 类
│       ├── lattice.h       # Lattice 类
│       └── library.h       # 库初始化
├── src/
│   ├── voxels.cpp
│   ├── mesh.cpp
│   ├── lattice.cpp
│   └── library.cpp
└── bindings/
    └── rust/
        └── wrapper.cpp     # Rust FFI 包装器
```

#### 2.2 依赖项

```cmake
# CMakeLists.txt
cmake_minimum_required(VERSION 3.15)
project(picogk-native)

# 依赖项
find_package(OpenVDB REQUIRED)
find_package(TBB REQUIRED)
find_package(Blosc REQUIRED)

# 编译选项
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 源文件
add_library(picogk SHARED
    src/library.cpp
    src/voxels.cpp
    src/mesh.cpp
    src/lattice.cpp
    bindings/rust/wrapper.cpp
)

# 链接库
target_link_libraries(picogk
    OpenVDB::openvdb
    TBB::tbb
    Blosc::blosc
)
```

#### 2.3 核心实现示例

**voxels.h**:
```cpp
#pragma once
#include <openvdb/openvdb.h>
#include <memory>

namespace picogk {

class Voxels {
public:
    Voxels();
    ~Voxels();

    // Boolean operations
    void boolAdd(const Voxels& other);
    void boolSubtract(const Voxels& other);
    void boolIntersect(const Voxels& other);

    // Morphological operations
    void offset(float distance);
    void smoothen(float distance);

    // Access internal grid
    openvdb::FloatGrid::Ptr getGrid() const { return grid_; }

private:
    openvdb::FloatGrid::Ptr grid_;
};

} // namespace picogk
```

**voxels.cpp**:
```cpp
#include "picogk/voxels.h"
#include <openvdb/tools/LevelSetSphere.h>
#include <openvdb/tools/Composite.h>
#include <openvdb/tools/LevelSetFilter.h>

namespace picogk {

Voxels::Voxels() {
    grid_ = openvdb::FloatGrid::create();
    grid_->setName("PicoGK Voxels");
}

Voxels::~Voxels() = default;

void Voxels::boolAdd(const Voxels& other) {
    openvdb::tools::csgUnion(*grid_, *other.grid_);
}

void Voxels::boolSubtract(const Voxels& other) {
    openvdb::tools::csgDifference(*grid_, *other.grid_);
}

void Voxels::boolIntersect(const Voxels& other) {
    openvdb::tools::csgIntersection(*grid_, *other.grid_);
}

void Voxels::offset(float distance) {
    openvdb::tools::LevelSetFilter<openvdb::FloatGrid> filter(*grid_);
    filter.offset(distance);
}

void Voxels::smoothen(float distance) {
    offset(distance);
    offset(-2.0f * distance);
    offset(distance);
}

} // namespace picogk
```

**wrapper.cpp** (Rust FFI):
```cpp
#include "picogk/voxels.h"
#include "picogk/library.h"

extern "C" {

// Library functions
void Library_Init(float voxel_size_mm) {
    openvdb::initialize();
    // Set global voxel size
}

void Library_Destroy() {
    openvdb::uninitialize();
}

// Voxels functions
void* Voxels_hCreate() {
    return new picogk::Voxels();
}

void Voxels_Destroy(void* handle) {
    delete static_cast<picogk::Voxels*>(handle);
}

void Voxels_BoolAdd(void* this_ptr, const void* operand) {
    auto* vox = static_cast<picogk::Voxels*>(this_ptr);
    auto* other = static_cast<const picogk::Voxels*>(operand);
    vox->boolAdd(*other);
}

void Voxels_Offset(void* this_ptr, float dist_mm) {
    auto* vox = static_cast<picogk::Voxels*>(this_ptr);
    vox->offset(dist_mm);
}

// ... 其他函数

} // extern "C"
```

### 方案 3：使用 Rust 的 build.rs 编译 C++

创建 `build.rs` 来自动编译 C++ 代码：

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    // 告诉 cargo 重新运行如果这些文件改变
    println!("cargo:rerun-if-changed=native/src/");
    println!("cargo:rerun-if-changed=native/include/");

    // 编译 C++ 代码
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .std("c++17")
        .file("native/src/library.cpp")
        .file("native/src/voxels.cpp")
        .file("native/src/mesh.cpp")
        .file("native/src/lattice.cpp")
        .file("native/bindings/rust/wrapper.cpp")
        .include("native/include")
        .include("/usr/local/include")  // OpenVDB
        .flag("-O3")
        .flag("-march=native");

    // macOS 特定设置
    if cfg!(target_os = "macos") {
        build
            .flag("-stdlib=libc++")
            .flag("-mmacosx-version-min=11.0");
    }

    build.compile("picogk_native");

    // 链接 OpenVDB 和依赖
    println!("cargo:rustc-link-lib=openvdb");
    println!("cargo:rustc-link-lib=tbb");
    println!("cargo:rustc-link-lib=blosc");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=c++");

    // 添加库搜索路径
    println!("cargo:rustc-link-search=/usr/local/lib");
}
```

### 方案 4：混合方案（推荐用于快速开始）

1. **短期**：使用现有的预编译库
   ```rust
   // 直接链接到 native/osx-arm64/picogk.1.7.dylib
   #[link(name = "picogk.1.7", kind = "dylib")]
   extern "C" { ... }
   ```

2. **中期**：逐步用 Rust 重写核心功能
   ```rust
   // 使用 Rust 的 OpenVDB 绑定
   // 或者直接用 Rust 实现体素算法
   ```

3. **长期**：完全 Rust 实现
   - 纯 Rust 的体素引擎
   - GPU 加速（wgpu）
   - 无 C++ 依赖

## 推荐的实施步骤

### 第一阶段：使用预编译库（1周）

```rust
// Cargo.toml
[build-dependencies]
# 暂时不需要

// build.rs
fn main() {
    // 链接到预编译库
    let lib_path = if cfg!(target_os = "macos") {
        "native/osx-arm64"
    } else if cfg!(target_os = "windows") {
        "native/win-x64"
    } else {
        panic!("Unsupported platform");
    };

    println!("cargo:rustc-link-search={}", lib_path);
    println!("cargo:rustc-link-lib=dylib=picogk.1.7");
}
```

### 第二阶段：获取或实现 C++ 源代码（1-2个月）

1. 联系 LEAP 71 获取 C++ 源代码
2. 或者基于 OpenVDB 实现核心功能
3. 创建完整的 CMake 构建系统

### 第三阶段：Rust 原生实现（3-6个月）

1. 研究 Rust 的体素库
2. 实现核心算法
3. 性能优化
4. GPU 加速

## 依赖安装

### macOS (Homebrew)

```bash
# 安装 OpenVDB 和依赖
brew install openvdb
brew install tbb
brew install blosc
brew install cmake

# 验证安装
pkg-config --modversion openvdb
```

### 检查依赖

```bash
# 查看 dylib 依赖
otool -L native/osx-arm64/picogk.1.7.dylib

# 输出示例：
# /usr/local/lib/libopenvdb.dylib
# /usr/local/lib/libtbb.dylib
# /usr/local/lib/libblosc.dylib
```

## 下一步行动

1. **立即**：使用预编译库完成 Rust 绑定测试
2. **本周**：联系 LEAP 71 询问 C++ 源代码
3. **下周**：如果无法获取，开始基于 OpenVDB 的实现
4. **本月**：完成基本的 C++ 实现和编译系统

## 参考资源

- [OpenVDB Documentation](https://www.openvdb.org/documentation/)
- [OpenVDB GitHub](https://github.com/AcademySoftwareFoundation/openvdb)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
- [cc-rs Documentation](https://docs.rs/cc/)
