# %% Imports

import matplotlib.pyplot as plt
import numpy as np
import polars as pl

# %% Load data

data = pl.read_csv("../results/prelim.csv").with_columns(
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
    .agg(
        pl.col("duration").mean(),
        pl.col("total_decisions").mean(),
        pl.col("unique_decisions").mean(),
    )
)

remaining = summary.filter(pl.col("provider") == "Remaining")
nonremaining = summary.filter(pl.col("provider") != "Remaining")

remaining_comparison = (
    nonremaining.join(
        remaining,
        on="name",
        validate="m:1",
        suffix="_rem",
    )
    .drop("provider_rem")
    .with_columns(
        pl.col("duration") / pl.col("duration_rem"),
        pl.col("total_decisions") / pl.col("total_decisions_rem"),
        pl.col("unique_decisions") / pl.col("unique_decisions_rem"),
    )
    .drop(pl.selectors.ends_with("_rem"))
)

summary = summary.filter(
    pl.col("total_decisions") > 0,
    pl.col("provider") != "Remaining",
)


# % % Main


def catplot(df, *, by, val):
    fig, ax = plt.subplots(1, 1, figsize=(5, 3))
    ticks = []
    labels = []

    rng = np.random.default_rng(seed=0)

    for x, ((title,), g) in enumerate(
        df.sort(by).group_by(
            by,
            maintain_order=True,
        )
    ):
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
            alpha=0.3,
            zorder=10,
        )

        print(title, val, y.mean())
        ax.hlines(
            y=y.mean(),
            xmin=x - 0.25,
            xmax=x + 0.25,
            color="r",
            zorder=20,
            alpha=0.8,
        )

        ticks.append(x)
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
    val="unique_decisions",
)[0].savefig(
    "out/unique_decisions.svg",
)

catplot(
    remaining_comparison,
    by="provider",
    val="duration",
)[0].savefig(
    "out/comparative-duration.svg",
)

catplot(
    remaining_comparison,
    by="provider",
    val="total_decisions",
)[0].savefig(
    "out/comparative-total_decisions.svg",
)

catplot(
    remaining_comparison,
    by="provider",
    val="unique_decisions",
)[0].savefig(
    "out/comparative-unique_decisions.svg",
)
