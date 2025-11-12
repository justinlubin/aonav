#!/usr/bin/env bash

mkdir -p benchmark
rm -rf benchmark
mkdir -p benchmark

mkdir -p benchmark-reduced
rm -rf benchmark-reduced
mkdir -p benchmark-reduced

cd raw-benchmark
cargo b
for f in *.json; do
    ../target/debug/under convert -f AOJsonGraph --randomize "$f" > "../benchmark/$f"
    # ../target/debug/under convert -f AOJsonGraph --reduce --randomize "$f" > "../benchmark-reduced/$f"
done

cd ..

./target/debug/under generate-solutions -c 1 --parallel benchmark
