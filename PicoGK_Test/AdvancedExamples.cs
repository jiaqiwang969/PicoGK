using System.Numerics;
using PicoGK;

namespace PicoGKTest;

/// <summary>
/// 高级示例：展示 PicoGK 的强大功能
/// </summary>
public class AdvancedExamples
{
    public static void Run()
    {
        try
        {
            var repoRoot = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "../../../../"));
            var lightsFile = Path.Combine(repoRoot, "ViewerEnvironment", "PicoGKDefaultEnv.zip");
            Library.Go(
                0.3f,
                Task,
                strSrcFolder: repoRoot,
                strLightsFile: lightsFile,
                bEndAppWithTask: true
            );  // 使用更小的体素尺寸以获得更高精度
        }
        catch (Exception ex)
        {
            Console.WriteLine($"错误: {ex.Message}");
        }
    }

    /// <summary>
    /// 以 headless 模式运行（不创建 Viewer 窗口），方便在 CI/对照验证里批量生成 STL 输出。
    /// </summary>
    public static void RunHeadless()
    {
        try
        {
            var repoRoot = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "../../../../"));
            // Allow CI/tests to redirect outputs into a build directory (avoid polluting the repo).
            // If relative, treat it as relative to the repo root.
            var outDir = Environment.GetEnvironmentVariable("PICOGK_TEST_OUTPUT_DIR");
            if (string.IsNullOrWhiteSpace(outDir))
            {
                outDir = Path.Combine(repoRoot, "PicoGK_Test");
            }
            else if (!Path.IsPathRooted(outDir))
            {
                outDir = Path.GetFullPath(Path.Combine(repoRoot, outDir));
            }

            Directory.CreateDirectory(outDir);
            Directory.SetCurrentDirectory(outDir);

            using (new Library(0.3f))
            {
                Task();
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"错误: {ex.Message}");
        }
    }

    static void Task()
    {
        Console.WriteLine("\n=== PicoGK 高级示例 ===\n");
        Console.WriteLine($"PicoGK 版本: {Library.strVersion()}\n");

        // 示例 1: 使用隐式函数创建 Gyroid 结构（三周期极小曲面）
        Example1_GyroidStructure();

        // 示例 2: 复杂的偏移和平滑操作
        Example2_OffsetAndSmooth();

        // 示例 3: 创建壳体结构
        Example3_ShellStructures();

        // 示例 4: 自定义隐式函数 - 扭曲的圆环
        Example4_TwistedTorus();

        // 示例 5: 复杂的工程结构 - 热交换器
        Example5_HeatExchanger();

        // 示例 6: 参数化晶格结构
        Example6_ParametricLattice();

        Console.WriteLine("\n=== 所有高级示例完成！===\n");
    }

    /// <summary>
    /// 示例 1: Gyroid 三周期极小曲面结构
    /// Gyroid 是一种数学曲面，常用于轻量化结构设计
    /// </summary>
    static void Example1_GyroidStructure()
    {
        Console.WriteLine("1. 创建 Gyroid 三周期极小曲面结构...");

        // 定义 Gyroid 隐式函数
        var gyroid = new GyroidImplicit(
            scale: 10.0f,      // 周期大小
            thickness: 1.5f,   // 壁厚
            bounds: new BBox3(
                new Vector3(-30, -30, -30),
                new Vector3(30, 30, 30)
            )
        );

        Voxels voxGyroid = new Voxels(gyroid);
        Console.WriteLine($"   Gyroid 边界框: {voxGyroid.oCalculateBoundingBox()}");

        Mesh meshGyroid = voxGyroid.mshAsMesh();
        meshGyroid.SaveToStlFile("gyroid.stl");
        Console.WriteLine("   已保存: gyroid.stl\n");

        voxGyroid.Dispose();
    }

    /// <summary>
    /// 示例 2: 偏移和平滑操作
    /// 展示如何使用 Offset 和 Smoothen 来修改几何体
    /// </summary>
    static void Example2_OffsetAndSmooth()
    {
        Console.WriteLine("2. 偏移和平滑操作...");

        // 创建一个基础形状
        Voxels voxBase = Voxels.voxSphere(Vector3.Zero, 15.0f);

        // 向外偏移 5mm
        Voxels voxExpanded = voxBase.voxOffset(5.0f);
        voxExpanded.mshAsMesh().SaveToStlFile("expanded_sphere.stl");
        Console.WriteLine("   已保存: expanded_sphere.stl (向外偏移 5mm)");

        // 向内偏移 -3mm
        Voxels voxShrunk = voxBase.voxOffset(-3.0f);
        voxShrunk.mshAsMesh().SaveToStlFile("shrunk_sphere.stl");
        Console.WriteLine("   已保存: shrunk_sphere.stl (向内偏移 3mm)");

        // 平滑操作（三次偏移）
        Voxels voxSmooth = voxBase.voxSmoothen(2.0f);
        voxSmooth.mshAsMesh().SaveToStlFile("smooth_sphere.stl");
        Console.WriteLine("   已保存: smooth_sphere.stl (平滑处理)\n");

        voxBase.Dispose();
        voxExpanded.Dispose();
        voxShrunk.Dispose();
        voxSmooth.Dispose();
    }

    /// <summary>
    /// 示例 3: 壳体结构
    /// 创建空心壳体，常用于轻量化设计
    /// </summary>
    static void Example3_ShellStructures()
    {
        Console.WriteLine("3. 创建壳体结构...");

        // 创建一个复杂的基础形状
        Lattice lat = new Lattice();
        lat.AddSphere(Vector3.Zero, 20.0f);

        // 添加一些突出的特征
        for (int i = 0; i < 6; i++)
        {
            float angle = i * MathF.PI / 3;
            Vector3 dir = new Vector3(MathF.Cos(angle), MathF.Sin(angle), 0) * 25;
            lat.AddBeam(Vector3.Zero, dir, 5.0f, 2.0f);
        }

        Voxels voxSolid = new Voxels(lat);

        // 创建 3mm 厚的壳体
        Voxels voxShell = voxSolid.voxShell(-3.0f, 0.0f);
        voxShell.mshAsMesh().SaveToStlFile("shell_structure.stl");
        Console.WriteLine("   已保存: shell_structure.stl (3mm 壳体)");

        // 创建带平滑内部的壳体
        Voxels voxShellSmooth = voxSolid.voxShell(-4.0f, 1.0f, 1.5f);
        voxShellSmooth.mshAsMesh().SaveToStlFile("shell_smooth.stl");
        Console.WriteLine("   已保存: shell_smooth.stl (平滑内部壳体)\n");

        lat.Dispose();
        voxSolid.Dispose();
        voxShell.Dispose();
        voxShellSmooth.Dispose();
    }

    /// <summary>
    /// 示例 4: 自定义隐式函数 - 扭曲的圆环
    /// </summary>
    static void Example4_TwistedTorus()
    {
        Console.WriteLine("4. 创建扭曲圆环...");

        var torus = new TwistedTorusImplicit(
            majorRadius: 20.0f,
            minorRadius: 5.0f,
            twists: 3.0f,
            bounds: new BBox3(
                new Vector3(-30, -30, -10),
                new Vector3(30, 30, 10)
            )
        );

        Voxels voxTorus = new Voxels(torus);
        voxTorus.mshAsMesh().SaveToStlFile("twisted_torus.stl");
        Console.WriteLine("   已保存: twisted_torus.stl\n");

        voxTorus.Dispose();
    }

    /// <summary>
    /// 示例 5: 复杂工程结构 - 热交换器概念
    /// </summary>
    static void Example5_HeatExchanger()
    {
        Console.WriteLine("5. 创建热交换器结构...");

        // 外壳
        Voxels voxOuter = Voxels.voxSphere(Vector3.Zero, 25.0f);

        // 内部 Gyroid 结构用于增加表面积
        var gyroid = new GyroidImplicit(
            scale: 8.0f,
            thickness: 1.0f,
            bounds: new BBox3(
                new Vector3(-23, -23, -23),
                new Vector3(23, 23, 23)
            )
        );
        Voxels voxGyroid = new Voxels(gyroid);

        // 创建进出口通道
        Lattice channels = new Lattice();

        // 入口通道
        channels.AddBeam(
            new Vector3(0, 0, -30),
            new Vector3(0, 0, -20),
            3.0f, 3.0f
        );

        // 出口通道
        channels.AddBeam(
            new Vector3(0, 0, 20),
            new Vector3(0, 0, 30),
            3.0f, 3.0f
        );

        Voxels voxChannels = new Voxels(channels);

        // 组合：外壳与 Gyroid 的交集，然后减去通道
        Voxels voxHeatExchanger = voxOuter.voxBoolIntersect(voxGyroid);
        voxHeatExchanger.BoolAdd(voxChannels);  // 添加通道

        voxHeatExchanger.mshAsMesh().SaveToStlFile("heat_exchanger.stl");
        Console.WriteLine("   已保存: heat_exchanger.stl\n");

        voxOuter.Dispose();
        voxGyroid.Dispose();
        channels.Dispose();
        voxChannels.Dispose();
        voxHeatExchanger.Dispose();
    }

    /// <summary>
    /// 示例 6: 参数化晶格结构
    /// </summary>
    static void Example6_ParametricLattice()
    {
        Console.WriteLine("6. 创建参数化晶格结构...");

        Lattice lattice = new Lattice();

        // 创建一个立方体晶格
        int gridSize = 5;
        float spacing = 10.0f;
        float beamRadius = 0.8f;

        // 添加节点（球体）
        for (int x = 0; x < gridSize; x++)
        {
            for (int y = 0; y < gridSize; y++)
            {
                for (int z = 0; z < gridSize; z++)
                {
                    Vector3 pos = new Vector3(
                        (x - gridSize / 2.0f) * spacing,
                        (y - gridSize / 2.0f) * spacing,
                        (z - gridSize / 2.0f) * spacing
                    );

                    lattice.AddSphere(pos, beamRadius * 1.5f);
                }
            }
        }

        // 添加连接梁
        for (int x = 0; x < gridSize; x++)
        {
            for (int y = 0; y < gridSize; y++)
            {
                for (int z = 0; z < gridSize; z++)
                {
                    Vector3 pos = new Vector3(
                        (x - gridSize / 2.0f) * spacing,
                        (y - gridSize / 2.0f) * spacing,
                        (z - gridSize / 2.0f) * spacing
                    );

                    // X 方向连接
                    if (x < gridSize - 1)
                    {
                        Vector3 next = pos + new Vector3(spacing, 0, 0);
                        lattice.AddBeam(pos, next, beamRadius, beamRadius);
                    }

                    // Y 方向连接
                    if (y < gridSize - 1)
                    {
                        Vector3 next = pos + new Vector3(0, spacing, 0);
                        lattice.AddBeam(pos, next, beamRadius, beamRadius);
                    }

                    // Z 方向连接
                    if (z < gridSize - 1)
                    {
                        Vector3 next = pos + new Vector3(0, 0, spacing);
                        lattice.AddBeam(pos, next, beamRadius, beamRadius);
                    }
                }
            }
        }

        Voxels voxLattice = new Voxels(lattice);
        voxLattice.mshAsMesh().SaveToStlFile("parametric_lattice.stl");
        Console.WriteLine("   已保存: parametric_lattice.stl\n");

        lattice.Dispose();
        voxLattice.Dispose();
    }
}

