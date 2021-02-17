#!/bin/sh

set -eu

if [ "$#" -ne 0 ]; then
  echo "usage: $0"
  exit 1
fi

cd "$(dirname "$0")"
cd ..

root="$PWD"
ok=true
for c in analysis fmt; do
  cd "crates/$c/src/tests"
  for f in data/*.c0; do
    if ! git grep -q "$f" -- mod.rs; then
      echo "$c: $f"
      ok=false
    fi
  done
  cd "$root"
done

if "$ok"; then
  echo "ok: all test data files are mentioned"
else
  echo "error: missing a test for the above files"
  exit 1
fi
