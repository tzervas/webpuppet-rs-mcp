# Local Multi-Architecture Build Guide

## Overview

This guide covers building ARM64 and AMD64 Linux binaries locally using Docker containers with optimized resource allocation.

## Prerequisites

- Docker installed and running
- 8+ CPU cores available
- 16GB+ RAM recommended
- GitHub CLI (`gh`) for release uploads (optional)

## Quick Start

```bash
# Build both architectures (uses version from Cargo.toml)
./scripts/build_release.sh

# Build specific version
./scripts/build_release.sh 0.1.0-alpha.3
```

## How It Works

The build script:
1. **Parallel Builds**: Runs AMD64 and ARM64 builds simultaneously in separate Docker containers
2. **Resource Allocation**: Allocates 4 CPU cores and 8GB RAM per container
3. **Static Linking**: Uses musl for portable Linux binaries
4. **Binary Stripping**: Reduces binary size
5. **Artifact Creation**: Creates tarballs and checksums
6. **Optional Upload**: Can upload directly to GitHub releases

## Build Process

### AMD64 Build
- Container: `rust-build-amd64`
- Target: `x86_64-unknown-linux-musl`
- Resources: 4 cores, 8GB RAM
- Output: `webpuppet-mcp-linux-amd64`

### ARM64 Build
- Container: `rust-build-arm64`
- Target: `aarch64-unknown-linux-musl`
- Cross-compiler: `aarch64-linux-gnu-gcc`
- Resources: 4 cores, 8GB RAM
- Output: `webpuppet-mcp-linux-arm64`

## Resource Requirements

### Minimum
- 8 CPU cores (4 per build)
- 16GB RAM (8GB per build)
- 10GB disk space

### Recommended
- 12+ CPU cores (for system overhead)
- 24GB+ RAM
- 20GB disk space

## Build Duration

Typical build times (with 4 cores per container):
- **Cold build**: ~15-20 minutes per architecture
- **Incremental build**: ~2-5 minutes
- **Parallel total time**: ~15-20 minutes (both architectures)

## Output Artifacts

All artifacts are created in `release-artifacts/`:

```
release-artifacts/
├── webpuppet-mcp-linux-amd64              # Raw AMD64 binary
├── webpuppet-mcp-linux-arm64              # Raw ARM64 binary
├── webpuppet-mcp-v0.1.0-alpha.3-linux-amd64.tar.gz
├── webpuppet-mcp-v0.1.0-alpha.3-linux-arm64.tar.gz
└── webpuppet-mcp-v0.1.0-alpha.3-checksums.txt
```

## Uploading to GitHub

### Automatic Upload
The script prompts to upload after successful builds:
```bash
./scripts/build_release.sh
# Answer 'y' when prompted
```

### Manual Upload
```bash
# Using GitHub CLI
gh release upload v0.1.0-alpha.3 \
  release-artifacts/*.tar.gz \
  release-artifacts/*-checksums.txt

# Or create release and upload
gh release create v0.1.0-alpha.3 \
  --title "Release v0.1.0-alpha.3" \
  --notes-file CHANGELOG.md \
  release-artifacts/*.tar.gz \
  release-artifacts/*-checksums.txt
```

## Verifying Binaries

### Check Binary Info
```bash
# AMD64 binary
file release-artifacts/webpuppet-mcp-linux-amd64
# Output: ELF 64-bit LSB executable, x86-64, statically linked

# ARM64 binary
file release-artifacts/webpuppet-mcp-linux-arm64
# Output: ELF 64-bit LSB executable, ARM aarch64, statically linked
```

### Verify Checksums
```bash
cd release-artifacts
sha256sum -c webpuppet-mcp-v0.1.0-alpha.3-checksums.txt
```

### Test Binary
```bash
# AMD64 (on x86_64 host)
./release-artifacts/webpuppet-mcp-linux-amd64 --version

# ARM64 (requires ARM64 host or emulation)
docker run --rm -v $PWD/release-artifacts:/app \
  --platform linux/arm64 \
  ubuntu:22.04 \
  /app/webpuppet-mcp-linux-arm64 --version
```

## Troubleshooting