/// <summary>
/// Gyroid 隐式函数实现
/// Gyroid 是一种三周期极小曲面，公式：sin(x)cos(y) + sin(y)cos(z) + sin(z)cos(x) = 0
/// </summary>
class GyroidImplicit : IBoundedImplicit
{
    private readonly float _scale;
    private readonly float _thickness;
    private readonly BBox3 _bounds;

    public GyroidImplicit(float scale, float thickness, BBox3 bounds)
    {
        _scale = scale;
        _thickness = thickness;
        _bounds = bounds;
    }

    public BBox3 oBounds => _bounds;

    public float fSignedDistance(in Vector3 vec)
    {
        // 缩放坐标
        float x = vec.X / _scale;
        float y = vec.Y / _scale;
        float z = vec.Z / _scale;

        // Gyroid 公式
        float gyroid = MathF.Sin(x) * MathF.Cos(y) +
                       MathF.Sin(y) * MathF.Cos(z) +
                       MathF.Sin(z) * MathF.Cos(x);

        // 返回到表面的距离（考虑壁厚）
        return MathF.Abs(gyroid) - _thickness / _scale;
    }
}

/// <summary>
/// 扭曲圆环隐式函数
/// </summary>
class TwistedTorusImplicit : IBoundedImplicit
{
    private readonly float _majorRadius;
    private readonly float _minorRadius;
    private readonly float _twists;
    private readonly BBox3 _bounds;

