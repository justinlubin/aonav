# %% Imports

import matplotlib.pyplot as plt
import numpy as np
import polars as pl

# %% Load data

metadata = pl.read_csv("metadata.csv")

data = pl.read_csv("../results/minimal.csv").with_columns(
    duration=pl.col("duration") / 1000,
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

# remaining = summary.filter(pl.col("provider") == "Remaining")
# nonremaining = summary.filter(pl.col("provider") != "Remaining")
#
# remaining_comparison = (
#     nonremaining.join(
#         remaining,
#         on="name",
#         validate="m:1",
#         suffix="_rem",
#     )
#     .drop("provider_rem")
#     .with_columns(
#         pl.col("duration") / pl.col("duration_rem"),
#         pl.col("total_decisions") / pl.col("total_decisions_rem"),
#         pl.col("unique_decisions") / pl.col("unique_decisions_rem"),
#     )
#     .drop(pl.selectors.ends_with("_rem"))
# )

summary = summary.filter(
    pl.col("total_decisions") > 0,
    pl.col("provider") != "Remaining",
)

summary = summary.join(metadata, on="provider").with_columns(
    suite=pl.col("name").str.split_exact("-", 1).struct.field("field_0"),
)

# % % Main


def catplot(
    df,
    *,
    by,
    val,
    val_label,
    order="order",
    label="label",
):
    fig, ax = plt.subplots(1, 1, figsize=(5, 5))
    ticks = []
    labels = []

    rng = np.random.default_rng(seed=0)

    for x, ((_, title), g) in enumerate(
        df.sort([order, by]).group_by(
            [by, label],
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
            alpha=0.2,
            zorder=10,
        )

        print(title, val, y.mean())
        ax.hlines(
            y=y.mean(),
            xmin=x - 0.25,
            xmax=x + 0.25,
            color="r",
            zorder=20,
            alpha=1,
        )

        ticks.append(x)
        labels.append(title)

    ax.set_xticks(ticks, labels=labels)
    ax.set_xlim([min(ticks) - 0.5, max(ticks) + 0.5])

    if "decision" in val:
        ymax = round(1 + 1.1 * max(df[val]) / 100) * 100
        ax.set_ylim(0, ymax)
        ax.set_yticks(np.arange(0, ymax + 1, 100))

    ax.set_ylabel(val_label)

    ax.spines[["top", "right"]].set_visible(False)

    fig.tight_layout()

    return fig, ax


for (suite,), g in summary.group_by("suite"):
    catplot(
        g,
        by="provider",
        val="duration",
        val_label="Duration (s)",
    )[0].savefig(
        f"out/duration-{suite}.svg",
    )

    catplot(
        g,
        by="provider",
        val="total_decisions",
        val_label="Total decisions",
    )[0].savefig(
        f"out/total_decisions-{suite}.svg",
    )

# catplot(
#     summary,
#     by="provider",
#     val="unique_decisions",
#     val_label="Unique decisions",
# )[0].savefig(
#     "out/unique_decisions.svg",
# )

# catplot(
#     remaining_comparison,
#     by="provider",
#     val="duration",
# )[0].savefig(
#     "out/comparative-duration.svg",
# )
#
# catplot(
#     remaining_comparison,
#     by="provider",
#     val="total_decisions",
# )[0].savefig(
#     "out/comparative-total_decisions.svg",
# )
#
# catplot(
#     remaining_comparison,
#     by="provider",
#     val="unique_decisions",
# )[0].savefig(
#     "out/comparative-unique_decisions.svg",
# )
