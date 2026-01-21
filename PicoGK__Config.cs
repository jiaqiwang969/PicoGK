//
// SPDX-License-Identifier: Apache-2.0
//
// PicoGK ("peacock") is a compact software kernel for computational geometry,
// specifically for use in Computational Engineering Models (CEM).
//
// For more information, please visit https://picogk.org
// 
// PicoGK is developed and maintained by LEAP 71 - © 2023-2026 by LEAP 71
// https://leap71.com
//
// Computational Engineering will profoundly change our physical world in the
// years ahead. Thank you for being part of the journey.
//
// We have developed this library to be used widely, for both commercial and
// non-commercial projects alike. Therefore, we have released it under a 
// permissive open-source license.
//
// The foundation of PicoGK is a thin layer on top of the powerful open-source
// OpenVDB project, which in turn uses many other Free and Open Source Software
// libraries. We are grateful to be able to stand on the shoulders of giants.
//
// LEAP 71 licenses this file to you under the Apache License, Version 2.0
// (the "License"); you may not use this file except in compliance with the
// License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, THE SOFTWARE IS
// PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED.
//
// See the License for the specific language governing permissions and
// limitations under the License.   
//

using System.Runtime.InteropServices;
using System.Reflection;
using System.Threading;

namespace PicoGK
{
    public partial class Config
    {
        // PicoGK Runtime to load

        public const string strPicoGKLib = "picogk.1.7"; // dll or dylib

        // if you want to load it from a specific location instead of
        // a standard system path, you can specify it as well
        // You need to include the full path, filename and extension such as:
        //
        // public const string strPicoGKLib = "/Users/myuser/PicoGKRuntime/picogk.1.0.dylib"
        //
    }
}

namespace PicoGK
{
    public partial class Config
    {
        static int s_nativeLoadAttempted = 0;
        static int s_resolverInstalled = 0;

        static IntPtr s_picogkHandle = IntPtr.Zero;

        /// <summary>
        /// Best-effort loader for the native PicoGK runtime library.
        ///
        /// This is primarily for local development in this repository, where the dylib/dll lives
        /// under ./native/. If the library cannot be found, this method does nothing and the
        /// subsequent P/Invoke call will fail with the standard error.
        /// </summary>
        public static void TryLoadNativeRuntime()
        {
            // Only attempt once per process.
            if (Interlocked.Exchange(ref s_nativeLoadAttempted, 1) != 0)
                return;

            try
            {
                InstallDllImportResolver();

                string? nativeDir = FindNativeDir();
                if (nativeDir is null)
                    return;

                if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
                {
                    // Preload side-by-side deps first (if present).
                    TryLoadFromDir(nativeDir, "picogk.1.7_liblzma.5.dylib");
                    TryLoadFromDir(nativeDir, "picogk.1.7_libzstd.1.dylib");
                    s_picogkHandle = TryLoadFromDir(nativeDir, "picogk.1.7.dylib");
                    if (s_picogkHandle == IntPtr.Zero)
                        s_picogkHandle = TryLoadFromDir(nativeDir, "libpicogk.1.7.dylib");
                }
                else if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
                {
                    s_picogkHandle = TryLoadFromDir(nativeDir, "picogk.1.7.dll");
                }
            }
            catch
            {
                // Swallow all exceptions here; the caller will get a meaningful error
                // from the first native entrypoint invocation.
            }
        }

        static void InstallDllImportResolver()
        {
            if (Interlocked.Exchange(ref s_resolverInstalled, 1) != 0)
                return;

            NativeLibrary.SetDllImportResolver(typeof(Config).Assembly, ResolvePicoGkLibrary);
        }

        static IntPtr ResolvePicoGkLibrary(string libraryName, Assembly assembly, DllImportSearchPath? searchPath)
        {
            // Only handle our own library; let everything else fall back to default resolution.
            if (!string.Equals(libraryName, strPicoGKLib, StringComparison.Ordinal))
                return IntPtr.Zero;

            if (s_picogkHandle != IntPtr.Zero)
                return s_picogkHandle;

            // Try a late load here as well, in case the resolver is invoked before explicit init.
            string? nativeDir = FindNativeDir();
            if (nativeDir is null)
                return IntPtr.Zero;

            if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            {
                TryLoadFromDir(nativeDir, "picogk.1.7_liblzma.5.dylib");
                TryLoadFromDir(nativeDir, "picogk.1.7_libzstd.1.dylib");
                s_picogkHandle = TryLoadFromDir(nativeDir, "picogk.1.7.dylib");
                if (s_picogkHandle == IntPtr.Zero)
                    s_picogkHandle = TryLoadFromDir(nativeDir, "libpicogk.1.7.dylib");
                return s_picogkHandle;
            }

            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                s_picogkHandle = TryLoadFromDir(nativeDir, "picogk.1.7.dll");
                return s_picogkHandle;
            }

            return IntPtr.Zero;
        }

        static IntPtr TryLoadFromDir(string dir, string fileName)
        {
            string path = Path.Combine(dir, fileName);
            if (!File.Exists(path))
                return IntPtr.Zero;

            try
            {
                return NativeLibrary.Load(path);
            }
            catch
            {
                // Don't block the default resolution path on preload failures (wrong arch, missing deps, etc.).
                return IntPtr.Zero;
            }
        }

        static string? FindNativeDir()
        {
            // 1) NuGet/runtime layout
            if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX) &&
                RuntimeInformation.OSArchitecture == Architecture.Arm64)
            {
                string p = Path.Combine(AppContext.BaseDirectory, "runtimes", "osx-arm64", "native");
                if (Directory.Exists(p))
                    return p;
            }
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows) &&
                RuntimeInformation.OSArchitecture == Architecture.X64)
            {
                string p = Path.Combine(AppContext.BaseDirectory, "runtimes", "win-x64", "native");
                if (Directory.Exists(p))
                    return p;
            }

            // 2) Repo layout: walk up from the current base dir and look for ./native/<platform>
            string platformSubdir = RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
                ? Path.Combine("native", "osx-arm64")
                : Path.Combine("native", "win-x64");

            DirectoryInfo? dir = new DirectoryInfo(AppContext.BaseDirectory);
            for (int i = 0; i < 10 && dir is not null; i++)
            {
                string candidate = Path.Combine(dir.FullName, platformSubdir);
                if (Directory.Exists(candidate))
                    return candidate;
                dir = dir.Parent;
            }

            return null;
        }
    }
}
