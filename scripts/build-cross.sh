#!/bin/bash
# Cross-compilation build script for shortener project

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="shortener"
VERSION=$(cat version.txt 2>/dev/null || echo "0.1.0")
BUILD_DIR="target/release-builds"

# Supported targets
TARGETS=(
    "x86_64-unknown-linux-gnu"      # Linux x86_64 (glibc)
    "x86_64-unknown-linux-musl"     # Linux x86_64 (musl, static)
    "aarch64-unknown-linux-gnu"     # Linux ARM64 (glibc)
    "aarch64-unknown-linux-musl"    # Linux ARM64 (musl, static)
    "armv7-unknown-linux-gnueabihf" # Linux ARMv7
    "x86_64-pc-windows-gnu"         # Windows x86_64
)

# Optional targets (require special setup)
OPTIONAL_TARGETS=(
    "x86_64-apple-darwin"           # macOS x86_64
    "aarch64-apple-darwin"          # macOS ARM64 (Apple Silicon)
)

# Function to print colored output
print_info() {
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

# Function to check if cross is installed
check_cross() {
    if ! command -v cross &> /dev/null; then
        print_error "cross is not installed"
        echo ""
        echo "Install cross with:"
        echo "  cargo install cross --git https://github.com/cross-rs/cross"
        echo ""
        exit 1
    fi
    print_success "cross is installed"
}

# Function to build for a specific target
build_target() {
    local target=$1
    local package=$2
    
    print_info "Building ${package} for ${target}..."
    
    if cross build --release --target ${target} -p ${package}; then
        print_success "Built ${package} for ${target}"
        return 0
    else
        print_error "Failed to build ${package} for ${target}"
        return 1
    fi
}

# Function to package binaries
package_binary() {
    local target=$1
    local package=$2
    local binary_name=$3
    
    local target_dir="target/${target}/release"
    local output_dir="${BUILD_DIR}/${target}"
    
    # Create output directory
    mkdir -p ${output_dir}
    
    # Determine binary extension
    local ext=""
    if [[ ${target} == *"windows"* ]]; then
        ext=".exe"
    fi
    
    local binary="${target_dir}/${binary_name}${ext}"
    
    if [ -f "${binary}" ]; then
        # Copy binary
        cp ${binary} ${output_dir}/
        
        # Create archive
        local archive_name="${package}-${VERSION}-${target}"
        
        if [[ ${target} == *"windows"* ]]; then
            # Create zip for Windows
            (cd ${output_dir} && zip -q ${archive_name}.zip ${binary_name}${ext})
            print_success "Created ${archive_name}.zip"
        else
            # Create tar.gz for Unix-like systems
            (cd ${output_dir} && tar czf ${archive_name}.tar.gz ${binary_name}${ext})
            print_success "Created ${archive_name}.tar.gz"
        fi
        
        # Calculate checksum
        if command -v sha256sum &> /dev/null; then
            (cd ${output_dir} && sha256sum ${archive_name}.* > ${archive_name}.sha256)
        fi
    else
        print_warning "Binary not found: ${binary}"
    fi
}

# Function to build all targets
build_all() {
    local package=$1
    local binary_name=$2
    
    print_info "Building ${package} for all targets..."
    echo ""
    
    local success_count=0
    local fail_count=0
    
    for target in "${TARGETS[@]}"; do
        if build_target ${target} ${package}; then
            package_binary ${target} ${package} ${binary_name}
            ((success_count++))
        else
            ((fail_count++))
        fi
        echo ""
    done
    
    print_info "Build summary for ${package}:"
    print_success "Successful: ${success_count}"
    if [ ${fail_count} -gt 0 ]; then
        print_error "Failed: ${fail_count}"
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -t, --target TARGET     Build for specific target"
    echo "  -p, --package PACKAGE   Build specific package (server or cli)"
    echo "  -a, --all               Build all packages for all targets"
    echo "  -l, --list              List available targets"
    echo "  --server                Build server only"
    echo "  --cli                   Build CLI only"
    echo ""
    echo "Examples:"
    echo "  $0 --all                                    # Build everything"
    echo "  $0 --server                                 # Build server for all targets"
    echo "  $0 --cli                                    # Build CLI for all targets"
    echo "  $0 -t x86_64-unknown-linux-musl --server   # Build server for specific target"
    echo "  $0 -l                                       # List targets"
}

# Function to list targets
list_targets() {
    echo "Available targets:"
    echo ""
    echo "Standard targets:"
    for target in "${TARGETS[@]}"; do
        echo "  - ${target}"
    done
    echo ""
    echo "Optional targets (require special setup):"
    for target in "${OPTIONAL_TARGETS[@]}"; do
        echo "  - ${target}"
    done
}

# Main script
main() {
    local target=""
    local package=""
    local build_all_flag=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -l|--list)
                list_targets
                exit 0
                ;;
            -t|--target)
                target="$2"
                shift 2
                ;;
            -p|--package)
                package="$2"
                shift 2
                ;;
            -a|--all)
                build_all_flag=true
                shift
                ;;
            --server)
                package="server"
                shift
                ;;
            --cli)
                package="cli"
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Check if cross is installed
    check_cross
    
    # Create build directory
    mkdir -p ${BUILD_DIR}
    
    echo ""
    print_info "Starting cross-compilation build"
    print_info "Version: ${VERSION}"
    echo ""
    
    # Build based on options
    if [ "${build_all_flag}" = true ]; then
        build_all "shortener-server" "shortener-server"
        echo ""
        build_all "shortener-cli" "shortener-cli"
    elif [ -n "${package}" ]; then
        if [ "${package}" = "server" ]; then
            if [ -n "${target}" ]; then
                build_target ${target} "shortener-server"
                package_binary ${target} "shortener-server" "shortener-server"
            else
                build_all "shortener-server" "shortener-server"
            fi
        elif [ "${package}" = "cli" ]; then
            if [ -n "${target}" ]; then
                build_target ${target} "shortener-cli"
                package_binary ${target} "shortener-cli" "shortener-cli"
            else
                build_all "shortener-cli" "shortener-cli"
            fi
        else
            print_error "Unknown package: ${package}"
            exit 1
        fi
    else
        print_error "No build target specified"
        show_usage
        exit 1
    fi
    
    echo ""
    print_success "Build complete!"
    print_info "Binaries are in: ${BUILD_DIR}"
}

# Run main function
main "$@"
