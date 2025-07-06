# 🤝 Contributing to Blockchain Node

Thank you for your interest in contributing to this blockchain node project! This document provides guidelines and information for contributors.

## 📋 **Table of Contents**

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [Contributing Process](#contributing-process)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Documentation](#documentation)
8. [Performance Considerations](#performance-considerations)
9. [Security Guidelines](#security-guidelines)

## 📜 **Code of Conduct**

This project adheres to a code of conduct that we expect all contributors to follow:

- **Be respectful**: Treat everyone with respect and kindness
- **Be inclusive**: Welcome newcomers and help them learn
- **Be collaborative**: Work together towards common goals
- **Be constructive**: Provide helpful feedback and suggestions
- **Be professional**: Maintain a professional tone in all interactions

## 🚀 **Getting Started**

### **Prerequisites**
- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git
- Basic understanding of blockchain concepts
- Familiarity with async Rust programming

### **Areas for Contribution**
- **Core Development**: Consensus, networking, storage improvements
- **Performance**: Optimization and benchmarking
- **Testing**: Unit tests, integration tests, fuzzing
- **Documentation**: API docs, tutorials, examples
- **Tooling**: Development tools, scripts, automation
- **Security**: Auditing, vulnerability fixes

## 🔧 **Development Setup**

### **1. Fork and Clone**
```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/blockchain-node.git
cd blockchain-node

# Add upstream remote
git remote add upstream https://github.com/ORIGINAL_OWNER/blockchain-node.git
```

### **2. Environment Setup**
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
rustup component add rustfmt clippy
cargo install cargo-watch cargo-tarpaulin cargo-audit

# Run setup script
./scripts/build_and_test.sh setup-dev  # Linux/Mac
scripts\build_and_test.bat setup-dev   # Windows
```

### **3. Build and Test**
```bash
# Build the project
cargo build

# Run tests
cargo test

# Run all checks
./scripts/build_and_test.sh check-all
```

## 🔄 **Contributing Process**

### **1. Issue First**
- Check existing issues before creating new ones
- For bugs: provide reproduction steps and environment details
- For features: discuss the proposal and get feedback
- Use issue templates when available

### **2. Branch Strategy**
```bash
# Create feature branch from main
git checkout main
git pull upstream main
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-description
```

### **3. Development Workflow**
```bash
# Make your changes
# Write tests for new functionality
# Update documentation as needed

# Run checks locally
cargo fmt
cargo clippy
cargo test

# Commit with descriptive messages
git add .
git commit -m "feat: add new consensus optimization

- Implement batch message processing
- Add performance benchmarks
- Update documentation

Closes #123"
```

### **4. Pull Request Process**
1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request**:
   - Use the PR template
   - Provide clear description of changes
   - Link related issues
   - Add screenshots/examples if applicable

3. **Code Review**:
   - Address reviewer feedback
   - Keep PR focused and atomic
   - Rebase if requested

4. **Merge**:
   - Squash commits if requested
   - Ensure CI passes
   - Wait for maintainer approval

## 📝 **Coding Standards**

### **Rust Style Guide**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write idiomatic Rust code

### **Code Organization**
```rust
// File structure
src/
├── main.rs              // Entry point
├── lib.rs               // Library root (if applicable)
├── types.rs             // Core types
├── module/              // Feature modules
│   ├── mod.rs           // Module interface
│   ├── implementation.rs // Implementation
│   └── tests.rs         // Module tests
└── utils.rs             // Utility functions
```

### **Naming Conventions**
- **Functions**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Files**: `snake_case.rs`

### **Documentation**
```rust
/// Brief description of the function.
///
/// Longer description with more details about what the function does,
/// its parameters, return value, and any important notes.
///
/// # Arguments
///
/// * `param1` - Description of parameter 1
/// * `param2` - Description of parameter 2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of possible errors
///
/// # Examples
///
/// ```
/// use blockchain_node::example_function;
/// 
/// let result = example_function(42, "test");
/// assert_eq!(result, expected_value);
/// ```
pub fn example_function(param1: i32, param2: &str) -> Result<String> {
    // Implementation
}
```

## 🧪 **Testing Guidelines**

### **Test Categories**
1. **Unit Tests**: Test individual functions/methods
2. **Integration Tests**: Test component interactions
3. **Performance Tests**: Benchmark critical paths
4. **Property Tests**: Test invariants with random inputs

### **Test Structure**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
    
    #[tokio::test]
    async fn test_async_function() {
        // Test async functions
    }
}
```

### **Test Requirements**
- All new code must have tests
- Aim for >80% code coverage
- Test both success and error cases
- Use descriptive test names
- Include edge cases and boundary conditions

### **Running Tests**
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with coverage
cargo tarpaulin --all-features

# Run benchmarks
cargo bench
```

## 📚 **Documentation**

### **Documentation Types**
1. **Code Documentation**: Inline docs with `///`
2. **API Documentation**: Public interface documentation
3. **Architecture Documentation**: High-level design docs
4. **User Documentation**: Usage guides and tutorials

### **Documentation Standards**
- Document all public APIs
- Include examples in documentation
- Keep documentation up-to-date with code changes
- Use clear, concise language
- Provide context and rationale

### **Generating Documentation**
```bash
# Generate and open documentation
cargo doc --open

# Check documentation
cargo doc --no-deps
```

## ⚡ **Performance Considerations**

### **Performance Guidelines**
- Profile before optimizing
- Measure performance impact of changes
- Consider memory allocation patterns
- Use appropriate data structures
- Minimize lock contention

### **Benchmarking**
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("function_name", |b| {
        b.iter(|| {
            // Code to benchmark
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

### **Performance Testing**
```bash
# Run benchmarks
cargo bench

# Profile with perf (Linux)
perf record --call-graph=dwarf target/release/blockchain-node
perf report
```

## 🔒 **Security Guidelines**

### **Security Practices**
- Never commit secrets or private keys
- Validate all inputs
- Use secure random number generation
- Follow cryptographic best practices
- Handle errors securely

### **Security Review**
- All cryptographic code requires review
- Security-sensitive changes need extra scrutiny
- Run security audits regularly
- Report security issues privately

### **Security Tools**
```bash
# Security audit
cargo audit

# Check for common issues
cargo clippy -- -W clippy::all

# Dependency analysis
cargo deny check
```

## 🐛 **Bug Reports**

### **Bug Report Template**
```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. See error

**Expected behavior**
What you expected to happen.

**Environment:**
- OS: [e.g. Ubuntu 20.04]
- Rust version: [e.g. 1.70.0]
- Project version: [e.g. 0.1.0]

**Additional context**
Any other context about the problem.
```

## 💡 **Feature Requests**

### **Feature Request Template**
```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Alternative solutions or features you've considered.

**Additional context**
Any other context or screenshots about the feature request.
```

## 📞 **Getting Help**

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Documentation**: Check existing documentation first
- **Code Review**: Ask for feedback on complex changes

## 🎉 **Recognition**

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes for significant contributions
- GitHub contributor statistics

Thank you for contributing to the blockchain node project! 🚀
