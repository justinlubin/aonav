# Getting Started Guide

_Estimated time to complete this section: 20 minutes._

## Step 1: Download Docker image and runner script

Download the following files from the Zenodo archive to a new directory on your
computer:
- `aonav-image.tar.gz`: Docker image of software used for the paper
- `run.sh`: Script to run the Docker image

## Step 2: Install and set up Podman

Install the lightweight Docker alternative [Podman](https://podman.io/).
If this is your first time ever using Podman, you will need to set up a virtual
machine by running the following commands:
```bash
podman machine init
podman machine start
```

## Step 3: Load Docker image

Load the Docker image by running the following command from the directory
you created in Step 1:
```bash
podman load -i aonav-image.tar.gz
```

## Step 4: Run the evaluation

Run the following command from the directory you created in Step 1:
```
sh run.sh
```

**If everything is installed correctly, you should see loading bars appear in
your terminal.** These loading bars indicate the evaluation harness is running.

There are a total of two loading bars to progress through for the evaluation.
On recent MacBook Pro hardware, the evaluation harness completes in ~2 minutes.

**If you are short on time,** you can press Ctrl-C to stop the evaluation
harness. The fact that the loading bars appeared means you have successfully
"kicked the tires" of the artifact. Whenever you have time, you can re-run the
`run.sh` command to run the full evaluation harness.

# Step-by-Step Instructions

_Estimated hands-on time to complete this section: 30 minutes._

_Estimated hands-off time to complete this section (if you have not fully
completed Step 4 above): 2 minutes to 1 hour._

Please ensure the script from Step 4 above has run to completion. We estimate
this script should take no longer than 1 hour to run.

The final line of output should say "All done!" when the evaluation is complete.

## Step 1: Verify graphs match paper

TODO

## Step 2: Verify empirical claims

We make four main empirical claims in the paper:

- **Section 8.1 (RQ1):** Strong Soundness reduces decision count by 1.7-2x
- **Section 8.2 (RQ2):** Relevancy Pruning further reduces decision count by 1.2-3x
- **Section 8.3 (RQ3):** Strong Soundness and Relevancy Pruning are 0.15-0.2x as fast
- **Section 8.4 (RQ4):** Strong Soundness and Relevancy Pruning are slower for large graphs

TODO

And that's it! Thank you so much for your service as an artifact evaluator!

# Optional: Looking at the AONav codebase

If you would like to take a look at implementation of AONav, please refer to the
file `ARCHITECTURE.md` in the Zenodo code repository (or Github) for how to dive
in! Each module in the codebase also has documentation that should help in
understanding individual modules.

# Optional: Running more examples

If you would like to use `aonav` interactively, run the following command:
```bash
podman run -it aonav /bin/bash
```

This will load a `bash` shell inside the Docker image. You can then run `aonav`
on a file `FILENAME` by running the following command:
```bash
aonav interact FILENAME
```

There are many examples you can run in the `entries` folder in the Docker image.
For example:
```
aonav interact entries/unreduced/aesop-not_proven_Aesop_2.json
```
You can also create your own JSON files modeled after those in the `entries`
folder, but be warned that Docker images delete all saved files inside the image
when they are shut down.
