# Getting Started Guide

_Estimated time to complete this section: 10 minutes._

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

Please ensure the `run.sh` script from Step 4 above has run to completion. We
estimate this script should take no longer than 1 hour to run, and potentially
take _much_ less time depending on your hardware (it takes about 2 minutes on
an M-series MacBook Pro). The final line of output should say "All done!" when
the evaluation is complete.

The `run.sh` script from Step 4 creates a directory `mnt` that contains the CSVs
of results in the `results/` subdirectory and summary graphs in the `graphs/`
subdirectory.

*Important note:* For the "decision count" metrics below, we changed the
way decisions are counted in the implementation to better reflect the theory
in the paper. This affects all conditions equally (all decision counts are
approximately halved now compared to what they were in the original submission),
so the relative comparisons between conditions should be approximately the same.
To run our evaluation as it was when we submitted the paper, you can run Step 4
with the command `bash run.sh --count-unordered`. We will notify reviewers of
this small change in the cover letter for our revision.

## Step 1: Verify graphs match paper

Please ensure that the generated graphs all roughly match the correspoding
graphs in the paper. They will not match exactly but should demonstrate similar
trends. For the decision counts, the graphs should be very similar but with
approximately half the number of decisions for every condition and suite. For
the durations, the specific timing values will heavily depend on your hardware,
but the speedup/slowdown trends should be the same as in the paper.

- **For Figure 2,** check the three `01-total_decisions-*.pdf` graphs.
- **For Figure 3,** check the three `02-duration-*.pdf` graphs.
- **For Figure 4,** check the `03-scal.pdf` graph.

## Step 2: Verify empirical claims

We make four main empirical claims in the paper:

- **Section 8.1 (RQ1):** Strong Soundness reduces decision count by 1.7-2x
- **Section 8.2 (RQ2):** Relevancy Pruning further reduces decision count by 1.2-3x
- **Section 8.3 (RQ3):** Strong Soundness and Relevancy Pruning are 0.15-0.2x as fast
- **Section 8.4 (RQ4):** Strong Soundness and Relevancy Pruning are slower for large graphs

To verify these claims, please inspect the `summary.txt` file in the `graphs/`
subdirectory and ensure that the entries all follow the same trend as the paper
(they will likely not fall within the exact ranges, especially for durations
due to hardware differences, but they should be close).

This file contains a set of summary statistics, each of the following form:
```
### <STATISTIC> (<SUITE>)

<ALGORITHM 1> vs. <ALGORITHM 2>: ___× (___× better)
... etc.
```

As an example, consider the statistic "Decision Count" just for the "Manual"
suite. Looking at the "Str. Sound vs. Unsound" entry under the "Manual" suite
grouping will answer RQ1 for the "Manual" suite (which we list as 1.72× in the
paper). Looking at the "Str. Sound + Rel. vs. Str. Sound" entry will answer RQ2
for the "Manual" suite (which we list as 1.54× in the paper).

For a visual aid in this task, you can also take a look at the
`04-forest-decisions-*.pdf` and `05-forest-duration-*.pdf` graphs.

Once you have verified RQ1 through RQ4 using `summary.txt`, that's it!

Thank you so much for your service as an artifact evaluator!

# Optional: Looking at the `aonav` codebase

If you would like to take a look at implementation of `aonav`, please refer to
the file `ARCHITECTURE.md` in the Zenodo code repository (or Github) for how to
dive in! Each module in the codebase also has documentation that should help in
understanding individual modules.

For example, you will see where Sections 4, 5, and 6 of the paper are
implemented in the codebase. Each of our step providers in the paper has a
corresponding implementation in the Rust code.

# Optional: Running more examples

If you would like to use `aonav` interactively, run the following command:
```bash
podman run -it --entrypoint /bin/bash aonav
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
