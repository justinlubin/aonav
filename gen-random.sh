
min_num_nodes=$1
max_num_nodes=$2
num_nodes_interval=$3

min_num_children=$4
max_num_chidren=$5
num_children_interval=$6

for (( num_nodes=min_num_nodes; num_nodes<=max_num_nodes; num_nodes+=num_nodes_interval )); do
    echo $num_nodes
  for (( children=min_num_children; children<=max_num_chidren; children+=num_children_interval )); do
    echo $children
    file=random-ao/random-$num_nodes-$children.json
    python3 gen-and-or.py $num_nodes $children > $file
    ./target/debug/under render $file
    mv out/RENDERED.pdf out/random-$num_nodes-$children.pdf
  done
done
