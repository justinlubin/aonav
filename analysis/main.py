# %% Imports

import matplotlib.pyplot as plt
import numpy as np
import polars as pl

SERIF_FONT = "Linux Libertine O"
SANS_SERIF_FONT = "Linux Biolinum O"

plt.rcParams.update(
    {
        "pdf.fonttype": 42,
        "font.family": [SANS_SERIF_FONT],
        "mathtext.fontset": "custom",
        "mathtext.rm": SANS_SERIF_FONT,
        "mathtext.it": SANS_SERIF_FONT + ":italic",
        "mathtext.bf": SANS_SERIF_FONT + ":bold",
    }
)

# %% Load data

nice_suite = {
    "manual": "Manual",
    "random": "Random",
    "argus": "Argus",
}

metadata = pl.read_csv("metadata.csv")

data = pl.read_csv("../results/combined.csv").with_columns(
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
    figtitle,
    prefix,
    places,
    short="short",
    order="order",
    label="label",
):
    fig, ax = plt.subplots(1, 1, figsize=(5, 5))
    ticks = []
    labels = []

    rng = np.random.default_rng(seed=0)

    for x, ((_, title, short), g) in enumerate(
        df.sort([order, by]).group_by(
            [by, label, short],
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

        rhs = str(round(y.mean(), places) if places > 0 else round(y.mean()))
        print("\\newcommand{\\" + prefix + figtitle + short + "}{" + rhs + "}")

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

    ax.set_xticks(ticks, labels=labels, fontweight="bold")
    ax.set_xlim([min(ticks) - 0.5, max(ticks) + 0.5])

    m = max(df[val])
    if m < 1:
        ydelta = 0.05
    elif m < 5:
        ydelta = 0.5
    elif m < 30:
        ydelta = 5
    elif m < 1000:
        ydelta = 50
    else:
        ydelta = 100

    ymax = int(1 + max(df[val]) / ydelta) * ydelta
    ax.set_ylim(0, ymax)
    ax.set_yticks(np.arange(0, ymax + 0.000001, ydelta))

    ax.set_ylabel(val_label, fontweight="bold")

    ax.spines[["top", "right"]].set_visible(False)

    fig.suptitle(figtitle, fontweight="bold", fontsize=14)
    fig.tight_layout()

    return fig, ax


for (suite,), g in summary.group_by("suite"):
    catplot(
        g,
        by="provider",
        val="duration",
        val_label="Duration (s)",
        figtitle=nice_suite[suite],
        prefix="ResDur",
        places=2,
    )[0].savefig(
        f"out/duration-{suite}.pdf",
    )

    catplot(
        g,
        by="provider",
        val="total_decisions",
        val_label="Total decisions",
        figtitle=nice_suite[suite],
        prefix="ResDec",
        places=0,
    )[0].savefig(
        f"out/total_decisions-{suite}.pdf",
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
