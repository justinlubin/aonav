RAYON_NUM_THREADS="$(nproc)"

echo "Running on ${RAYON_NUM_THREADS} cores (implementations are single-threaded)"

for path in entries/*; do
  collection=$(basename $path)

  echo "Benchmarking '${collection}'..."

  aonav benchmark \
    --parallel \
    --replicates 3 \
    --minimal \
    --providers AlphabeticalUnsound,AlphabeticalComplete,AlphabeticalRelevant \
    "${path}" \
    > "results/${collection}.csv"
done
