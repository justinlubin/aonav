# %% Imports

import matplotlib.pyplot as plt
import numpy as np
import polars as pl

# %% Load data

data = pl.read_csv("../results/results.csv").with_columns(
    pl.col("duration") / 1000,
)

summary = (
    data.group_by("provider", "name", "chosen_solution")
    .agg(
        pl.col("duration").mean(),
        pl.col("total_decisions").mean(),
        pl.col("unique_decisions").mean(),
    )
    .group_by("provider", "name")
    .mean()
)

# %% Main


def catplot(df, *, by, val):
    fig, ax = plt.subplots(1, 1, figsize=(5, 3))
    ticks = []
    labels = []

    rng = np.random.default_rng(seed=0)

    for i, ((title,), g) in enumerate(df.group_by(by)):
        x = np.repeat(
            i,
            len(g),
        )

        jitter = rng.uniform(
            low=-0.25,
            high=0.25,
            size=len(g),
        )

        y = g[val]

        ax.scatter(
            x + jitter,
            y,
            c="k",
            alpha=0.7,
            zorder=10,
        )

        ax.hlines(
            y=y.mean(),
            xmin=x - 0.25,
            xmax=x + 0.25,
            color="r",
            zorder=20,
            alpha=0.7,
        )

        ticks.append(i)
        labels.append(title)

    ax.set_xticks(ticks, labels=labels)
    ax.set_xlim([min(ticks) - 1, max(ticks) + 1])
    ax.set_ylim(0, 1.1 * max(df[val]))

    return fig, ax


catplot(
    summary,
    by="provider",
    val="duration",
)[0].savefig(
    "out/duration.svg",
)

catplot(
    summary,
    by="provider",
    val="total_decisions",
)[0].savefig(
    "out/total_decisions.svg",
)

catplot(
    summary,
    by="provider",
    val="total_decisions",
)[0].savefig(
    "out/unique_decisions.svg",
)
