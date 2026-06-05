#!/usr/bin/env bash
set -euo pipefail

BINARY_NAME="hermes_newsletter_script"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "==> Building ${BINARY_NAME}..."

if command -v cargo &>/dev/null; then
    # Build locally if Rust is installed
    cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null \
        || cargo build --release
    if [ -f "${SCRIPT_DIR}/target/x86_64-unknown-linux-musl/release/${BINARY_NAME}" ]; then
        BIN_PATH="${SCRIPT_DIR}/target/x86_64-unknown-linux-musl/release/${BINARY_NAME}"
    else
        BIN_PATH="${SCRIPT_DIR}/target/release/${BINARY_NAME}"
    fi
else
    # Build in Docker if Rust is not installed
    echo "    Rust not found, building in Docker..."
    docker build -t "${BINARY_NAME}-builder" "${SCRIPT_DIR}"
    docker create --name "${BINARY_NAME}-tmp" "${BINARY_NAME}-builder" >/dev/null 2>&1
    docker cp "${BINARY_NAME}-tmp:/usr/local/bin/${BINARY_NAME}" "${SCRIPT_DIR}/${BINARY_NAME}" >/dev/null
    docker rm "${BINARY_NAME}-tmp" >/dev/null 2>&1
    BIN_PATH="${SCRIPT_DIR}/${BINARY_NAME}"
fi

echo "==> Installing to ${INSTALL_DIR}/${BINARY_NAME}"
cp "${BIN_PATH}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

# Ensure tags file exists
TAGS_FILE="${HOME}/.hermes/newsletter-tags.json"
if [ ! -f "${TAGS_FILE}" ]; then
    mkdir -p "$(dirname "${TAGS_FILE}")"
    echo '{ "tags": ["rust", "kubernetes", "ai"] }' > "${TAGS_FILE}"
    echo "==> Created default tags file at ${TAGS_FILE}"
fi

echo "==> Done. Run with: ${BINARY_NAME}"
