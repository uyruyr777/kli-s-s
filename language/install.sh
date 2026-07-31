#!/usr/bin/env bash
set -e

# Устанавливает интерпретатор KSS как команду `kli-s-s` в /usr/local/bin —
# доступна всем пользователям системы. Требует sudo (запись в /usr/local/bin).
# Запускать из корня проекта (там, где лежит Cargo.toml).
cargo build --release

BIN=$(find target/release -maxdepth 1 -type f -executable ! -name "*.d" | head -n1)

if [ -z "$BIN" ]; then
    exit 1
fi


INSTALL_DIR="/usr/local/bin"

sudo cp "$BIN" "$INSTALL_DIR/kli-s-s"
sudo chmod +x "$INSTALL_DIR/kli-s-s"

which kli-s-s

echo ""
echo "Usage: kli-s-s -x path/to/file.kss"

