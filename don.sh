#!/usr/bin/env bash
set -e

rm -rf "$HOME/.kli-s-s"
mkdir -p "$HOME/.kli-s-s"
curl -fsSL https://github.com/uyruyr777/kli-s-s/archive/refs/heads/main.tar.gz | tar -xz -C "$HOME/.kli-s-s" --strip-components=1

cd "$HOME/.kli-s-s/l"
chmod +x is.sh
./is.sh
