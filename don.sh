#!/usr/bin/env bash
set -e

REPO_URL="https://github.com/uyruyr777/kli-s-s.git"
TARGET_DIR="$HOME/.kli-s-s"

if [ -d "$TARGET_DIR/.git" ]; then
    git -C "$TARGET_DIR" pull
else
    rm -rf "$TARGET_DIR"
    git clone "$REPO_URL" "$TARGET_DIR"
fi

cd "$TARGET_DIR"
chmod +x is.sh
./is.sh