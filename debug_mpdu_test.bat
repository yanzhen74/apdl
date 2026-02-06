@echo off
cd /d "d:\user\yqd\project\apdl\code\apdl-poem"

echo 正在构建MPDU综合测试...
cargo build --test mpdu_comprehensive_test

if %ERRORLEVEL% NEQ 0 (
    echo 构建失败，请检查错误
    pause
    exit /b 1
)

echo.
echo 正在运行MPDU综合测试...
cargo test test_mpdu_comprehensive_scenario -- --nocapture

if %ERRORLEVEL% NEQ 0 (
    echo 测试失败，请检查错误
    pause
    exit /b 1
)

echo.
echo 测试运行完成！
pause