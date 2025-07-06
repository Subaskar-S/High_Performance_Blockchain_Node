#!/bin/bash

# Build and test script for the blockchain node project
# This script provides various commands for building, testing, and running the blockchain node

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    print_status "Rust version: $(rustc --version)"
    print_status "Cargo version: $(cargo --version)"
}

# Function to check dependencies
check_dependencies() {
    print_status "Checking system dependencies..."
    
    # Check for required system libraries
    if command -v pkg-config &> /dev/null; then
        print_success "pkg-config found"
    else
        print_warning "pkg-config not found - may be needed for some dependencies"
    fi
    
    # Check for Python (for test scripts)
    if command -v python3 &> /dev/null; then
        print_success "Python 3 found: $(python3 --version)"
    else
        print_warning "Python 3 not found - test scripts may not work"
    fi
}

# Function to clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    cargo clean
    rm -rf testnet_data/
    print_success "Clean completed"
}

# Function to format code
format() {
    print_status "Formatting code..."
    cargo fmt
    print_success "Code formatting completed"
}

# Function to run linting
lint() {
    print_status "Running linting..."
    cargo clippy -- -D warnings
    print_success "Linting completed"
}

# Function to build the project
build() {
    print_status "Building project..."
    cargo build
    print_success "Build completed"
}

# Function to build release version
build_release() {
    print_status "Building release version..."
    cargo build --release
    print_success "Release build completed"
}

# Function to run tests
test() {
    print_status "Running tests..."
    cargo test
    print_success "Tests completed"
}

# Function to run tests with output
test_verbose() {
    print_status "Running tests with verbose output..."
    RUST_LOG=debug cargo test -- --nocapture
    print_success "Verbose tests completed"
}

# Function to run benchmarks
benchmark() {
    print_status "Running benchmarks..."
    cargo bench
    print_success "Benchmarks completed"
}

# Function to check code coverage (requires cargo-tarpaulin)
coverage() {
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_warning "cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin
    fi
    
    print_status "Running code coverage analysis..."
    cargo tarpaulin --out Html --output-dir coverage/
    print_success "Coverage report generated in coverage/"
}

# Function to run security audit
audit() {
    if ! command -v cargo-audit &> /dev/null; then
        print_warning "cargo-audit not found. Installing..."
        cargo install cargo-audit
    fi
    
    print_status "Running security audit..."
    cargo audit
    print_success "Security audit completed"
}

# Function to run a single node
run_single() {
    print_status "Starting single blockchain node..."
    cargo run --release -- \
        --node-id validator-1 \
        --mode validator \
        --listen-addr "/ip4/0.0.0.0/tcp/8000" \
        --rpc-port 9000 \
        --metrics-port 9100 \
        --dev-mode
}

# Function to run testnet
run_testnet() {
    print_status "Starting 5-node testnet..."
    if [ -f "scripts/run_testnet.py" ]; then
        python3 scripts/run_testnet.py --nodes 5
    else
        print_error "Testnet script not found"
        exit 1
    fi
}

# Function to run all checks
check_all() {
    print_status "Running all checks..."
    format
    lint
    build
    test
    print_success "All checks passed!"
}

# Function to setup development environment
setup_dev() {
    print_status "Setting up development environment..."
    
    # Install useful cargo tools
    cargo install cargo-watch cargo-expand cargo-tree
    
    # Install rustfmt and clippy if not already installed
    rustup component add rustfmt clippy
    
    print_success "Development environment setup completed"
}

# Function to generate documentation
docs() {
    print_status "Generating documentation..."
    cargo doc --open
    print_success "Documentation generated and opened"
}

# Function to show help
show_help() {
    echo "Blockchain Node Build and Test Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  check-rust     Check if Rust is installed"
    echo "  check-deps     Check system dependencies"
    echo "  clean          Clean build artifacts"
    echo "  format         Format code with rustfmt"
    echo "  lint           Run clippy linting"
    echo "  build          Build the project"
    echo "  build-release  Build release version"
    echo "  test           Run tests"
    echo "  test-verbose   Run tests with verbose output"
    echo "  benchmark      Run performance benchmarks"
    echo "  coverage       Generate code coverage report"
    echo "  audit          Run security audit"
    echo "  run-single     Run a single blockchain node"
    echo "  run-testnet    Run 5-node testnet"
    echo "  check-all      Run format, lint, build, and test"
    echo "  setup-dev      Setup development environment"
    echo "  docs           Generate and open documentation"
    echo "  help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 check-all      # Run all checks before committing"
    echo "  $0 run-single     # Start a single node for testing"
    echo "  $0 run-testnet    # Start a full testnet"
}

# Main script logic
case "${1:-help}" in
    "check-rust")
        check_rust
        ;;
    "check-deps")
        check_dependencies
        ;;
    "clean")
        clean
        ;;
    "format")
        format
        ;;
    "lint")
        lint
        ;;
    "build")
        build
        ;;
    "build-release")
        build_release
        ;;
    "test")
        test
        ;;
    "test-verbose")
        test_verbose
        ;;
    "benchmark")
        benchmark
        ;;
    "coverage")
        coverage
        ;;
    "audit")
        audit
        ;;
    "run-single")
        run_single
        ;;
    "run-testnet")
        run_testnet
        ;;
    "check-all")
        check_all
        ;;
    "setup-dev")
        setup_dev
        ;;
    "docs")
        docs
        ;;
    "help"|*)
        show_help
        ;;
esac
