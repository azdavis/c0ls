#!/bin/sh

set -eu

if [ "$#" -ne 0 ]; then
  echo "usage: $0"
  exit 1
fi

cd "$(dirname "$0")"
cd ../crates/tests/src

ok=true
for f in data/*.c0; do
  if ! git grep -q "$f" -- lib.rs; then
    echo "$f"
    ok=false
  fi
done
if "$ok"; then
  echo "ok: all test data files are mentioned"
else
  echo "error: missing a test for the above files"
  exit 1
fi
