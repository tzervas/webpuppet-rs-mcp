#!/usr/bin/env bash
# Security Audit Script for MCP Server
# Performs comprehensive security checks and generates audit report

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
AUDIT_DIR="${PROJECT_ROOT}/security-audits"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
AUDIT_LOG="${AUDIT_DIR}/audit_${TIMESTAMP}.log"

mkdir -p "${AUDIT_DIR}"

echo "🔒 Security Audit for embeddenator-webpuppet-mcp v0.1.0-alpha.3"
echo "=============================================================="
echo ""

cd "${PROJECT_ROOT}"

# 1. Dependency vulnerability scan
echo "1. Scanning dependencies for known vulnerabilities..."
if command -v cargo-audit &> /dev/null; then
    cargo audit --json > "${AUDIT_DIR}/vulnerabilities_${TIMESTAMP}.json" 2>&1 || true
    cargo audit | tee -a "${AUDIT_LOG}"
    echo "✓ Vulnerability scan complete"
else
    echo "⚠️  cargo-audit not installed. Install with: cargo install cargo-audit"
fi
echo ""

# 2. Check for outdated dependencies
echo "2. Checking for outdated dependencies..."
if command -v cargo-outdated &> /dev/null; then
    cargo outdated | tee -a "${AUDIT_LOG}"
    echo "✓ Outdated dependency check complete"
else
    echo "⚠️  cargo-outdated not installed. Install with: cargo install cargo-outdated"
fi
echo ""

# 3. License compliance check
echo "3. Checking license compliance..."
if command -v cargo-license &> /dev/null; then
    cargo license | tee "${AUDIT_DIR}/licenses_${TIMESTAMP}.txt"
    echo "✓ License check complete"
else
    echo "⚠️  cargo-license not installed. Install with: cargo install cargo-license"
fi
echo ""

# 4. Code quality analysis
echo "4. Running clippy for code quality..."
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee -a "${AUDIT_LOG}" || true
echo "✓ Clippy analysis complete"
echo ""

# 5. Check for unsafe code
echo "5. Scanning for unsafe code blocks..."
UNSAFE_COUNT=$(grep -r "unsafe" src/ --include="*.rs" | wc -l || echo "0")
echo "Unsafe code blocks found: ${UNSAFE_COUNT}"
if [[ "${UNSAFE_COUNT}" -gt 0 ]]; then
    echo "⚠️  Review unsafe code usage:"
    grep -n "unsafe" src/ --include="*.rs" | tee -a "${AUDIT_LOG}"
fi
echo ""

# 6. Secret scanning
echo "6. Scanning for potential secrets..."
if command -v gitleaks &> /dev/null; then
    gitleaks detect --no-git -v 2>&1 | tee -a "${AUDIT_LOG}" || true
    echo "✓ Secret scan complete"
else
    echo "⚠️  gitleaks not installed. Install from: https://github.com/gitleaks/gitleaks"
fi
echo ""

# 7. SBOM generation
echo "7. Generating Software Bill of Materials (SBOM)..."
if command -v cargo-sbom &> /dev/null; then
    cargo sbom > "${AUDIT_DIR}/sbom_${TIMESTAMP}.json"
    echo "✓ SBOM generated"
else
    echo "⚠️  cargo-sbom not installed. Install with: cargo install cargo-sbom"
fi
echo ""

echo "=============================================================="
echo "✅ Security audit complete"
echo "Audit log: ${AUDIT_LOG}"
echo "Review all findings before merging to production"
echo "=============================================================="
