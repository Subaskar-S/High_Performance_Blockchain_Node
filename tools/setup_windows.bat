@echo off
echo ========================================
echo Blockchain Node Windows Setup Script
echo ========================================
echo.

REM Check if Rust is installed
echo [1/5] Checking Rust installation...
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust/Cargo not found!
    echo.
    echo Please install Rust first:
    echo 1. Go to https://rustup.rs/
    echo 2. Download rustup-init.exe
    echo 3. Run it and choose default installation
    echo 4. Restart this script after installation
    echo.
    pause
    exit /b 1
) else (
    echo [SUCCESS] Rust is installed
    cargo --version
)

echo.
echo [2/5] Checking Python installation...
where python >nul 2>nul
if %errorlevel% neq 0 (
    echo [WARNING] Python not found. Test scripts may not work.
    echo You can install Python from: https://www.python.org/downloads/
) else (
    echo [SUCCESS] Python is installed
    python --version
)

echo.
echo [3/5] Installing Python dependencies...
pip install requests >nul 2>nul
if %errorlevel% equ 0 (
    echo [SUCCESS] Python requests library installed
) else (
    echo [WARNING] Could not install Python requests library
)

echo.
echo [4/5] Building the blockchain node...
echo This may take several minutes for the first build...
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Build failed!
    echo.
    echo Common solutions:
    echo 1. Install Visual Studio Build Tools
    echo 2. Install Visual Studio Community with C++ workload
    echo 3. Run: rustup update
    echo.
    pause
    exit /b 1
) else (
    echo [SUCCESS] Build completed successfully!
)

echo.
echo [5/5] Running basic tests...
cargo test --lib
if %errorlevel% neq 0 (
    echo [WARNING] Some tests failed, but the build is complete
) else (
    echo [SUCCESS] Tests passed!
)

echo.
echo ========================================
echo Setup Complete!
echo ========================================
echo.
echo You can now run:
echo.
echo Single node:
echo   cargo run --release -- --node-id validator-1 --mode validator
echo.
echo 5-node testnet:
echo   python scripts\run_testnet.py --nodes 5
echo.
echo Build and test commands:
echo   scripts\build_and_test.bat help
echo.
pause
