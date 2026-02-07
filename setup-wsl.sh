#!/bin/bash
# WSL Build Environment Setup for Verso Graph Browser
# Run this script once in a fresh WSL Ubuntu installation
# Usage: bash setup-wsl.sh

set -e  # Exit on first error

echo "🚀 Setting up Verso Graph Browser build environment in WSL..."
echo ""

# Update package lists
echo "📦 Updating package lists..."
sudo apt update

# Install system dependencies
echo "📦 Installing system dependencies..."
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libfontconfig1-dev \
    libfreetype6-dev \
    libxrender-dev \
    libxcb1-dev \
    python3 \
    python3-pip \
    curl \
    git

# Install uv (Python package manager)
echo "📦 Installing uv package manager..."
curl -LsSf https://astral.sh/uv/install.sh | sh

# Add uv to PATH for current session
export PATH="$HOME/.local/bin:$PATH"

# Verify installations
echo ""
echo "✅ Verifying installations..."
python3 --version
git --version
uv --version
curl --version

echo ""
echo "✨ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Navigate to Servo: cd /mnt/c/Users/mark_/Code/servo"
echo "  2. Install Servo build dependencies: ./mach bootstrap"
echo "  3. Build verso-graph: ./mach build -r verso-graph"
echo ""
echo "First build will take 15-30 minutes. Subsequent builds are much faster."
