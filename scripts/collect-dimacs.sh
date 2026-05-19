#!/usr/bin/env bash

set -e

cargo build --release

rm -rf aonav-dimacs
mkdir -p aonav-dimacs

run() {
  ./target/release/aonav benchmark \
    --parallel \
    --replicates 1 \
    --minimal \
    --timeout 30 \
    --providers "$2" \
    --incremental \
    --dimacs-log "aonav-dimacs/base" \
    "benchmark/entries/$1" \
    > /dev/null
}

for path in benchmark/entries/*/; do
  collection=$(basename $path)
  run "$collection" "AlphabeticalComplete"
done

cd aonav-dimacs

for f in *.dimacs; do
 shasum -a 256 $f
done > sha256.txt
