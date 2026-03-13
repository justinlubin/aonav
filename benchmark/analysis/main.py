# %% Imports

import glob
import os
import sys

import matplotlib
import matplotlib.pyplot as plt
import numpy as np
import polars as pl
from matplotlib import font_manager as fm

MINIMAL = len(sys.argv) > 1 and sys.argv[1] == "MINIMAL"

if MINIMAL:
    plt.rcParams["font.cursive"] = "DejaVu Sans"
    for f in fm.findSystemFonts():
        try:
            name = fm.FontProperties(fname=f).get_name()
            if "Keyboard" in name:
                continue
            if "Linux Libertine" in name:
                SERIF_FONT = name
            elif "Linux Biolinum" in name:
                SANS_SERIF_FONT = name
        except Exception:
            continue
else:
    SERIF_FONT = "Linux Libertine"
    SANS_SERIF_FONT = "Linux Biolinum"

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


def save(self, filename, exts=None, *args, **kwargs):
    dirname = os.path.dirname(filename)
    basename = os.path.basename(filename)

    os.makedirs(dirname, exist_ok=True)

    if exts is not None:
        for ext in exts:
            new_dirname = os.path.join(dirname, ext)
            os.makedirs(new_dirname, exist_ok=True)
            self.savefig(
                os.path.join(new_dirname, basename + "." + ext),
                *args,
                **kwargs,
            )
    else:
        self.savefig(filename, *args, **kwargs)

    plt.close(self)


matplotlib.figure.Figure.save = save  # ty:ignore[possibly-missing-attribute]

# %% Load data


def attach_suite(df):
    return df.with_columns(
        suite=pl.col("name").str.split_exact("-", 1).struct.field("field_0"),
    )


nice_suite = {
    "manual": "Manual",
    "random": "Random",
    "argusReduced": "Argus",
    "aesop": "Aesop",
}

suite_order = ["manual", "random", "argusReduced", "aesop"]

metadata = pl.read_csv("metadata.csv")

suite_datas = []
for path in glob.glob("../entries/*.csv"):
    suite_datas.append(pl.read_csv(path))

suite_data = (
    attach_suite(pl.concat(suite_datas))
    .with_columns(
        consumer_count=pl.col("consumer_count")
        .str.split(";")
        .list.eval(pl.element().cast(pl.Int64)),
        provider_count=pl.col("provider_count")
        .str.split(";")
        .list.eval(pl.element().cast(pl.Int64)),
        premise_count=pl.col("premise_count")
        .str.split(";")
        .list.eval(pl.element().cast(pl.Int64)),
    )
    .with_columns(
        or_count=pl.col("consumer_count").list.len(),
        and_count=pl.col("premise_count").list.len(),
        consumer_count_median=pl.col("consumer_count").list.median(),
        provider_count_median=pl.col("provider_count").list.median(),
        premise_count_median=pl.col("premise_count").list.median(),
    )
    .select(
        "name",
        "suite",
        "depth",
        "or_count",
        "and_count",
        "consumer_count_median",
        "provider_count_median",
        "premise_count_median",
    )
)


def summarize(xs):
    mid = xs.median()
    lo = xs.quantile(0.25)
    hi = xs.quantile(0.75)
    return f"{mid:0.2f} ({lo:0.2f}--{hi:0.2f})"


if not MINIMAL:
    print(
        "\\textsc{\\textbf{Suite}}",
        "\\textsc{\\textbf{Depth}}",
        "\\textsc{\\textbf{\\# ORs}}",
        "\\textsc{\\textbf{\\# ANDs}}",
        "\\textsc{\\textbf{Median \\# consumers}}",
        "\\textsc{\\textbf{Median \\# providers}}",
        "\\textsc{\\textbf{Median \\# premises}} \\\\",
        sep=" & ",
    )
    print("\\midrule")
    for (s,), g in suite_data.sort("suite").group_by(
        "suite", maintain_order=True
    ):
        print(
            "\\textsc{" + nice_suite[s] + "}",
            summarize(g["depth"]),
            summarize(g["or_count"]),
            summarize(g["and_count"]),
            summarize(g["consumer_count_median"]),
            summarize(g["provider_count_median"]),
            summarize(g["premise_count_median"]) + " \\\\",
            sep=" & ",
        )

