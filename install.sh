#!/usr/bin/env bash
set -euo pipefail

REPO="waheed786dar-cell/orbvynx"
BINARY_NAME="orbvynx"

echo "Installing ORBVYNX..."

if [ -n "${PREFIX:-}" ] && [ -d "${PREFIX}/bin" ]; then
  INSTALL_DIR="${PREFIX}/bin"
  PLATFORM="termux-arm64"
elif [ "$(uname -s)" = "Linux" ]; then
  INSTALL_DIR="${HOME}/.local/bin"
  PLATFORM="linux-x86_64"
elif [ "$(uname -s)" = "Darwin" ]; then
  INSTALL_DIR="${HOME}/.local/bin"
  PLATFORM="macos"
else
  echo "Unsupported platform: $(uname -s)"
  exit 1
fi

mkdir -p "$INSTALL_DIR"

LATEST_URL=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep "browser_download_url" \
  | grep "$PLATFORM" \
  | cut -d '"' -f 4 \
  | head -n1)

if [ -z "$LATEST_URL" ]; then
  echo "No prebuilt binary found for platform: $PLATFORM"
  echo "Falling back to source build (requires cargo)..."

  if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo not found. Install Rust first (e.g. 'pkg install rust' on Termux) and re-run this script."
    exit 1
  fi

  TMP_DIR=$(mktemp -d)
  git clone --depth 1 "https://github.com/${REPO}.git" "$TMP_DIR"
  (cd "$TMP_DIR" && cargo build --release -p orbvynx-cli)
  cp "$TMP_DIR/target/release/${BINARY_NAME}" "$INSTALL_DIR/${BINARY_NAME}"
  rm -rf "$TMP_DIR"
else
  echo "Downloading ${BINARY_NAME} from ${LATEST_URL}..."
  curl -sSL "$LATEST_URL" -o "$INSTALL_DIR/${BINARY_NAME}"
fi

chmod +x "$INSTALL_DIR/${BINARY_NAME}"

echo ""
echo "ORBVYNX installed to: $INSTALL_DIR/${BINARY_NAME}"

if ! command -v "$BINARY_NAME" >/dev/null 2>&1; then
  echo ""
  echo "NOTE: $INSTALL_DIR is not on your PATH."
  echo "Add this to your shell profile (~/.bashrc or ~/.zshrc):"
  echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
else
  echo "Run 'orbvynx --help' to get started (or 'orbvynx git status' as a quick test)."
fi
