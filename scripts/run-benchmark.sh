#!/usr/bin/env bash

echo "Running on $RAYON_NUM_THREADS cores"

cargo build

run() {
  echo "Running provider '$2' on suite '$1'..."
  
  ./target/debug/under benchmark \
    --parallel \
    --replicates 3 \
    --minimal \
    --timeout 120 \
    --providers "$2" \
    "benchmark/suites/$1" \
    > "benchmark/results/$1-$2.csv"

  echo "Done!"
}

for path in benchmark/suites/*; do
  suite=$(basename $path)

  run "$suite" "AlphabeticalUnsound"
  run "$suite" "AlphabeticalComplete"
  run "$suite" "AlphabeticalRelevant"

  run "$suite" "MaxInfoGainRelevant"
  run "$suite" "MaxInfoGain"
done
