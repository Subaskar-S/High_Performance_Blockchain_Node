@echo off
REM Build and test script for the blockchain node project (Windows version)
REM This script provides various commands for building, testing, and running the blockchain node

setlocal enabledelayedexpansion

REM Function to print status messages
:print_status
echo [INFO] %~1
goto :eof

:print_success
echo [SUCCESS] %~1
goto :eof

:print_warning
echo [WARNING] %~1
goto :eof

:print_error
echo [ERROR] %~1
goto :eof

REM Function to check if Rust is installed
:check_rust
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    call :print_error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit /b 1
)

for /f "tokens=*" %%i in ('rustc --version') do set rust_version=%%i
for /f "tokens=*" %%i in ('cargo --version') do set cargo_version=%%i
call :print_status "Rust version: !rust_version!"
call :print_status "Cargo version: !cargo_version!"
goto :eof

REM Function to clean build artifacts
:clean
call :print_status "Cleaning build artifacts..."
cargo clean
if exist testnet_data rmdir /s /q testnet_data
call :print_success "Clean completed"
goto :eof

REM Function to format code
:format
call :print_status "Formatting code..."
cargo fmt
call :print_success "Code formatting completed"
goto :eof

REM Function to run linting
:lint
call :print_status "Running linting..."
cargo clippy -- -D warnings
call :print_success "Linting completed"
goto :eof

REM Function to build the project
:build
call :print_status "Building project..."
cargo build
call :print_success "Build completed"
goto :eof

REM Function to build release version
:build_release
call :print_status "Building release version..."
cargo build --release
call :print_success "Release build completed"
goto :eof

REM Function to run tests
:test
call :print_status "Running tests..."
cargo test
call :print_success "Tests completed"
goto :eof

REM Function to run tests with output
:test_verbose
call :print_status "Running tests with verbose output..."
set RUST_LOG=debug
cargo test -- --nocapture
call :print_success "Verbose tests completed"
goto :eof

REM Function to run benchmarks
:benchmark
call :print_status "Running benchmarks..."
cargo bench
call :print_success "Benchmarks completed"
goto :eof

REM Function to run a single node
:run_single
call :print_status "Starting single blockchain node..."
cargo run --release -- --node-id validator-1 --mode validator --listen-addr "/ip4/0.0.0.0/tcp/8000" --rpc-port 9000 --metrics-port 9100 --dev-mode
goto :eof

REM Function to run testnet
:run_testnet
call :print_status "Starting 5-node testnet..."
if exist "scripts\run_testnet.py" (
    python scripts\run_testnet.py --nodes 5
) else (
    call :print_error "Testnet script not found"
    exit /b 1
)
goto :eof

REM Function to run all checks
:check_all
call :print_status "Running all checks..."
call :format
call :lint
call :build
call :test
call :print_success "All checks passed!"
goto :eof

REM Function to generate documentation
:docs
call :print_status "Generating documentation..."
cargo doc --open
call :print_success "Documentation generated and opened"
goto :eof

REM Function to show help
:show_help
echo Blockchain Node Build and Test Script (Windows)
echo.
echo Usage: %~nx0 [COMMAND]
echo.
echo Commands:
echo   check-rust     Check if Rust is installed
echo   clean          Clean build artifacts
echo   format         Format code with rustfmt
echo   lint           Run clippy linting
echo   build          Build the project
echo   build-release  Build release version
echo   test           Run tests
echo   test-verbose   Run tests with verbose output
echo   benchmark      Run performance benchmarks
echo   run-single     Run a single blockchain node
echo   run-testnet    Run 5-node testnet
echo   check-all      Run format, lint, build, and test
echo   docs           Generate and open documentation
echo   help           Show this help message
echo.
echo Examples:
echo   %~nx0 check-all      # Run all checks before committing
echo   %~nx0 run-single     # Start a single node for testing
echo   %~nx0 run-testnet    # Start a full testnet
goto :eof

REM Main script logic
if "%1"=="" goto show_help
if "%1"=="check-rust" goto check_rust
if "%1"=="clean" goto clean
if "%1"=="format" goto format
if "%1"=="lint" goto lint
if "%1"=="build" goto build
if "%1"=="build-release" goto build_release
if "%1"=="test" goto test
if "%1"=="test-verbose" goto test_verbose
if "%1"=="benchmark" goto benchmark
if "%1"=="run-single" goto run_single
if "%1"=="run-testnet" goto run_testnet
if "%1"=="check-all" goto check_all
if "%1"=="docs" goto docs
if "%1"=="help" goto show_help

REM Default case
goto show_help
