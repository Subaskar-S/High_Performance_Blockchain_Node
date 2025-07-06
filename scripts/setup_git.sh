#!/bin/bash

# Git setup and GitHub push script for blockchain node project

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Function to check if git is installed
check_git() {
    if ! command -v git &> /dev/null; then
        print_error "Git is not installed. Please install Git first."
        exit 1
    fi
    print_success "Git is installed: $(git --version)"
}

# Function to initialize git repository
init_git() {
    print_status "Initializing Git repository..."
    
    if [ -d ".git" ]; then
        print_warning "Git repository already exists"
    else
        git init
        print_success "Git repository initialized"
    fi
}

# Function to configure git (if not already configured)
configure_git() {
    print_status "Checking Git configuration..."
    
    if ! git config user.name &> /dev/null; then
        read -p "Enter your Git username: " git_username
        git config user.name "$git_username"
        print_success "Git username set to: $git_username"
    else
        print_success "Git username already configured: $(git config user.name)"
    fi
    
    if ! git config user.email &> /dev/null; then
        read -p "Enter your Git email: " git_email
        git config user.email "$git_email"
        print_success "Git email set to: $git_email"
    else
        print_success "Git email already configured: $(git config user.email)"
    fi
}

# Function to add all files
add_files() {
    print_status "Adding files to Git..."
    
    # Add all files
    git add .
    
    # Show status
    print_status "Git status:"
    git status --short
    
    print_success "Files added to Git staging area"
}

# Function to create initial commit
create_commit() {
    print_status "Creating initial commit..."
    
    # Check if there are any changes to commit
    if git diff --staged --quiet; then
        print_warning "No changes to commit"
        return
    fi
    
    # Create commit with detailed message
    git commit -m "Initial commit: High-throughput blockchain node

Features:
- Byzantine Fault Tolerant (BFT) consensus with PBFT
- High-performance P2P networking with libp2p
- RocksDB-based persistent storage
- Priority-based transaction mempool
- Comprehensive validation engine
- Prometheus metrics and monitoring
- JSON-RPC API for external interactions
- Multi-node testnet simulation
- Comprehensive documentation and testing

Architecture:
- Modular design with clean separation of concerns
- Async/await with Tokio runtime
- Designed for 1000+ peers and 10,000+ TPS
- Byzantine fault tolerance up to f=(n-1)/3 malicious nodes
- Sub-10ms block propagation in LAN environments

Components:
- Consensus engine with PBFT, leader election, view changes
- Network layer with gossip protocol and peer discovery
- Storage layer with blocks, state, and transaction stores
- Transaction pool with priority queue and validation
- Metrics collection with Prometheus integration
- CLI interface with validator/observer modes
- Testing suite with benchmarks and integration tests"

    print_success "Initial commit created"
}

# Function to add GitHub remote
add_remote() {
    print_status "Setting up GitHub remote..."
    
    echo ""
    print_warning "Before proceeding, please:"
    echo "1. Create a new repository on GitHub"
    echo "2. Copy the repository URL (HTTPS or SSH)"
    echo ""
    
    read -p "Enter your GitHub repository URL: " repo_url
    
    if [ -z "$repo_url" ]; then
        print_error "Repository URL cannot be empty"
        exit 1
    fi
    
    # Check if origin remote already exists
    if git remote get-url origin &> /dev/null; then
        print_warning "Origin remote already exists. Updating..."
        git remote set-url origin "$repo_url"
    else
        git remote add origin "$repo_url"
    fi
    
    print_success "GitHub remote added: $repo_url"
}

# Function to push to GitHub
push_to_github() {
    print_status "Pushing to GitHub..."
    
    # Check if we have commits to push
    if ! git log --oneline -1 &> /dev/null; then
        print_error "No commits to push. Please create a commit first."
        exit 1
    fi
    
    # Push to GitHub
    print_status "Pushing to origin main..."
    
    # Set upstream and push
    if git push -u origin main; then
        print_success "Successfully pushed to GitHub!"
    else
        print_warning "Push failed. This might be because:"
        echo "1. The repository already has content"
        echo "2. Authentication failed"
        echo "3. Network issues"
        echo ""
        print_status "Trying to pull and merge first..."
        
        # Try to pull and merge
        if git pull origin main --allow-unrelated-histories; then
            print_status "Merged remote changes. Pushing again..."
            git push -u origin main
            print_success "Successfully pushed to GitHub!"
        else
            print_error "Failed to push to GitHub. Please resolve conflicts manually."
            exit 1
        fi
    fi
}

# Function to create and push tags
create_tags() {
    print_status "Creating version tags..."
    
    # Create initial version tag
    git tag -a v0.1.0 -m "Initial release v0.1.0

High-throughput blockchain node with BFT consensus
- PBFT consensus algorithm
- libp2p networking
- RocksDB storage
- Comprehensive testing and documentation"
    
    # Push tags
    git push origin --tags
    
    print_success "Version tag v0.1.0 created and pushed"
}

# Function to show next steps
show_next_steps() {
    echo ""
    print_success "🎉 Repository successfully set up on GitHub!"
    echo ""
    print_status "Next steps:"
    echo "1. Visit your GitHub repository to verify the upload"
    echo "2. Set up branch protection rules (recommended)"
    echo "3. Configure GitHub Actions secrets if needed:"
    echo "   - DOCKER_USERNAME (for Docker Hub)"
    echo "   - DOCKER_PASSWORD (for Docker Hub)"
    echo "4. Enable GitHub Pages for documentation (optional)"
    echo "5. Set up issue and PR templates"
    echo ""
    print_status "Repository URL: $(git remote get-url origin)"
    echo ""
    print_status "To continue development:"
    echo "git checkout -b feature/your-feature"
    echo "# Make changes"
    echo "git add ."
    echo "git commit -m 'Add new feature'"
    echo "git push origin feature/your-feature"
    echo "# Create pull request on GitHub"
}

# Main execution
main() {
    echo "=========================================="
    echo "Blockchain Node - Git Setup Script"
    echo "=========================================="
    echo ""
    
    # Check prerequisites
    check_git
    
    # Initialize git if needed
    init_git
    
    # Configure git user
    configure_git
    
    # Add files to git
    add_files
    
    # Create initial commit
    create_commit
    
    # Add GitHub remote
    add_remote
    
    # Push to GitHub
    push_to_github
    
    # Create version tags
    create_tags
    
    # Show next steps
    show_next_steps
}

# Run main function
main "$@"
