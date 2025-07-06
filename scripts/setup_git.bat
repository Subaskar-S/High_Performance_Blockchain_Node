@echo off
REM Git setup and GitHub push script for blockchain node project (Windows)

setlocal enabledelayedexpansion

echo ==========================================
echo Blockchain Node - Git Setup Script
echo ==========================================
echo.

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

REM Check if git is installed
:check_git
where git >nul 2>nul
if %errorlevel% neq 0 (
    call :print_error "Git is not installed. Please install Git first."
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('git --version') do set git_version=%%i
call :print_success "Git is installed: !git_version!"
goto :eof

REM Initialize git repository
:init_git
call :print_status "Initializing Git repository..."

if exist ".git" (
    call :print_warning "Git repository already exists"
) else (
    git init
    call :print_success "Git repository initialized"
)
goto :eof

REM Configure git user
:configure_git
call :print_status "Checking Git configuration..."

git config user.name >nul 2>nul
if %errorlevel% neq 0 (
    set /p git_username="Enter your Git username: "
    git config user.name "!git_username!"
    call :print_success "Git username set to: !git_username!"
) else (
    for /f "tokens=*" %%i in ('git config user.name') do set current_username=%%i
    call :print_success "Git username already configured: !current_username!"
)

git config user.email >nul 2>nul
if %errorlevel% neq 0 (
    set /p git_email="Enter your Git email: "
    git config user.email "!git_email!"
    call :print_success "Git email set to: !git_email!"
) else (
    for /f "tokens=*" %%i in ('git config user.email') do set current_email=%%i
    call :print_success "Git email already configured: !current_email!"
)
goto :eof

REM Add all files
:add_files
call :print_status "Adding files to Git..."

git add .

call :print_status "Git status:"
git status --short

call :print_success "Files added to Git staging area"
goto :eof

REM Create initial commit
:create_commit
call :print_status "Creating initial commit..."

REM Check if there are any changes to commit
git diff --staged --quiet
if %errorlevel% equ 0 (
    call :print_warning "No changes to commit"
    goto :eof
)

REM Create commit with detailed message
git commit -m "Initial commit: High-throughput blockchain node" -m "" -m "Features:" -m "- Byzantine Fault Tolerant (BFT) consensus with PBFT" -m "- High-performance P2P networking with libp2p" -m "- RocksDB-based persistent storage" -m "- Priority-based transaction mempool" -m "- Comprehensive validation engine" -m "- Prometheus metrics and monitoring" -m "- JSON-RPC API for external interactions" -m "- Multi-node testnet simulation" -m "- Comprehensive documentation and testing" -m "" -m "Architecture:" -m "- Modular design with clean separation of concerns" -m "- Async/await with Tokio runtime" -m "- Designed for 1000+ peers and 10,000+ TPS" -m "- Byzantine fault tolerance up to f=(n-1)/3 malicious nodes" -m "- Sub-10ms block propagation in LAN environments"

call :print_success "Initial commit created"
goto :eof

REM Add GitHub remote
:add_remote
call :print_status "Setting up GitHub remote..."

echo.
call :print_warning "Before proceeding, please:"
echo 1. Create a new repository on GitHub
echo 2. Copy the repository URL (HTTPS or SSH)
echo.

set /p repo_url="Enter your GitHub repository URL: "

if "!repo_url!"=="" (
    call :print_error "Repository URL cannot be empty"
    pause
    exit /b 1
)

REM Check if origin remote already exists
git remote get-url origin >nul 2>nul
if %errorlevel% equ 0 (
    call :print_warning "Origin remote already exists. Updating..."
    git remote set-url origin "!repo_url!"
) else (
    git remote add origin "!repo_url!"
)

call :print_success "GitHub remote added: !repo_url!"
goto :eof

REM Push to GitHub
:push_to_github
call :print_status "Pushing to GitHub..."

REM Check if we have commits to push
git log --oneline -1 >nul 2>nul
if %errorlevel% neq 0 (
    call :print_error "No commits to push. Please create a commit first."
    pause
    exit /b 1
)

call :print_status "Pushing to origin main..."

REM Set upstream and push
git push -u origin main
if %errorlevel% equ 0 (
    call :print_success "Successfully pushed to GitHub!"
) else (
    call :print_warning "Push failed. This might be because:"
    echo 1. The repository already has content
    echo 2. Authentication failed
    echo 3. Network issues
    echo.
    call :print_status "Trying to pull and merge first..."
    
    git pull origin main --allow-unrelated-histories
    if %errorlevel% equ 0 (
        call :print_status "Merged remote changes. Pushing again..."
        git push -u origin main
        call :print_success "Successfully pushed to GitHub!"
    ) else (
        call :print_error "Failed to push to GitHub. Please resolve conflicts manually."
        pause
        exit /b 1
    )
)
goto :eof

REM Create and push tags
:create_tags
call :print_status "Creating version tags..."

git tag -a v0.1.0 -m "Initial release v0.1.0" -m "" -m "High-throughput blockchain node with BFT consensus" -m "- PBFT consensus algorithm" -m "- libp2p networking" -m "- RocksDB storage" -m "- Comprehensive testing and documentation"

git push origin --tags

call :print_success "Version tag v0.1.0 created and pushed"
goto :eof

REM Show next steps
:show_next_steps
echo.
call :print_success "🎉 Repository successfully set up on GitHub!"
echo.
call :print_status "Next steps:"
echo 1. Visit your GitHub repository to verify the upload
echo 2. Set up branch protection rules (recommended)
echo 3. Configure GitHub Actions secrets if needed:
echo    - DOCKER_USERNAME (for Docker Hub)
echo    - DOCKER_PASSWORD (for Docker Hub)
echo 4. Enable GitHub Pages for documentation (optional)
echo 5. Set up issue and PR templates
echo.

for /f "tokens=*" %%i in ('git remote get-url origin') do set repo_url=%%i
call :print_status "Repository URL: !repo_url!"
echo.
call :print_status "To continue development:"
echo git checkout -b feature/your-feature
echo # Make changes
echo git add .
echo git commit -m "Add new feature"
echo git push origin feature/your-feature
echo # Create pull request on GitHub
goto :eof

REM Main execution
call :check_git
call :init_git
call :configure_git
call :add_files
call :create_commit
call :add_remote
call :push_to_github
call :create_tags
call :show_next_steps

echo.
pause
