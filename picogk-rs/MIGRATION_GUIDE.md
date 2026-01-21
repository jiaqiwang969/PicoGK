# C# 到 Rust 迁移指南

## 语法对照表

### 1. 基础类型

| C# | Rust | 说明 |
|----|----|------|
| `int` | `i32` | 32位有符号整数 |
| `float` | `f32` | 32位浮点数 |
| `double` | `f64` | 64位浮点数 |
| `bool` | `bool` | 布尔值 |
| `string` | `String` | 可变字符串 |
| `string` | `&str` | 字符串切片（不可变） |
| `Vector3` | `Vector3<f32>` | 3D向量（nalgebra） |

### 2. 变量声明

**C#:**
```csharp
int x = 5;
float y = 3.14f;
var z = "hello";
```

**Rust:**
```rust
let x: i32 = 5;
let y: f32 = 3.14;
let z = "hello";  // 类型推断
```

### 3. 可变性

**C#:**
```csharp
// 默认可变
int x = 5;
x = 10;  // OK

// 不可变需要显式声明
readonly int y = 5;
```

**Rust:**
```rust
// 默认不可变
let x = 5;
// x = 10;  // 编译错误！

// 可变需要显式声明
let mut y = 5;
y = 10;  // OK
```

### 4. 空值处理

**C#:**
```csharp
Voxels? vox = null;
if (vox != null) {
    vox.Offset(5.0f);
}

// 或使用空值合并
vox?.Offset(5.0f);
```

**Rust:**
```rust
let vox: Option<Voxels> = None;
if let Some(mut v) = vox {
    v.offset(5.0);
}

// 或使用 map
vox.map(|mut v| v.offset(5.0));
```

### 5. 错误处理

**C#:**
```csharp
try {
    var mesh = Mesh.LoadFromFile("model.stl");
} catch (Exception ex) {
    Console.WriteLine($"Error: {ex.Message}");
}
```

**Rust:**
```rust
match Mesh::load_stl("model.stl") {
    Ok(mesh) => println!("Loaded!"),
    Err(e) => eprintln!("Error: {}", e),
}

// 或使用 ? 操作符
let mesh = Mesh::load_stl("model.stl")?;
```

### 6. 类与结构体

**C#:**
```csharp
public class Voxels : IDisposable {
    private IntPtr handle;

    public Voxels() {
        handle = _Create();
    }

    public void Dispose() {
        if (handle != IntPtr.Zero) {
            _Destroy(handle);
        }
    }
}
```

**Rust:**
```rust
pub struct Voxels {
    handle: *mut CVoxels,
}

impl Voxels {
    pub fn new() -> Result<Self> {
        let handle = unsafe { ffi::Voxels_Create() };
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }
}

impl Drop for Voxels {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::Voxels_Destroy(self.handle);
            }
        }
    }
}
```

### 7. 方法调用

**C#:**
```csharp
var vox = new Voxels();
vox.BoolAdd(other);
vox.Offset(5.0f);
```

**Rust:**
```rust
let mut vox = Voxels::new()?;
vox.bool_add(&other);
vox.offset(5.0);
```

### 8. 集合操作

**C#:**
```csharp
var list = new List<Voxels>();
list.Add(vox1);
list.Add(vox2);

foreach (var vox in list) {
    vox.Offset(1.0f);
}
```

**Rust:**
```rust
let mut list = Vec::new();
list.push(vox1);
list.push(vox2);

for vox in &mut list {
    vox.offset(1.0);
}
```

### 9. 并行处理

**C#:**
```csharp
Parallel.For(0, 100, i => {
    var vox = Voxels.voxSphere(new Vector3(i, 0, 0), 5.0f);
    // ...
});
```

**Rust:**
```rust
use rayon::prelude::*;

(0..100).into_par_iter().for_each(|i| {
    let vox = Voxels::sphere(Vector3::new(i as f32, 0.0, 0.0), 5.0).unwrap();
    // ...
});
```

## 完整示例对照

### 示例 1: 创建球体

**C# 版本:**
```csharp
using System.Numerics;
using PicoGK;

try
{
    Library.Go(0.5f, Task);
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}

void Task()
{
    var sphere = Voxels.voxSphere(Vector3.Zero, 20.0f);
    var mesh = sphere.mshAsMesh();
    mesh.SaveToStlFile("sphere.stl");

    sphere.Dispose();
}
```

**Rust 版本:**
```rust
use picogk::{Library, Voxels, Result};
use nalgebra::Vector3;

fn main() -> Result<()> {
    let _lib = Library::init(0.5)?;

    let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    let mesh = sphere.as_mesh()?;
    mesh.save_stl("sphere.stl")?;

    // 自动释放，无需手动 Dispose
    Ok(())
}
```

### 示例 2: 布尔运算

**C# 版本:**
```csharp
var sphere1 = Voxels.voxSphere(new Vector3(-10, 0, 0), 15.0f);
var sphere2 = Voxels.voxSphere(new Vector3(10, 0, 0), 15.0f);

var combined = sphere1.voxBoolAdd(sphere2);
combined.mshAsMesh().SaveToStlFile("combined.stl");

sphere1.Dispose();
sphere2.Dispose();
combined.Dispose();
```

