# %% Imports

import glob

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

datas = []
for path in glob.glob("../results/*.csv"):
    datas.append(pl.read_csv(path))

data = pl.concat(datas).with_columns(
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

# summary = summary.filter(
#     pl.col("total_decisions") > 0,
#     pl.col("provider") != "Remaining",
# )

# % %

baseline_provider = "AlphabeticalUnsound"

baseline = summary.filter(
    pl.col("provider") == baseline_provider,
    pl.col("duration") > 0,
    pl.col("total_decisions") > 0,
    pl.col("unique_decisions") > 0,
)

nonbaseline = summary.filter(
    pl.col("provider") != baseline_provider,
)

comparison = (
    nonbaseline.join(
        baseline,
        on="name",
        validate="m:1",
        suffix="_base",
    ).drop("provider_base")
    # .with_columns(
    #     pl.col("duration") / pl.col("duration_base"),
    #     pl.col("total_decisions") / pl.col("total_decisions_base"),
    #     pl.col("unique_decisions") / pl.col("unique_decisions_base"),
    # )
    # .drop(pl.selectors.ends_with("_base"))
)

summary = summary.join(metadata, on="provider").with_columns(
    suite=pl.col("name").str.split_exact("-", 1).struct.field("field_0"),
)

comparison = comparison.join(metadata, on="provider").with_columns(
    suite=pl.col("name").str.split_exact("-", 1).struct.field("field_0"),
)

scal = (
    summary.filter(
        pl.col("suite") == "random",
    )
    .with_columns(
        size=pl.col("name")
        .str.split_exact("-", 2)
        .struct.field("field_1")
        .cast(pl.Int32),
    )
    .group_by("provider", "size")
    .agg(pl.col("duration").mean())
)

scal = scal.join(metadata, on="provider")

# %% Main


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
    fig, ax = plt.subplots(1, 1, figsize=(5, 2.5))
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
        f"out/01-duration-{suite}.pdf",
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
        f"out/01-total_decisions-{suite}.pdf",
    )

# %%


def compareplot(
    df,
    *,
    x,
    y,
    figtitle,
):
    fig, ax = plt.subplots(1, 1, figsize=(5, 5))

    ax.scatter(
        df[x],
        df[y],
        c="k",
        alpha=0.2,
        zorder=10,
    )

    lim = max(df[x].max(), df[y].max())

    ax.axline(xy1=(0, 0), slope=1, ls="--", c="lightgray", zorder=1)

    ax.set_xlim(0, lim)
    ax.set_ylim(0, lim)

    ax.set_aspect("equal", adjustable="box")
    ax.spines[["top", "right"]].set_visible(False)

    fig.suptitle(figtitle, fontweight="bold", fontsize=14)
    # fig.tight_layout()

    return fig, ax


for (suite, provider), g in comparison.group_by("suite", "provider"):
    compareplot(
        g,
        x="duration_base",
        y="duration",
        figtitle=nice_suite[suite],
    )[0].savefig(
        f"out/02-cmp-duration-{suite}-{provider}.pdf",
    )

    compareplot(
        g,
        x="total_decisions_base",
        y="total_decisions",
        figtitle=nice_suite[suite],
    )[0].savefig(
        f"out/02-cmp-total_decisions-{suite}-{provider}.pdf",
    )

# %%

for (suite, provider, short), g in comparison.group_by(
    "suite", "provider", "short"
):
    dur_multiplier = (g["duration_base"] / g["duration"]).median()
    dec_multiplier = (g["total_decisions_base"] / g["total_decisions"]).median()
    print(
        "\\newcommand{\\MultDur"
        + nice_suite[suite]
        + short
        + "}{"
        + str(round(dur_multiplier, 2))
        + "}"
    )
    print(
        "\\newcommand{\\MultDec"
        + nice_suite[suite]
        + short
        + "}{"
        + str(round(dec_multiplier, 2))
        + "}"
    )

# %%


def scalplot(
    df,
    *,
    x,
    y,
    order="order",
):
    fig, ax = plt.subplots(1, 1, figsize=(7, 3))

    for (label, color, marker), g in df.sort(order).group_by(
        "label",
        "color",
        "marker",
        maintain_order=True,
    ):
        g = g.sort(x)
        ax.plot(
            g[x],
            g[y],
            c=color,
            marker=marker,
            clip_on=False,
            label=label,
        )

    ax.set_xticks(np.arange(0, 201, 20))
    ax.set_yticks(np.arange(0, 20.1, 2))

    ax.set_xlim(0, 200)
    ax.set_ylim(0, 10)

    ax.spines[["top", "right"]].set_visible(False)

    ax.set_xlabel("Size (OR nodes)", fontweight="bold")
    ax.set_ylabel("Duration (s)", fontweight="bold")

    # fig.suptitle(figtitle, fontweight="bold", fontsize=14)
    fig.tight_layout()
    # fig.legend(loc='center left', bbox_to_anchor=(1, 0))
    fig.legend(
        loc="upper left",
        bbox_to_anchor=(0.12, 1),
    )

    return fig, ax


scalplot(
    scal,
    x="size",
    y="duration",
)[0].savefig(
    "out/03-scal.pdf",
    bbox_inches="tight",
)
