#!/bin/bash
# this is actually get trees
mkdir search_trees
lake test

# For each file in AesopTest: add tree line, run test > outfile, remove from tree line
for file in AesopTest/*; do
    test_name=${file:9}

    # Add search tree line right below 'import Aesop'
    filename=$file
    line_num=1

    echo 'import Aesop' > temp_file.txt
    echo 'set_option trace.aesop.tree true' >> temp_file.txt
    echo $file
    while IFS= read -r line; do
        if [[ "$line" == "import Aesop" ]]; then
            if [[ $line_num != 1 ]]; then
                head -n $(($line_num - 1)) $file >> temp_file.txt
            fi
            tail -n +$(($line_num + 1)) $file >> temp_file.txt
            break
        fi
        
        ((line_num++))

    done < "$filename"

    mv temp_file.txt $file

    # Run test and save output
    lake test  > search_trees${test_name%?????}.txt
    python3 make_tree.py search_trees${test_name%?????}.txt

    # delete tree line (line 2)
    head -n $((1)) $file > temp_file.txt
    tail -n +$((3)) $file >> temp_file.txt
    mv temp_file.txt $file

done
