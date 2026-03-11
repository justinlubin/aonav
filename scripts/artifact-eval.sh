RAYON_NUM_THREADS="$(nproc)"

echo "Running on ${RAYON_NUM_THREADS} cores (implementations are single-threaded)"
echo "Running with extra flag (may be empty): '$1'"

for path in entries/*/; do
  collection=$(basename $path)

  echo "Benchmarking '${collection}'..."

  aonav benchmark $1 \
    --parallel \
    --replicates 3 \
    --minimal \
    --providers AlphabeticalUnsound,AlphabeticalComplete,AlphabeticalRelevant \
    "${path}" \
    > "results/${collection}.csv"
done

cd analysis
uv run main.py MINIMAL > out/summary.txt

echo "All done!"