### Docker Container Issues

**Problem**: Container fails to start
```bash
# Check Docker is running
docker ps

# Check available resources
docker system df
docker system prune  # if needed
```

**Problem**: Out of memory
```bash
# Increase Docker memory limit in Docker Desktop
# Or adjust script to use less memory per container
```

### Build Failures

**Problem**: Dependency download fails
```bash
# Pre-cache dependencies
cargo fetch

# Or use local cache in Docker
docker run -v ~/.cargo/registry:/usr/local/cargo/registry ...
```

**Problem**: Cross-compilation issues (ARM64)
```bash
# Verify cross-compiler installed in container
docker run --rm rust:1.75-bookworm \
  bash -c "apt-get update && apt-get install -y gcc-aarch64-linux-gnu"
```

### Upload Issues

**Problem**: GitHub CLI not authenticated
```bash
gh auth login
gh auth status
```

**Problem**: Release doesn't exist
```bash
# Create release first
gh release create v0.1.0-alpha.3 \
  --title "Release v0.1.0-alpha.3" \
  --notes-file CHANGELOG.md
```

## Advanced Usage

### Custom Resource Allocation

Edit `scripts/build_release.sh` to adjust:
```bash
# Change CPU allocation (default: 4)
--cpus="6"

# Change memory allocation (default: 8g)
--memory="12g"
```

### Build Only One Architecture

```bash
# AMD64 only
docker run --rm --cpus="4" --memory="8g" \
  -v $PWD:/workspace -w /workspace \
  rust:1.75-bookworm bash -c "
    apt-get update && apt-get install -y musl-tools
    rustup target add x86_64-unknown-linux-musl
    cargo build --release --target x86_64-unknown-linux-musl
  "

# ARM64 only
docker run --rm --cpus="4" --memory="8g" \
  -v $PWD:/workspace -w /workspace \
  rust:1.75-bookworm bash -c "
    apt-get update && apt-get install -y gcc-aarch64-linux-gnu
    rustup target add aarch64-unknown-linux-musl
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
    cargo build --release --target aarch64-unknown-linux-musl
  "
```

### Using Docker BuildKit

For faster builds with layer caching:
```bash
# Enable BuildKit
export DOCKER_BUILDKIT=1

# Create Dockerfile for persistent cache
cat > Dockerfile.release <<EOF
FROM rust:1.75-bookworm
RUN apt-get update && apt-get install -y musl-tools gcc-aarch64-linux-gnu
WORKDIR /workspace
EOF

docker build -t rust-cross -f Dockerfile.release .
```

## Performance Tips

1. **Use Local Cargo Cache**: Mount `~/.cargo` to speed up dependency downloads
2. **Incremental Builds**: Keep `target/` directory between builds
3. **Parallel Jobs**: Set `CARGO_BUILD_JOBS` environment variable
4. **SSD Storage**: Use SSD for Docker volumes and build directory
5. **More Cores**: Allocate 6-8 cores per container if available

## CI/CD Integration

This local build process complements GitHub Actions:
- **Local**: Fast iteration, immediate binaries
- **GitHub Actions**: Automated releases on tags

To fully automate:
```bash
# Tag and push
git tag v0.1.0-alpha.3
git push origin v0.1.0-alpha.3

# Build locally
./scripts/build_release.sh

# Upload artifacts
gh release upload v0.1.0-alpha.3 release-artifacts/*.tar.gz
```

## Security Considerations

- **Static Linking**: Binaries are self-contained with musl
- **Stripped Binaries**: Debug symbols removed
- **Checksums**: SHA256 hashes provided for verification
- **Reproducible**: Same inputs produce same outputs
- **Isolated**: Docker containers prevent system contamination

## Support

For issues with local builds:
- Check Docker logs: `docker logs rust-build-amd64`
- Verify system resources: `docker stats`
- Review build output in `release-artifacts/`
- Test in clean container to isolate issues

## References

- [Rust Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Docker Resource Constraints](https://docs.docker.com/config/containers/resource_constraints/)
- [musl libc](https://musl.libc.org/)
- [GitHub CLI Documentation](https://cli.github.com/manual/)
