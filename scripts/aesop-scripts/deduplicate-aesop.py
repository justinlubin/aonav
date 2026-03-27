import glob
import sys
import os

parent_dir = sys.argv[1]

contents = set()

for path in glob.glob(f"{parent_dir}/aograph-json/*.json"):
    with open(path, "r") as f:
        content = f.read()
        if content in contents:
            continue
        contents.add(content)
        with open(f"{parent_dir}/output/aesop-{os.path.basename(path)}", "w") as out:
            out.write(content)