    public TwistedTorusImplicit(float majorRadius, float minorRadius, float twists, BBox3 bounds)
    {
        _majorRadius = majorRadius;
        _minorRadius = minorRadius;
        _twists = twists;
        _bounds = bounds;
    }

    public BBox3 oBounds => _bounds;

    public float fSignedDistance(in Vector3 vec)
    {
        // 计算到 Z 轴的距离
        float distToAxis = MathF.Sqrt(vec.X * vec.X + vec.Y * vec.Y);

        // 计算角度
        float angle = MathF.Atan2(vec.Y, vec.X);

        // 扭曲：根据 Z 坐标旋转截面
        float twist = angle + _twists * vec.Z / 10.0f;

        // 圆环的圆心位置
        Vector3 torusCenter = new Vector3(
            _majorRadius * MathF.Cos(angle),
            _majorRadius * MathF.Sin(angle),
            vec.Z
        );

        // 计算到圆环表面的距离
        Vector3 diff = vec - torusCenter;

        // 应用扭曲
        float rotatedX = diff.X * MathF.Cos(twist) - diff.Y * MathF.Sin(twist);
        float rotatedY = diff.X * MathF.Sin(twist) + diff.Y * MathF.Cos(twist);

        float distToSurface = MathF.Sqrt(rotatedX * rotatedX + rotatedY * rotatedY + diff.Z * diff.Z);

        return distToSurface - _minorRadius;
    }
}