**Rust 版本:**
```rust
let sphere1 = Voxels::sphere(Vector3::new(-10.0, 0.0, 0.0), 15.0)?;
let sphere2 = Voxels::sphere(Vector3::new(10.0, 0.0, 0.0), 15.0)?;

let mut combined = sphere1;
combined.bool_add(&sphere2);
combined.as_mesh()?.save_stl("combined.stl")?;

// 自动释放
```

### 示例 3: Gyroid 结构

**C# 版本:**
```csharp
class GyroidImplicit : IBoundedImplicit
{
    private float _scale;
    private float _thickness;
    private BBox3 _bounds;

    public GyroidImplicit(float scale, float thickness, BBox3 bounds)
    {
        _scale = scale;
        _thickness = thickness;
        _bounds = bounds;
    }

    public BBox3 oBounds => _bounds;

    public float fSignedDistance(in Vector3 vec)
    {
        float x = vec.X / _scale;
        float y = vec.Y / _scale;
        float z = vec.Z / _scale;

        float gyroid = MathF.Sin(x) * MathF.Cos(y) +
                       MathF.Sin(y) * MathF.Cos(z) +
                       MathF.Sin(z) * MathF.Cos(x);

        return MathF.Abs(gyroid) - _thickness / _scale;
    }
}

// 使用
var gyroid = new GyroidImplicit(10.0f, 1.5f, bounds);
var voxels = new Voxels(gyroid);
```

**Rust 版本:**
```rust
pub struct GyroidImplicit {
    scale: f32,
    thickness: f32,
    bounds: BBox3,
}

impl Implicit for GyroidImplicit {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let x = point.x / self.scale;
        let y = point.y / self.scale;
        let z = point.z / self.scale;

        let gyroid = x.sin() * y.cos() +
                     y.sin() * z.cos() +
                     z.sin() * x.cos();

        gyroid.abs() - self.thickness / self.scale
    }

    fn bounds(&self) -> Option<BBox3> {
        Some(self.bounds)
    }
}

// 使用
let gyroid = GyroidImplicit::new(10.0, 1.5, bounds);
let voxels = Voxels::from_implicit(&gyroid)?;
```

## 关键差异总结

### 1. 内存管理

| 特性 | C# | Rust |
|------|----|----|
| 资源释放 | 手动调用 Dispose | 自动（Drop trait） |
| 内存泄漏 | 可能忘记 Dispose | 编译时保证不会泄漏 |
| 垃圾回收 | 有 GC | 无 GC |

### 2. 错误处理

| 特性 | C# | Rust |
|------|----|----|
| 异常 | try-catch | Result<T, E> |
| 空值 | null, ? | Option<T> |
| 强制处理 | 否 | 是（编译时） |

### 3. 并发安全

| 特性 | C# | Rust |
|------|----|----|
| 数据竞争 | 运行时检测 | 编译时防止 |
| 线程安全 | 手动保证 | Send/Sync trait |
| 锁 | lock, Monitor | Mutex, RwLock |

### 4. 性能

| 特性 | C# | Rust |
|------|----|----|
| FFI 开销 | P/Invoke 有开销 | 零开销 |
| 运行时 | .NET Runtime | 无运行时 |
| 内联 | 受限 | 完全内联 |

## 迁移步骤

### 第一步：设置环境

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 创建项目
cargo new my_picogk_project
cd my_picogk_project

# 添加依赖
cargo add picogk nalgebra rayon
```

### 第二步：转换代码

1. 将 `using` 改为 `use`
2. 将 `var` 改为 `let` 或 `let mut`
3. 将 `new` 改为 `::new()`
4. 将方法名从 PascalCase 改为 snake_case
5. 添加 `?` 处理错误
6. 删除所有 `Dispose()` 调用

### 第三步：编译和测试

```bash
# 编译
cargo build

# 运行
cargo run

# 测试
cargo test

# 性能测试
cargo bench
```

## 常见问题

### Q: 如何处理 C# 的 null？

**A:** 使用 `Option<T>`

```rust
// C#: Voxels? vox = null;
let vox: Option<Voxels> = None;

// C#: if (vox != null) { ... }
if let Some(v) = vox {
    // ...
}
```

### Q: 如何处理异常？

**A:** 使用 `Result<T, E>`

```rust
// C#: try { ... } catch { ... }
match operation() {
    Ok(result) => { /* success */ },
    Err(e) => { /* error */ },
}

// 或使用 ?
let result = operation()?;
```

### Q: 如何共享数据？

**A:** 使用 `Arc<T>` 或 `Rc<T>`

```rust
use std::sync::Arc;

let vox = Arc::new(Voxels::new()?);
let vox_clone = Arc::clone(&vox);

// 在多个线程中使用
```

### Q: 如何修改共享数据？

**A:** 使用 `Arc<Mutex<T>>`

```rust
use std::sync::{Arc, Mutex};

let vox = Arc::new(Mutex::new(Voxels::new()?));

// 在线程中
let mut v = vox.lock().unwrap();
v.offset(5.0);
```

## 总结

Rust 迁移的主要优势：

1. ✅ **更安全** - 编译时保证内存安全和线程安全
2. ✅ **更快** - 零成本抽象，无 GC 暂停
3. ✅ **更可靠** - 强类型系统，编译时错误检查
4. ✅ **更简单** - 自动资源管理，无需手动 Dispose
5. ✅ **更现代** - 现代化的工具链和生态系统

虽然语法有所不同，但 Rust 的设计使得代码更加安全和高效！
