#!/usr/bin/env bash
# Multi-Architecture Release Build Script
# Builds ARM64 and AMD64 binaries in parallel using Docker containers

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION="${1:-$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')}"
RELEASE_DIR="${PROJECT_ROOT}/release-artifacts"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')] ✓${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')] ⚠${NC} $*"
}

log "========================================="
log "Multi-Architecture Release Build"
log "Version: v${VERSION}"
log "========================================="

# Create release directory
mkdir -p "${RELEASE_DIR}"

# Cleanup function
cleanup() {
    log "Cleaning up Docker containers..."
    docker rm -f rust-build-amd64 2>/dev/null || true
    docker rm -f rust-build-arm64 2>/dev/null || true
}

trap cleanup EXIT

cd "${PROJECT_ROOT}"

# Build AMD64 in background
log "Starting AMD64 build (4 cores allocated)..."
docker run --rm \
    --name rust-build-amd64 \
    --cpus="4" \
    --memory="8g" \
    -v "${PROJECT_ROOT}:/workspace" \
    -w /workspace \
    rust:1.75-bookworm \
    bash -c "
        apt-get update -qq && apt-get install -y -qq musl-tools > /dev/null 2>&1
        rustup target add x86_64-unknown-linux-musl
        cargo build --release --target x86_64-unknown-linux-musl
        strip target/x86_64-unknown-linux-musl/release/webpuppet-mcp
        cp target/x86_64-unknown-linux-musl/release/webpuppet-mcp /workspace/release-artifacts/webpuppet-mcp-linux-amd64
    " &
AMD64_PID=$!

# Build ARM64 in background
log "Starting ARM64 build (4 cores allocated)..."
docker run --rm \
    --name rust-build-arm64 \
    --cpus="4" \
    --memory="8g" \
    -v "${PROJECT_ROOT}:/workspace" \
    -w /workspace \
    rust:1.75-bookworm \
    bash -c "
        apt-get update -qq && apt-get install -y -qq gcc-aarch64-linux-gnu musl-tools > /dev/null 2>&1
        rustup target add aarch64-unknown-linux-musl
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
        export CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc
        cargo build --release --target aarch64-unknown-linux-musl
        aarch64-linux-gnu-strip target/aarch64-unknown-linux-musl/release/webpuppet-mcp
        cp target/aarch64-unknown-linux-musl/release/webpuppet-mcp /workspace/release-artifacts/webpuppet-mcp-linux-arm64
    " &
ARM64_PID=$!

log "Building both architectures in parallel..."
log "AMD64 build PID: ${AMD64_PID}"
log "ARM64 build PID: ${ARM64_PID}"

# Wait for both builds
AMD64_SUCCESS=0
ARM64_SUCCESS=0

if wait ${AMD64_PID}; then
    log_success "AMD64 build completed"
    AMD64_SUCCESS=1
else
    log_warning "AMD64 build failed"
fi

if wait ${ARM64_PID}; then
    log_success "ARM64 build completed"
    ARM64_SUCCESS=1
else
    log_warning "ARM64 build failed"
fi

# Verify binaries exist
if [[ ${AMD64_SUCCESS} -eq 1 ]] && [[ -f "${RELEASE_DIR}/webpuppet-mcp-linux-amd64" ]]; then
    AMD64_SIZE=$(du -h "${RELEASE_DIR}/webpuppet-mcp-linux-amd64" | cut -f1)
    log_success "AMD64 binary: ${AMD64_SIZE}"
    
    # Create tarball
    cd "${RELEASE_DIR}"
    tar czf "webpuppet-mcp-v${VERSION}-linux-amd64.tar.gz" webpuppet-mcp-linux-amd64
    log_success "Created webpuppet-mcp-v${VERSION}-linux-amd64.tar.gz"
fi

if [[ ${ARM64_SUCCESS} -eq 1 ]] && [[ -f "${RELEASE_DIR}/webpuppet-mcp-linux-arm64" ]]; then
    ARM64_SIZE=$(du -h "${RELEASE_DIR}/webpuppet-mcp-linux-arm64" | cut -f1)
    log_success "ARM64 binary: ${ARM64_SIZE}"
    
    # Create tarball
    cd "${RELEASE_DIR}"
    tar czf "webpuppet-mcp-v${VERSION}-linux-arm64.tar.gz" webpuppet-mcp-linux-arm64
    log_success "Created webpuppet-mcp-v${VERSION}-linux-arm64.tar.gz"
fi

cd "${PROJECT_ROOT}"

# Generate checksums
if [[ ${AMD64_SUCCESS} -eq 1 ]] || [[ ${ARM64_SUCCESS} -eq 1 ]]; then
    log "Generating checksums..."
    cd "${RELEASE_DIR}"
    sha256sum *.tar.gz > "webpuppet-mcp-v${VERSION}-checksums.txt" 2>/dev/null || true
    log_success "Checksums generated"
fi

# Summary
log "========================================="
log "Build Summary"
log "========================================="
log "Version: v${VERSION}"
log "AMD64: $(if [[ ${AMD64_SUCCESS} -eq 1 ]]; then echo "✓ Success"; else echo "✗ Failed"; fi)"
log "ARM64: $(if [[ ${ARM64_SUCCESS} -eq 1 ]]; then echo "✓ Success"; else echo "✗ Failed"; fi)"
log "Artifacts: ${RELEASE_DIR}"
log ""
ls -lh "${RELEASE_DIR}"
log "========================================="

if [[ ${AMD64_SUCCESS} -eq 1 ]] && [[ ${ARM64_SUCCESS} -eq 1 ]]; then
    log_success "All builds completed successfully!"
    
    # Ask if user wants to upload to GitHub
    echo ""
    read -p "Upload artifacts to GitHub release v${VERSION}? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log "Uploading to GitHub..."
        
        if ! command -v gh &> /dev/null; then
            log_warning "GitHub CLI (gh) not installed. Install from: https://cli.github.com/"
            log "Manual upload command:"
            echo "  gh release upload v${VERSION} ${RELEASE_DIR}/*.tar.gz ${RELEASE_DIR}/*-checksums.txt"
            exit 0
        fi
        
        # Check if release exists
        if gh release view "v${VERSION}" &> /dev/null; then
            gh release upload "v${VERSION}" \
                "${RELEASE_DIR}/webpuppet-mcp-v${VERSION}-linux-amd64.tar.gz" \
                "${RELEASE_DIR}/webpuppet-mcp-v${VERSION}-linux-arm64.tar.gz" \
                "${RELEASE_DIR}/webpuppet-mcp-v${VERSION}-checksums.txt" \
                --clobber
            log_success "Artifacts uploaded to GitHub release v${VERSION}"
        else
            log_warning "Release v${VERSION} not found. Create it first with:"
            echo "  gh release create v${VERSION} --title 'Release v${VERSION}' --notes-file CHANGELOG.md"
        fi
    else
        log "Skipping upload. Artifacts available at: ${RELEASE_DIR}"
    fi
    
    exit 0
else
    log_warning "Some builds failed. Check logs above."
    exit 1
fi
