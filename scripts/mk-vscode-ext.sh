#!/bin/sh

set -eu

if [ "$#" -ne 0 ]; then
  echo "usage: $0"
  exit 1
fi

cd "$(dirname "$0")"
cd ..

vscode='extensions/vscode'
cargo build
mkdir -p "$vscode/out"
cp target/debug/c0ls "$vscode/out/c0ls"
cd "$vscode"
npm run build