# %%


def attach_metadata(df):
    df = attach_suite(df.join(metadata, on="provider"))

    if MINIMAL:
        df = df.filter(
            pl.col("suite").is_in(
                [
                    "manual",
                    "random",
                    "argusReduced",
                ]
            ),
            pl.col("provider").is_in(
                [
                    "AlphabeticalUnsound",
                    "AlphabeticalComplete",
                    "AlphabeticalRelevant",
                ]
            ),
        )

    return df


datas = []
for path in glob.glob("../results/NON_INCR-*.csv"):
    datas.append(pl.read_csv(path, infer_schema_length=10_000))

data = (
    pl.concat(datas)
    .with_columns(
        duration=pl.when(pl.col("success"))
        .then(pl.col("duration") / 1000)
        .otherwise(np.nan),
        decisions=pl.when(pl.col("success"))
        .then(pl.col("total_decisions"))
        .otherwise(np.nan),
        latencies=pl.when(pl.col("success"))
        .then(
            pl.col("latencies")
            .str.split(";")
            .list.eval(pl.element().cast(pl.Float64) / 1000)
        )
        .otherwise([]),
    )
    .with_columns(
        latency_median=pl.col("latencies").list.median(),
        latency_max=pl.col("latencies").list.max(),
        rounds=pl.col("latencies").list.len(),
    )
    .select(
        "name",
        "provider",
        "chosen_solution",
        "replicate",
        "success",
        "duration",
        "decisions",
        "latency_median",
        "latency_max",
        "rounds",
    )
    .sort(pl.col("*"))
)

agg = (
    data.group_by("provider", "name", "chosen_solution")
    .agg(
        pl.col("success").all(),
        pl.col("duration").mean(),
        pl.col("decisions").mean(),
        pl.col("latency_median").mean(),
        pl.col("latency_max").mean(),
        pl.col("rounds").mean(),
    )
    .group_by("provider", "name")
    .agg(
        pl.col("success").all(),
        pl.col("duration").mean(),
        pl.col("decisions").mean(),
        pl.col("latency_median").mean(),
        pl.col("latency_max").mean(),
        pl.col("rounds").mean(),
    )
    .select(
        "name",
        "provider",
        "duration",
        "success",
        "decisions",
        "latency_median",
        "latency_max",
        "rounds",
    )
    .sort(pl.col("*"))
)

agg = attach_metadata(agg)


comparisons = agg.join(agg, on="name", suffix="_baseline")

