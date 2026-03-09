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
    case "$f" in
        argus*)
            ../target/debug/under convert -f AOJsonGraph --reduce --randomize "$f" > "../benchmark-reduced/$f"
            # Failed (goal node already true)
            if [ -s "../benchmark-reduced/$f" ]; then
                :
            else
                echo "failed: $f"
                rm "../benchmark-reduced/$f"
            fi
            ;;
        *)
            ../target/debug/under convert -f AOJsonGraph --randomize "$f" > "../benchmark/$f"
            # Failed (goal node already true)
            if [ -s "../benchmark/$f" ]; then
                :
            else
                echo "failed: $f"
                rm "../benchmark/$f"
            fi
            ;;
    esac
done

cd ..

./target/debug/under generate-solutions -c 3 --parallel benchmark
./target/debug/under generate-solutions -c 3 --parallel benchmark-reduced
