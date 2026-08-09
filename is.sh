#!/usr/bin/env bash
set -e

INSTALL_DIR="/usr/local/bin"
BIN_NAME="kli-s-s"

sudo rm -f "$INSTALL_DIR/$BIN_NAME"

cargo clean

cargo build --release

BIN=$(find target/release -maxdepth 1 -type f -executable ! -name "*.d" | head -n1)

if [ -z "$BIN" ]; then
    exit 1
fi

sudo cp "$BIN" "$INSTALL_DIR/$BIN_NAME"
sudo chmod +x "$INSTALL_DIR/$BIN_NAME"

hash -r

which "$BIN_NAME"

echo ""
echo "Usage: $BIN_NAME -x path/to/file.kss"

