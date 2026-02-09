
get_tree() {
    # name of outfile should be out-example-hash
    IFS="/" read -ra split_path <<< "$1"
    example_name="${split_path[1]}"
    outfile="under-examples-5/out-${example_name}-$2.json"
    # get raw tree
    cargo +nightly-2025-08-20 argus tree $1 $2 $3 $4 $5 $6 | jq '.Ok' > tree.json
    # call frontend to get label and topology info
    npx ts-node ide/packages/panoptes/src/TreeView/under-tree.ts tree.json $outfile
}

# iterate through files in under-hashes
for file in "under-hashes-all"/*; do
    echo $file
    # path to file is in first line
    IFS= read -r file_arg < "$file"
    # Loop through each line of the file    
    tail -n +2 "$file" | while IFS= read -r line; do        
        arguments=($line) 
        get_tree $file_arg ${arguments[0]} ${arguments[1]} ${arguments[2]} ${arguments[3]} ${arguments[4]}

    done
done
