#!/bin/bash
#
# Quantumn Code - Installer Script
# Supports Linux and macOS
#

set -e

VERSION="${QUANTUMN_VERSION:-0.1.0}"
REPO="Akatsuki2r/QuantumCode"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Platform detection
detect_platform() {
    OS=$(uname -s)
    ARCH=$(uname -m)

    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64) echo "linux-x64" ;;
                aarch64|arm64) echo "linux-arm64" ;;
                *)
                    echo "Unsupported architecture: $ARCH" >&2
                    exit 1
                    ;;
            esac
            ;;
        Darwin)
            case "$ARCH" in
                x86_64) echo "darwin-x64" ;;
                arm64) echo "darwin-arm64" ;;
                *)
                    echo "Unsupported architecture: $ARCH" >&2
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $OS" >&2
            exit 1
            ;;
    esac
}

# Asset names
get_asset_name() {
    local platform=$1
    case "$platform" in
        linux-x64)   echo "quantumn-x86_64-unknown-linux-gnu.tar.gz" ;;
        linux-arm64) echo "quantumn-aarch64-unknown-linux-gnu.tar.gz" ;;
        darwin-x64)  echo "quantumn-x86_64-apple-darwin.tar.gz" ;;
        darwin-arm64) echo "quantumn-aarch64-apple-darwin.tar.gz" ;;
        *)
            echo "Unsupported platform: $platform" >&2
            exit 1
            ;;
    esac
}

# GitHub API to get release asset URL
get_download_url() {
    local platform=$1
    local asset_name=$(get_asset_name "$platform")

    # Get the latest release URL
    local url="https://github.com/${REPO}/releases/latest/download/${asset_name}"

    # Check if asset exists, if not try v$VERSION
    if ! curl -sI "$url" | head -1 | grep -q "200\|302"; then
        url="https://github.com/${REPO}/releases/download/v${VERSION}/${asset_name}"
    fi

    echo "$url"
}

# Download and install
install() {
    local platform=$(detect_platform)
    local download_url=$(get_download_url "$platform")
    local temp_dir=$(mktemp -d)
    local archive="${temp_dir}/quantumn.tar.gz"

    echo "Installing Quantumn Code v${VERSION}..."
    echo "Platform: ${platform}"
    echo "Downloading from: ${download_url}"

    # Download
    curl -sL "$download_url" -o "$archive"

    # Check if download succeeded
    if [ ! -s "$archive" ]; then
        echo "Download failed. Please check if version ${VERSION} exists." >&2
        rm -rf "$temp_dir"
        exit 1
    fi

    # Extract
    echo "Extracting..."
    tar -xzf "$archive" -C "$temp_dir"

    # Find the binary
    local binary="${temp_dir}/quantumn"
    if [ ! -f "$binary" ]; then
        # Try alternative extraction path
        binary=$(find "$temp_dir" -name "quantumn" -type f 2>/dev/null | head -1)
    fi

    if [ ! -f "$binary" ]; then
        echo "Failed to extract binary from archive." >&2
        rm -rf "$temp_dir"
        exit 1
    fi

    # Install
    echo "Installing to ${INSTALL_DIR}..."
    if [ ! -w "$INSTALL_DIR" ]; then
        echo "Need sudo to install to ${INSTALL_DIR}..."
        sudo cp "$binary" "${INSTALL_DIR}/quantumn"
        sudo chmod +x "${INSTALL_DIR}/quantumn"
    else
        cp "$binary" "${INSTALL_DIR}/quantumn"
        chmod +x "${INSTALL_DIR}/quantumn"
    fi

    # Cleanup
    rm -rf "$temp_dir"

    echo ""
    echo "Installation complete!"
    echo "Run 'quantumn --version' to verify."
}

# Check dependencies
check_deps() {
    local missing=""

    if ! command -v curl &> /dev/null; then
        missing="$missing curl"
    fi

    if [ -n "$missing" ]; then
        echo "Missing dependencies:$missing" >&2
        exit 1
    fi
}

# Main
main() {
    check_deps
    install
}

main "$@"
