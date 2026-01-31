#!/usr/bin/env sh
# gitnu install script
# Installs pre-built binaries from GitHub Releases when available.
# Falls back to `cargo install --git` if a binary isn't available.
#
# Usage: curl -fsSL https://raw.githubusercontent.com/OWNER/REPO/main/install.sh | sh
# Or:    curl -fsSL https://gitnu.com/install.sh | sh  (if hosted on your static site)

set -e

GITNU_REPO="${GITNU_REPO:-gitnu/gitnu}"
GITNU_GH="https://github.com/${GITNU_REPO}"
GITNU_API="https://api.github.com/repos/${GITNU_REPO}"
CARGO_BIN="${HOME}/.cargo/bin"
LOCAL_BIN="${HOME}/.local/bin"

echo "Installing gitnu..."
echo ""

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Darwin) OS_NAME="darwin" ;;
    Linux) OS_NAME="linux" ;;
    *) OS_NAME="unknown" ;;
esac

case "${ARCH}" in
    x86_64|amd64) ARCH_NAME="x86_64" ;;
    arm64|aarch64) ARCH_NAME="aarch64" ;;
    *) ARCH_NAME="unknown" ;;
esac

TARGET=""
if [ "${OS_NAME}" = "darwin" ] && [ "${ARCH_NAME}" = "x86_64" ]; then
    TARGET="x86_64-apple-darwin"
elif [ "${OS_NAME}" = "darwin" ] && [ "${ARCH_NAME}" = "aarch64" ]; then
    TARGET="aarch64-apple-darwin"
elif [ "${OS_NAME}" = "linux" ] && [ "${ARCH_NAME}" = "x86_64" ]; then
    TARGET="x86_64-unknown-linux-gnu"
elif [ "${OS_NAME}" = "linux" ] && [ "${ARCH_NAME}" = "aarch64" ]; then
    TARGET="aarch64-unknown-linux-gnu"
fi

install_binary() {
    if ! command -v curl >/dev/null 2>&1; then
        echo "curl is required to download binaries."
        return 1
    fi
    if ! command -v tar >/dev/null 2>&1; then
        echo "tar is required to extract binaries."
        return 1
    fi

    TAG="$(curl -fsSL "${GITNU_API}/releases/latest" | sed -n 's/.*"tag_name": "\(.*\)".*/\1/p' | head -n 1)"
    if [ -z "${TAG}" ]; then
        echo "No release tag found."
        return 1
    fi

    ASSET="gnu-${TAG}-${TARGET}.tar.gz"
    URL="${GITNU_GH}/releases/download/${TAG}/${ASSET}"
    TMP_DIR="$(mktemp -d)"

    if ! curl -fsSL "${URL}" -o "${TMP_DIR}/${ASSET}"; then
        echo "No pre-built binary found for ${TARGET}."
        return 1
    fi

    tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"

    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        mkdir -p "${LOCAL_BIN}"
        INSTALL_DIR="${LOCAL_BIN}"
    fi

    mv "${TMP_DIR}/gnu" "${INSTALL_DIR}/gnu"
    chmod +x "${INSTALL_DIR}/gnu"

    echo "Installed gnu to ${INSTALL_DIR}/gnu"
    return 0
}

if [ -n "${TARGET}" ]; then
    if install_binary; then
        echo ""
        echo "✓ gitnu installed. Run 'gnu --version' to verify."
        exit 0
    fi
fi

echo "Falling back to source install with cargo..."

if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust (cargo) is required. Install it first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "Then run this script again."
    exit 1
fi

export PATH="${CARGO_BIN}:${PATH}"

cargo install --git "${GITNU_GH}" || {
    echo "Install failed. Ensure the repo exists and is accessible: ${GITNU_GH}"
    exit 1
}

# Verify
if command -v gnu >/dev/null 2>&1; then
    echo ""
    echo "✓ gitnu installed. Run 'gnu --version' to verify."
else
    echo ""
    echo "Installed but 'gnu' not in PATH. Add to your shell config:"
    echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    echo "  export PATH=\"\${HOME}/.cargo/bin:\${PATH}\""
fi
