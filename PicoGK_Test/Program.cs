using PicoGKTest;

// 默认 headless 跑一遍，必要时可用 `--viewer` 打开窗口模式。
if (args.Contains("--viewer"))
    AdvancedExamples.Run();
else
    AdvancedExamples.RunHeadless();
