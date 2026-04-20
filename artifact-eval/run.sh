mkdir -p mnt/results
mkdir -p mnt/graphs

docker run -it \
    --mount type=bind,source="$(pwd)/mnt/results",target="/root/results" \
    --mount type=bind,source="$(pwd)/mnt/graphs",target="/root/analysis/out" \
    aonav $1
