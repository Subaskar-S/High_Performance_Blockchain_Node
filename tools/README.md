# 🔧 Setup Tools

This directory contains setup and installation tools for the blockchain node project.

## 📋 **Available Tools**

### **🪟 Windows Setup Tools**

| Tool | Description | Usage |
|------|-------------|-------|
| [`setup_windows.bat`](setup_windows.bat) | **Windows Batch Setup Script** - Automated setup for Windows Command Prompt | `setup_windows.bat` |
| [`setup_windows.ps1`](setup_windows.ps1) | **PowerShell Setup Script** - Advanced setup with better error handling | `.\setup_windows.ps1` |

## 🚀 **Quick Setup**

### **Windows 10/11 Setup**

#### **Option 1: Batch Script (Simple)**
```cmd
# Navigate to project directory
cd "path\to\blockchain-node"

# Run setup script
tools\setup_windows.bat
```

#### **Option 2: PowerShell Script (Recommended)**
```powershell
# Navigate to project directory
cd "path\to\blockchain-node"

# Run PowerShell setup
.\tools\setup_windows.ps1
```

## 🔧 **What These Tools Do**

### **Automated Setup Process**
1. **✅ Check Prerequisites**
   - Verify Rust installation
   - Check for Python (for test scripts)
   - Validate system dependencies

2. **📦 Install Dependencies**
   - Install Rust if not present
   - Install Python dependencies
   - Set up build tools if needed

3. **🏗️ Build Project**
   - Compile the blockchain node
   - Run basic tests
   - Verify installation

4. **✨ Final Verification**
   - Test basic functionality
   - Provide usage instructions
   - Show next steps

### **Setup Features**
- **Automatic Detection**: Checks what's already installed
- **Error Handling**: Provides clear error messages and solutions
- **Progress Feedback**: Shows setup progress and status
- **Verification**: Tests that everything works correctly

## 📋 **Prerequisites**

### **System Requirements**
- **OS**: Windows 10 or Windows 11
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 2GB free space for dependencies
- **Network**: Internet connection for downloads

### **Optional Prerequisites**
- **Git**: For version control (auto-installed if missing)
- **Visual Studio Build Tools**: For native dependencies (auto-installed if needed)

## 🛠️ **Manual Setup (If Automated Fails)**

### **1. Install Rust**
```powershell
# Download and install Rust
Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile "rustup-init.exe"
.\rustup-init.exe
```

### **2. Install Python**
```powershell
# Download Python installer
Invoke-WebRequest -Uri "https://www.python.org/ftp/python/3.11.7/python-3.11.7-amd64.exe" -OutFile "python-installer.exe"
.\python-installer.exe
```

### **3. Install Build Tools**
- Download Visual Studio Build Tools from Microsoft
- Install with "C++ build tools" workload

### **4. Build Project**
```cmd
# Build the blockchain node
cargo build --release

# Run tests
cargo test
```

## 🚨 **Troubleshooting**

### **Common Issues**

#### **Rust Installation Fails**
```cmd
# Solution 1: Manual installation
# Go to https://rustup.rs/ and download manually

# Solution 2: Use winget
winget install Rustlang.Rustup
```

#### **Build Fails with Linker Errors**
```cmd
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Install with "C++ build tools" workload
```

#### **Python Scripts Don't Work**
```cmd
# Ensure Python is in PATH
python --version

# Install requests library
pip install requests
```

#### **Permission Errors**
```cmd
# Run as Administrator
# Right-click Command Prompt/PowerShell -> "Run as administrator"
```

### **Getting Help**
- **Check Error Messages**: Setup scripts provide detailed error information
- **Run as Administrator**: Many setup operations require elevated privileges
- **Check Internet Connection**: Downloads require stable internet
- **Antivirus Software**: May interfere with downloads and compilation

## 📊 **Setup Verification**

### **Successful Setup Indicators**
- ✅ Rust compiler available (`rustc --version`)
- ✅ Cargo build tool available (`cargo --version`)
- ✅ Project builds successfully (`cargo build`)
- ✅ Tests pass (`cargo test`)
- ✅ Node starts (`cargo run -- --help`)

### **Post-Setup Testing**
```cmd
# Test single node
cargo run --release -- --node-id test-node --mode validator --dev-mode

# Test API (in another terminal)
curl http://localhost:8545 -X POST -H "Content-Type: application/json" -d "{\"jsonrpc\":\"2.0\",\"method\":\"blockchain_getNodeStatus\",\"params\":{},\"id\":1}"
```

## 🔄 **Updating Tools**

### **Tool Updates**
- Setup tools are updated with each project release
- Check for updates when encountering issues
- Tools are tested on latest Windows versions

### **Dependency Updates**
- Rust toolchain updates automatically
- Python dependencies updated as needed
- Build tools updated with Visual Studio releases

## 📞 **Support**

### **Getting Help**
- **GitHub Issues**: Report setup problems
- **Documentation**: Check main README and docs
- **Community**: GitHub Discussions for questions

### **Reporting Issues**
When reporting setup issues, include:
- Windows version
- Error messages (full text)
- Steps that failed
- System specifications

---

**These setup tools provide automated, reliable installation of the blockchain node development environment on Windows systems. They handle common issues and provide clear feedback throughout the setup process.**
