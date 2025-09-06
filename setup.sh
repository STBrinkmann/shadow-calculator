#!/bin/bash

# Shadow Calculator Setup Script
# This script will help you set up the project step by step

set -e  # Exit on error

echo "================================================"
echo "  Shadow Calculator - Automated Setup Script   "
echo "================================================"
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to print colored output
print_success() {
    echo -e "\033[0;32m✓ $1\033[0m"
}

print_error() {
    echo -e "\033[0;31m✗ $1\033[0m"
}

print_info() {
    echo -e "\033[0;34mℹ $1\033[0m"
}

# Step 1: Check prerequisites
echo "Step 1: Checking prerequisites..."
echo "--------------------------------"

# Check Rust
if command_exists rustc; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    print_success "Rust installed: $RUST_VERSION"
else
    print_error "Rust not found!"
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

# Check Node.js
if command_exists node; then
    NODE_VERSION=$(node --version)
    print_success "Node.js installed: $NODE_VERSION"
else
    print_error "Node.js not found!"
    echo "Please install Node.js from: https://nodejs.org/"
    exit 1
fi

# Check npm
if command_exists npm; then
    NPM_VERSION=$(npm --version)
    print_success "npm installed: $NPM_VERSION"
else
    print_error "npm not found!"
    exit 1
fi

# Check GDAL
if command_exists gdal-config; then
    GDAL_VERSION=$(gdal-config --version)
    print_success "GDAL installed: $GDAL_VERSION"
else
    print_error "GDAL not found!"
    echo ""
    echo "Please install GDAL:"
    echo "  Ubuntu/Debian: sudo apt-get install gdal-bin libgdal-dev"
    echo "  macOS: brew install gdal"
    echo "  Windows: Use WSL2 or download from https://gdal.org/"
    exit 1
fi

echo ""

# Step 2: Install system dependencies
echo "Step 2: Installing system dependencies..."
echo "-----------------------------------------"

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    print_info "Linux detected"
    
    # Check if running with sudo or ask for permission
    if [ "$EUID" -ne 0 ]; then 
        print_info "Some packages require sudo permission"
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            pkg-config \
            libssl-dev \
            libwebkit2gtk-4.0-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev
    else
        apt-get update
        apt-get install -y \
            build-essential \
            pkg-config \
            libssl-dev \
            libwebkit2gtk-4.0-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev
    fi
    print_success "Linux dependencies installed"
    
elif [[ "$OSTYPE" == "darwin"* ]]; then
    print_info "macOS detected"
    
    if command_exists brew; then
        brew install pkg-config
        print_success "macOS dependencies installed"
    else
        print_error "Homebrew not found! Please install from https://brew.sh/"
        exit 1
    fi
else
    print_info "Windows or unknown OS detected"
    echo "Please ensure you have the required dependencies installed"
fi

echo ""

# Step 3: Install Node dependencies
echo "Step 3: Installing Node.js dependencies..."
echo "------------------------------------------"

print_info "Running npm install..."
npm install

print_success "Node.js dependencies installed"
echo ""

# Step 4: Build Rust dependencies
echo "Step 4: Building Rust dependencies..."
echo "--------------------------------------"

cd src-tauri
print_info "Running cargo build (this may take a few minutes)..."
cargo build --release
cd ..

print_success "Rust dependencies built"
echo ""

# Step 5: Setup environment file
echo "Step 5: Setting up environment variables..."
echo "-------------------------------------------"

if [ ! -f .env.local ]; then
    print_info "Creating .env.local file..."
    cat > .env.local << EOF
# Mapbox Configuration
# Get your free token at: https://www.mapbox.com/
VITE_MAPBOX_TOKEN=YOUR_MAPBOX_TOKEN_HERE

# Optional: Default paths for testing
# VITE_DEFAULT_DTM_PATH=/path/to/your/test/dtm.tif
# VITE_DEFAULT_DSM_PATH=/path/to/your/test/dsm.tif
EOF
    print_success ".env.local created"
    print_info "Please add your Mapbox token to .env.local"
else
    print_info ".env.local already exists, skipping..."
fi

echo ""

# Step 6: Create test data directory
echo "Step 6: Creating test data directory..."
echo "---------------------------------------"

if [ ! -d "test_data" ]; then
    mkdir -p test_data
    print_success "test_data directory created"
else
    print_info "test_data directory already exists"
fi

echo ""

# Final message
echo "================================================"
echo "           Setup Complete! 🎉                  "
echo "================================================"
echo ""
print_success "All dependencies installed successfully!"
echo ""
echo "Next steps:"
echo "1. Add your Mapbox token to .env.local"
echo "   Get a free token at: https://www.mapbox.com/"
echo ""
echo "2. Place your test DTM and DSM files in the test_data/ directory"
echo ""
echo "3. Run the application:"
echo "   npm run tauri dev"
echo ""
echo "Happy shadow calculating! 🌞🏔️"