scal = (
    agg.filter(
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
    .join(metadata, on="provider")
)


# %% Main

TOTAL_ENTRIES = {}
SUCCESS_COUNT = {}

for (s,), g in suite_data.group_by("suite"):
    TOTAL_ENTRIES[s] = g.height

for (p, s), g in agg.group_by(
    "provider",
    "suite",
    maintain_order=True,
):
    SUCCESS_COUNT[(p, s)] = g.filter(pl.col("success")).height

# %%


def catplot(
    df,
    *,
    val,
    val_label,
    figtitle,
    prefix,
    places,
    suite,
    provider="provider",
    short="short",
    order="order",
    label="label",
):
    fig, ax = plt.subplots(1, 1, figsize=(5.5, 4))
    ticks = []
    labels = []

    rng = np.random.default_rng(seed=0)

    m = max(df[val].filter(df[val].is_not_null()))
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

    ymax = int(2 + m / ydelta) * ydelta

    divider_added = False

    for x, ((provider, title, short), g) in enumerate(
        df.sort([order, provider]).group_by(
            [provider, label, short],
            maintain_order=True,
        )
    ):
        y = g[val].filter(g[val].is_not_null())

        jitter = rng.uniform(
            low=-0.25,
            high=0.25,
            size=len(y),
        )

        ax.scatter(
            x + jitter,
            y,
            c="k",
            alpha=0.2,
            zorder=10,
        )

        # rhs = str(round(y.mean(), places) if places > 0 else round(y.mean()))
        # print("\\newcommand{\\" + prefix + figtitle + short + "}{" + rhs + "}")

        success_count = SUCCESS_COUNT[(provider, suite)]
        total_entries = TOTAL_ENTRIES[suite]

        if success_count == total_entries:
            ax.hlines(
                y=y.mean(),
                xmin=x - 0.25,
                xmax=x + 0.25,
                color="r",
                zorder=20,
                alpha=1,
            )

        ax.annotate(
            f"{success_count}/{total_entries}",
            xy=(x, ymax),
            xytext=(0, 0),
            textcoords="offset pixels",
            va="bottom",
            ha="center",
            fontsize=9,
            color="#999999" if success_count == total_entries else "#bf4040",
            # bbox=dict(
            #     boxstyle="square,pad=0",
            #     facecolor="white",
            #     edgecolor="none",
            # ),
        )

        if success_count != total_entries and not divider_added:
            # ax.vlines(
            #     x=x - 0.5,
            #     ymin=0,
            #     ymax=ymax,
            #     color="0.8",
            #     linestyle="--",
            #     lw=0.5,
            # )
            divider_added = True

        ticks.append(x)
        labels.append(title)

    ax.set_xticks(ticks, labels=labels, fontweight="bold")
    ax.set_xlim(min(ticks) - 0.5, max(ticks) + 0.5)

    ax.set_ylim(0, ymax)
    ax.set_yticks(np.arange(0, ymax + 0.000001, ydelta))

    ax.set_ylabel(val_label, fontweight="bold")

    ax.spines[["top", "right"]].set_visible(False)

    fig.suptitle(figtitle, fontweight="bold", fontsize=14)
    fig.tight_layout()

    return fig, ax


for (suite,), g in agg.group_by("suite"):
    sn = suite_order.index(suite)

    catplot(
        g,
        val="decisions",
        val_label="Total decisions",
        figtitle=nice_suite[suite],
        prefix="ResDec",
        places=0,
        suite=suite,
    )[0].save(
        f"out/01-total_decisions-{sn}-{suite}.pdf",
    )

    catplot(
        g,
        val="duration",
        val_label="Duration (s)",
        figtitle=nice_suite[suite],
        prefix="ResDur",
        places=2,
        suite=suite,
    )[0].save(
        f"out/02-duration-{sn}-{suite}.pdf",
    )

    if not MINIMAL:
        pass
        # catplot(
        #     g,
        #     val="rounds",
        #     val_label="Total rounds",
        #     figtitle=nice_suite[suite],
        #     prefix="ResRou",
        #     places=0,
        #     suite=suite,
        # )[0].save(
        #     f"out/08-rounds-{suite}.pdf",
        # )

for (suite, provider, short), g in (
    comparisons.filter(
        pl.col("provider") != "AlphabeticalUnsound",
        pl.col("provider_baseline") == "AlphabeticalUnsound",
    )
    .sort("suite", "provider", "short")
    .group_by("suite", "provider", "short", maintain_order=True)
):
    if MINIMAL:
        continue
    # print(g[["provider", "provider_baseline"]])
    dur_multiplier = (g["duration_baseline"] / g["duration"]).median()
    dec_multiplier = (g["decisions_baseline"] / g["decisions"]).median()
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
    ax.set_ylim(0, (int(df[y].max()) // 2) * 2 + 2)

    ax.spines[["top", "right"]].set_visible(False)

    ax.set_xlabel("Size (OR nodes)", fontweight="bold")
    ax.set_ylabel("Duration (s)", fontweight="bold")

    # fig.suptitle(figtitle, fontweight="bold", fontsize=14)
    fig.tight_layout()
    # fig.legend(loc='center left', bbox_to_anchor=(1, 0))
    fig.legend(
        loc="upper right",
        bbox_to_anchor=(0.98, 1),
    )

    return fig, ax


scalplot(
    scal,
    x="size",
    y="duration",
)[0].save(
    "out/03-scal.pdf",
    bbox_inches="tight",
)


def forest_plot(cmp, *, feature, title, median_color):
    if MINIMAL:
        print("###", title)

    rng = np.random.default_rng(seed=0)

    fig, ax = plt.subplots(1, 1)

    lim = 0

    pairs = []

    separator_rendered = False
    previous_baseline = None

    for (provider, provider_baseline), g in cmp.sort(
        "order_baseline", "order"
    ).group_by(
        "label",
        "label_baseline",
        maintain_order=True,
    ):
        if provider == provider_baseline:
            continue
        if (provider, provider_baseline) in pairs:
            continue
        if (provider_baseline, provider) in pairs:
            continue
        pairs.append((provider, provider_baseline))

        y = -len(pairs)

        if (
            not separator_rendered
            and previous_baseline is not None
            and provider_baseline != previous_baseline
        ):
            # ax.axhline(
            #     y=y + 0.5,
            #     color="0.8",
            #     linestyle="--",
            #     lw=0.5,
            # )
            separator_rendered = True

        previous_baseline = provider_baseline

        multiplier = (g[feature] + 0.001) / (g[f"{feature}_baseline"] + 0.001)
        multiplier = multiplier.filter(multiplier.is_not_null()).log(base=2)
        median = multiplier.median()

        y_jitter = rng.uniform(low=y - 0.1, high=y + 0.1, size=len(multiplier))
        ax.scatter(multiplier, y_jitter, color="0.2", alpha=0.05)
        ax.vlines(
            x=median,
            ymin=y - 0.25,
            ymax=y + 0.25,
            color=median_color,
            zorder=20,
            alpha=1,
        )

        label_val = 2**median
        label = f"{label_val:0.2f}× ({1 / label_val:0.2f}× better)"
        if MINIMAL:
            print(
                "- ", provider, " vs. ", provider_baseline, ": ", label, sep=""
            )
        ax.annotate(
            label,
            xy=(median, y),
            xytext=(5, -2),
            textcoords="offset pixels",
            va="top",
            ha="left",
            color=median_color,
            bbox=dict(
                boxstyle="square,pad=0",
                facecolor="white",
                edgecolor="none",
            ),
        )

        lim = max(lim, multiplier.abs().max())

    if MINIMAL:
        print()

    ax.axvline(0, color="0.5", ls="dashed")

    lim = np.ceil(lim + 0.1)

    ax.set_xlim(-lim, lim)
    xticks = np.arange(-lim, lim + 1, 1)

    def nice_xtick(xt):
        if xt < 0:
            prefix = "1/"
        else:
            prefix = ""
        return prefix + str(int(2 ** abs(xt))) + "×"

    ax.set_xticks(xticks, labels=map(nice_xtick, xticks))

    ax.set_xlabel(f"{title} ratio", fontweight="bold")

    def _bold(s):
        return r"$\bf{" + s.replace(" ", r"\ ") + r"}$"

    ax.set_yticks(
        np.arange(-len(pairs), 0),
        labels=[
            _bold(trt) + " vs. " + _bold(ctrl)
            for (trt, ctrl) in reversed(pairs)
        ],
    )
    ax.set_ylim(-len(pairs) - 1, 0)
    # ax2 = ax.twinx()
    # ax2.set_yticks(
    # np.arange(-len(pairs), 0),
    # labels=[p[1] for p in reversed(pairs)],
    # )
    # ax2.set_ylim(-len(pairs) - 1, 0)

    ax.spines[["top", "right", "left"]].set_visible(False)
    ax.tick_params(axis="y", which="both", length=0)

    fig.tight_layout()
    return fig, ax


if not MINIMAL:
    forest_plot(
        comparisons,
        feature="decisions",
        title="Decision count",
        median_color="red",
    )[0].save("out/06-forest-decisions.pdf")

    forest_plot(
        comparisons, feature="duration", title="Duration", median_color="red"
    )[0].save("out/07-forest-duration.pdf")

cmp = comparisons

if not MINIMAL:
    cmp = comparisons.filter(
        pl.col("provider_baseline") == "AlphabeticalUnsound"
    )

for (suite,), g in cmp.sort("suite").group_by("suite", maintain_order=True):
    forest_plot(
        g,
        feature="decisions",
        title=f"Decision count ({nice_suite[suite]})",
        median_color="orange",
    )[0].save(f"out/04-forest-decisions-{suite}.pdf")

    forest_plot(
        g,
        feature="duration",
        title=f"Duration ({nice_suite[suite]})",
        median_color="orange",
    )[0].save(f"out/05-forest-duration-{suite}.pdf")

    # if not MINIMAL:
    #     forest_plot(
    #         g,
    #         feature="rounds",
    #         title=f"Rounds ({nice_suite[suite]})",
    #         median_color="orange",
    #     )[0].save(f"out/09-forest-rounds-{suite}.pdf")

# %%
