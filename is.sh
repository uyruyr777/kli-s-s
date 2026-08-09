#!/usr/bin/env bash
set -e

INSTALL_DIR="/usr/local/bin"
BIN_NAME="kli-s-s"

echo ">> Удаляю старый бинарник (если есть)..."
sudo rm -f "$INSTALL_DIR/$BIN_NAME"

echo ">> Чищу старую сборку..."
cargo clean

echo ">> Собираю release..."
cargo build --release

BIN=$(find target/release -maxdepth 1 -type f -executable ! -name "*.d" | head -n1)

if [ -z "$BIN" ]; then
    echo "Не нашёл собранный бинарник в target/release"
    exit 1
fi

echo ">> Устанавливаю новый бинарник..."
sudo cp "$BIN" "$INSTALL_DIR/$BIN_NAME"
sudo chmod +x "$INSTALL_DIR/$BIN_NAME"

hash -r

which "$BIN_NAME"

echo ""
echo "Usage: $BIN_NAME -x path/to/file.kss"

