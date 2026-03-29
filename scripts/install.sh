#!/usr/bin/env bash
set -euo pipefail

# karma-cli installer
# Usage: curl -fsSL https://raw.githubusercontent.com/user/karma-cli/main/scripts/install.sh | bash

REPO="user/karma-cli"
BINARY="karma-cli"
INSTALL_DIR="${HOME}/.local/bin"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux)  TARGET_OS="unknown-linux-gnu" ;;
    Darwin) TARGET_OS="apple-darwin" ;;
    *)      echo "Unsupported OS: ${OS}"; exit 1 ;;
esac

case "${ARCH}" in
    x86_64|amd64) TARGET_ARCH="x86_64" ;;
    aarch64|arm64) TARGET_ARCH="aarch64" ;;
    *)            echo "Unsupported architecture: ${ARCH}"; exit 1 ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"

echo "Installing karma-cli for ${TARGET}..."

# Get latest release URL
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep "browser_download_url.*${TARGET}" | head -1 | cut -d '"' -f 4)

if [ -z "${LATEST}" ]; then
    echo "Error: Could not find release for ${TARGET}"
    echo "Try installing with: cargo install karma-cli"
    exit 1
fi

# Download and extract
TMPDIR=$(mktemp -d)
trap 'rm -rf "${TMPDIR}"' EXIT

echo "Downloading ${LATEST}..."
curl -fsSL "${LATEST}" -o "${TMPDIR}/karma-cli.tar.gz"

echo "Extracting..."
tar xzf "${TMPDIR}/karma-cli.tar.gz" -C "${TMPDIR}"

# Install
mkdir -p "${INSTALL_DIR}"
mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
chmod +x "${INSTALL_DIR}/${BINARY}"

echo ""
echo "Installed karma-cli to ${INSTALL_DIR}/${BINARY}"

# Check PATH
if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
    echo ""
    echo "Add ${INSTALL_DIR} to your PATH:"
    echo "  export PATH=\"${INSTALL_DIR}:\${PATH}\""
fi

echo ""
echo "Run 'karma-cli' to launch the interactive setup."
