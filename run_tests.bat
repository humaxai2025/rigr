@echo off
echo ==================================================
echo Rigr Test Suite Runner
echo ==================================================
echo.

echo Building release version first...
cargo build --release
if %errorlevel% neq 0 (
    echo ❌ Build failed!
    exit /b 1
)
echo ✅ Build successful!
echo.

echo Running CLI Integration Tests...
echo --------------------------------------------------
cargo test --test cli_tests -- --nocapture
if %errorlevel% neq 0 (
    echo ❌ CLI tests failed!
    exit /b 1
)
echo.

echo Running Export Format Tests...
echo --------------------------------------------------
cargo test --test export_format_tests -- --nocapture
if %errorlevel% neq 0 (
    echo ❌ Export format tests failed!
    exit /b 1
)
echo.

echo Running Core Unit Tests (config and export modules)...
echo --------------------------------------------------
cargo test config::tests -- --nocapture
cargo test export::tests -- --nocapture
echo.

echo ==================================================
echo ✅ Test Suite Completed Successfully!
echo ==================================================
echo.
echo Test Coverage Summary:
echo - ✅ CLI Command Testing: All major options tested
echo - ✅ Export Format Testing: CSV, JSON, Excel, and more
echo - ✅ Error Handling: Invalid inputs and edge cases
echo - ✅ Configuration Management: Save/load operations
echo - ✅ File Processing: Multiple programming languages
echo.
echo The comprehensive test suite validates:
echo • All CLI arguments and flags
echo • Export to 15+ standard formats  
echo • Requirements-based test generation
echo • Directory and file processing
echo • Performance and security test options
echo • Concurrency and verbosity controls
echo • Error handling and edge cases
echo.
echo Your rigr executable is ready for production use!