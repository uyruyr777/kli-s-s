#!/usr/bin/env bash
set -e

TMP=$(mktemp -d)
curl -fsSL https://github.com/uyruyr777/kli-s-s/archive/refs/heads/main.tar.gz | tar -xz -C "$TMP"

rm -rf "$HOME/.kli-s-s"
mkdir -p "$HOME/.kli-s-s"

shopt -s dotglob
mv "$TMP"/*/l/* "$HOME/.kli-s-s"/

rm -rf "$TMP"

cd "$HOME/.kli-s-s"
chmod +x is.sh
./is.sh