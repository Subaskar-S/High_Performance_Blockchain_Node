# Blockchain Node Windows Setup Script (PowerShell)
# Run this script in PowerShell as Administrator

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Blockchain Node Windows Setup Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Function to check if a command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Step 1: Check Rust installation
Write-Host "[1/6] Checking Rust installation..." -ForegroundColor Yellow
if (Test-Command "cargo") {
    Write-Host "[SUCCESS] Rust is installed" -ForegroundColor Green
    cargo --version
} else {
    Write-Host "[ERROR] Rust/Cargo not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Installing Rust automatically..." -ForegroundColor Yellow
    
    try {
        # Download and install Rust
        $rustupUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
        $rustupPath = "$env:TEMP\rustup-init.exe"
        
        Write-Host "Downloading Rust installer..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
        
        Write-Host "Running Rust installer..." -ForegroundColor Yellow
        Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait
        
        # Refresh environment variables
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
        
        Write-Host "[SUCCESS] Rust installed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "[ERROR] Failed to install Rust automatically." -ForegroundColor Red
        Write-Host "Please install manually from: https://rustup.rs/" -ForegroundColor Yellow
        Read-Host "Press Enter to continue after installing Rust"
    }
}

Write-Host ""

# Step 2: Check Python installation
Write-Host "[2/6] Checking Python installation..." -ForegroundColor Yellow
if (Test-Command "python") {
    Write-Host "[SUCCESS] Python is installed" -ForegroundColor Green
    python --version
} else {
    Write-Host "[WARNING] Python not found." -ForegroundColor Yellow
    Write-Host "Test scripts may not work without Python." -ForegroundColor Yellow
    Write-Host "You can install Python from: https://www.python.org/downloads/" -ForegroundColor Yellow
}

Write-Host ""

# Step 3: Install Python dependencies
Write-Host "[3/6] Installing Python dependencies..." -ForegroundColor Yellow
if (Test-Command "pip") {
    try {
        pip install requests | Out-Null
        Write-Host "[SUCCESS] Python requests library installed" -ForegroundColor Green
    } catch {
        Write-Host "[WARNING] Could not install Python requests library" -ForegroundColor Yellow
    }
} else {
    Write-Host "[WARNING] pip not found, skipping Python dependencies" -ForegroundColor Yellow
}

Write-Host ""

# Step 4: Check for Visual Studio Build Tools
Write-Host "[4/6] Checking for C++ build tools..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsInstalls = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | ConvertFrom-Json
    if ($vsInstalls.Count -gt 0) {
        Write-Host "[SUCCESS] Visual Studio C++ build tools found" -ForegroundColor Green
    } else {
        Write-Host "[WARNING] Visual Studio C++ build tools not found" -ForegroundColor Yellow
        Write-Host "You may need to install Visual Studio Build Tools if compilation fails" -ForegroundColor Yellow
    }
} else {
    Write-Host "[WARNING] Visual Studio installer not found" -ForegroundColor Yellow
    Write-Host "You may need to install Visual Studio Build Tools if compilation fails" -ForegroundColor Yellow
}

Write-Host ""

# Step 5: Build the project
Write-Host "[5/6] Building the blockchain node..." -ForegroundColor Yellow
Write-Host "This may take several minutes for the first build..." -ForegroundColor Yellow

try {
    cargo build --release
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[SUCCESS] Build completed successfully!" -ForegroundColor Green
    } else {
        throw "Build failed with exit code $LASTEXITCODE"
    }
} catch {
    Write-Host "[ERROR] Build failed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Common solutions:" -ForegroundColor Yellow
    Write-Host "1. Install Visual Studio Build Tools from:" -ForegroundColor Yellow
    Write-Host "   https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Yellow
    Write-Host "2. Install Visual Studio Community with C++ workload" -ForegroundColor Yellow
    Write-Host "3. Run: rustup update" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to continue"
    exit 1
}

Write-Host ""

# Step 6: Run basic tests
Write-Host "[6/6] Running basic tests..." -ForegroundColor Yellow
try {
    cargo test --lib
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[SUCCESS] Tests passed!" -ForegroundColor Green
    } else {
        Write-Host "[WARNING] Some tests failed, but the build is complete" -ForegroundColor Yellow
    }
} catch {
    Write-Host "[WARNING] Could not run tests, but the build is complete" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Setup Complete!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "You can now run:" -ForegroundColor Green
Write-Host ""
Write-Host "Single node:" -ForegroundColor Yellow
Write-Host "  cargo run --release -- --node-id validator-1 --mode validator" -ForegroundColor White
Write-Host ""
Write-Host "5-node testnet:" -ForegroundColor Yellow
Write-Host "  python scripts\run_testnet.py --nodes 5" -ForegroundColor White
Write-Host ""
Write-Host "Build and test commands:" -ForegroundColor Yellow
Write-Host "  scripts\build_and_test.bat help" -ForegroundColor White
Write-Host ""

Read-Host "Press Enter to exit"
