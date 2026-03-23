# %% Imports

import glob
import os

import matplotlib
import matplotlib.pyplot as plt
import numpy as np
import polars as pl

# % % Matplotlib settings and patching

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

# % % Load suite data


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

SECONDS_SUFFIX = " (s)"
metrics = [
    ("decisions", "Decision count"),
    ("rounds", "Round count"),
    ("duration", "Duration" + SECONDS_SUFFIX),
    ("latency_median", "Median latency" + SECONDS_SUFFIX),
]

timing_metrics = {"duration", "latency_median"}

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

# TODO figure out filtering?
# suite_data = suite_data.filter(
#     (pl.col("depth") > 2),
# )

# % % Make summary table of suite data


def summarize(xs):
    mid = xs.median()
    lo = xs.quantile(0.25)
    hi = xs.quantile(0.75)
    if mid == round(mid):
        mid = round(mid)
    if lo == round(lo):
        lo = round(lo)
    if hi == round(hi):
        hi = round(hi)
    return f"{mid} ({lo}--{hi})"


print(
    r"\newcommand{\NumBenchmarkTotal}{",
    suite_data.height,
    "}",
    sep="",
)

for s in suite_order:
    g = suite_data.filter(pl.col("suite") == s)
    print(
        r"\newcommand{\NumBenchmark",
        nice_suite[s],
        "}{",
        g.height,
        "}",
        sep="",
    )

# %%

print()
print(
    "\\textsc{\\textbf{Suite}}",
    "\\textsc{\\textbf{Depth}}",
    "\\textsc{\\textbf{ORs}}",
    "\\textsc{\\textbf{ANDs}}",
    "\\textsc{\\textbf{Consumers}}",
    "\\textsc{\\textbf{Providers}}",
    "\\textsc{\\textbf{Premises}} \\\\",
    sep=" & ",
)
print("\\midrule")
for (s,), g in suite_data.sort("suite").group_by(
    "suite",
    maintain_order=True,
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
print()

# % % Load benchmark data


def attach_metadata(df):
    df = attach_suite(df.join(metadata, on="provider"))

    return df


datas = []
for incr in [False, True]:
    if incr:
        prefix = "INCR"
    else:
        prefix = "NON_INCR"
    for path in glob.glob(f"../results/{prefix}-*.csv"):
        datas.append(
            pl.read_csv(path, infer_schema_length=10_000).with_columns(
                incremental=pl.lit(incr)
            )
        )

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
        latency_median=pl.when(pl.col("success"))
        .then(pl.col("latencies").list.median())
        .otherwise(np.nan),
        latency_max=pl.when(pl.col("success"))
        .then(pl.col("latencies").list.max())
        .otherwise(np.nan),
        rounds=pl.when(pl.col("success"))
        .then(pl.col("latencies").list.len())
        .otherwise(np.nan),
    )
    .select(
        "incremental",
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
    data.group_by("incremental", "provider", "name", "chosen_solution")
    .agg(
        pl.col("success").all(),
        pl.col("duration").mean(),
        pl.col("decisions").mean(),
        pl.col("latency_median").mean(),
        pl.col("latency_max").mean(),
        pl.col("rounds").mean(),
    )
    .group_by("incremental", "provider", "name")
    .agg(
        pl.col("success").all(),
        pl.col("duration").mean(),
        pl.col("decisions").mean(),
        pl.col("latency_median").mean(),
        pl.col("latency_max").mean(),
        pl.col("rounds").mean(),
    )
    .select(
        "incremental",
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

agg = agg.join(suite_data, on="name").drop(pl.col("^.*_right$"))

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


# % % Compute success / total benchmark counts

TOTAL_ENTRIES = {}
SUCCESS_COUNT = {}

for (s,), g in suite_data.group_by("suite"):
    TOTAL_ENTRIES[s] = g.height

for (i, p, s), g in agg.group_by(
    "incremental",
    "provider",
    "suite",
    maintain_order=True,
):
    SUCCESS_COUNT[(i, p, s)] = g.filter(pl.col("success")).height

# % % Make max latency plots

print("", *[nice_suite[s] for s in suite_order], sep=" & ")
print("\\midrule")

overall_worst_latency = 0
for (p,), g in (
    agg.filter(pl.col("incremental"))
    .sort("order")
    .group_by("short", maintain_order=True)
):
    if not g["success"].all():
        continue
    overall_worst_latency = max(
        overall_worst_latency,
        round(g["latency_max"].max() * 1000),
    )

for (p,), g1 in (
    agg.filter(pl.col("incremental"))
    .sort("order")
    .group_by("short", maintain_order=True)
):
    if not g1["success"].all():
        break
    vals = []
    for s in suite_order:
        g2 = g1.filter(pl.col("suite") == s)
        worst_latency = round(g2["latency_max"].max() * 1000)
        if worst_latency == overall_worst_latency:
            v = r"\textbf{" + str(worst_latency) + " ms}"
        elif worst_latency == 0:
            v = "<1 ms"
        else:
            v = str(worst_latency) + " ms"
        vals.append(v)
    print(p, *vals, sep=" & ")

# % % Make catplots


def catplot(
    df,
    *,
    val,
    val_label,
    suite,
    incremental,
    show_title,
    provider="provider",
    short="short",
    order="order",
    prefix=None,
    places=0,
):
    fig, ax = plt.subplots(1, 1, figsize=(3, 1.3))
    ticks = []
    labels = []
    colors = []

    rng = np.random.default_rng(seed=0)

    m = max(df[val].filter(df[val].is_not_null()))
    if m < 1.5:
        ydelta = 0.3
        ymax = 1.5
    elif m < 25:
        ydelta = 5
        ymax = 25
    elif m < 40:
        ydelta = 10
        ymax = 40
    elif m < 100:
        ydelta = 20
        ymax = 100
    elif m < 200:
        ydelta = 50
        ymax = 200
    else:
        ydelta = 100
        ymax = m

    divider_added = False

    for x, ((provider, short, color, marker), g) in enumerate(
        df.sort([order, provider]).group_by(
            [provider, short, "color", "marker"],
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
            c=color,
            # marker=marker,
            alpha=0.2,
            zorder=10,
        )

        # rhs = str(round(y.mean(), places) if places > 0 else round(y.mean()))
        # print("\\newcommand{\\" + prefix + figtitle + short + "}{" + rhs + "}")

        success_count = SUCCESS_COUNT[(incremental, provider, suite)]
        total_entries = TOTAL_ENTRIES[suite]

        if success_count > total_entries / 2:
            median = y.median()
            if median < 1:
                median_text = f"{median:0.2f}"
            else:
                median_text = f"{median:0.1f}"

            ax.hlines(
                y=median,
                xmin=x - 0.35,
                xmax=x + 0.35,
                color="k",
                zorder=20,
                alpha=1,
            )

            ax.annotate(
                median_text,
                xy=(x, median),
                xytext=(0, 2),
                fontsize=8,
                textcoords="offset pixels",
                va="bottom",
                ha="center",
                color="k",
                bbox=dict(
                    boxstyle="square,pad=0.1",
                    facecolor="white",
                    alpha=0.8,
                    edgecolor="none",
                ),
                zorder=30,
            )

        ax.annotate(
            f"{success_count}/{total_entries}",
            xy=(x, ymax),
            xytext=(0, 0),
            textcoords="offset pixels",
            va="bottom",
            ha="center",
            fontsize=7,
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
        labels.append(short)
        colors.append(color)

    ax.set_xticks(
        ticks,
        labels=labels,
        fontsize=8,
        # rotation=90,
        # ha="right",
    )

    # for t, c in zip(ax.get_xticklabels(), colors):
    #     t.set_color(c)

    ax.set_xlim(min(ticks) - 0.5, max(ticks) + 0.5)

    ax.set_ylim(0, ymax)
    ax.set_yticks(np.arange(0, ymax + 0.000001, ydelta))

    ax.set_ylabel(val_label, fontweight="bold")

    ax.spines[["top", "right"]].set_visible(False)

    if show_title:
        # figtitle = f"{nice_suite[suite]}: {val_label}"
        figtitle = nice_suite[suite]
        fig.suptitle(figtitle, fontweight="bold", fontsize=14)

    fig.tight_layout()

    return fig, ax


for (
    incremental,
    suite,
), g in agg.group_by("incremental", "suite"):
    sn = suite_order.index(suite)
    if incremental:
        prefix = "Fig"
    else:
        prefix = "Sup"
    for mn, (metric, metric_name) in enumerate(metrics):
        catplot(
            g,
            val=metric,
            val_label=metric_name,
            suite=suite,
            incremental=incremental,
            show_title=False,
        )[0].save(
            f"out/{prefix}1-descriptive/incr{incremental}-{mn}{sn}-{suite}-{metric}-descriptive.pdf",
        )


# %% Make forest plots


def forest_plot(
    cmp,
    *,
    feature,
    title,
    feature_label,
    median_color,
    prefix,
    print_key=False,
    ablation=False,
    jitter_amount=0,
    include=None,
):
    rng = np.random.default_rng(seed=0)

    if ablation:
        fig, ax = plt.subplots(1, 1, figsize=(4, 2.5))
    elif include is not None:
        fig, ax = plt.subplots(1, 1, figsize=(5, 2.5))
    else:
        fig, ax = plt.subplots(1, 1, figsize=(6, 4))

    lim = 0

    pairs = []

    separator_rendered = False
    previous_baseline = None

    for (provider, provider_baseline), g in cmp.sort(
        "order_baseline", "order"
    ).group_by(
        "short",
        "short_baseline",
        maintain_order=True,
    ):
        if include is not None and (provider, provider_baseline) not in include:
            continue
        if ablation:
            if provider != provider_baseline:
                continue
        else:
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

        y_jitter = rng.uniform(
            low=y - jitter_amount,
            high=y + jitter_amount,
            size=len(multiplier),
        )

        ax.scatter(
            multiplier,
            y_jitter,
            color="0.2",
            alpha=0.05,
        )

        # ax.boxplot(
        #     multiplier,
        #     positions=[y],
        #     orientation="horizontal",
        #     widths=0.7,
        #     showfliers=False,
        #     whis=0,
        #     boxprops=dict(color="red"),
        # )

        ax.vlines(
            x=median,
            ymin=y - 0.35,
            ymax=y + 0.35,
            color=median_color,
            zorder=20,
            alpha=1,
        )

        # ax.vlines(
        #     x=[multiplier.quantile(0.25), multiplier.quantile(0.75)],
        #     ymin=y - 0.15,
        #     ymax=y + 0.15,
        #     color=median_color,
        #     zorder=20,
        #     alpha=1,
        # )

        label_val = 2**median
        label = f"{label_val:0.2f}×"

        try:
            if label_val == int(label_val):
                print_label = str(int(label_val))
            else:
                print_label = f"{label_val:0.2f}"
        except ValueError:
            print_label = f"{label_val:0.2f}"

        print(
            r"\newcommand{\Cmp",
            prefix,
            title.replace(" ", ""),
            "X",
            feature.replace("_", ""),
            "X",
            provider.replace("+", ""),
            "v",
            provider_baseline.replace("+", ""),
            "}{",
            print_label,
            "}",
            sep="",
        )
        # label = f"{label_val:0.2f}× ({1 / label_val:0.2f}× better)"

        ax.annotate(
            label,
            xy=(median, y),
            xytext=(3, -3),
            textcoords="offset pixels",
            va="center",
            ha="left",
            color=median_color,
            fontsize=7,
            bbox=dict(
                boxstyle="square,pad=0.1",
                facecolor="white",
                alpha=0.8,
                edgecolor="none",
            ),
        )

        lim = max(lim, multiplier.abs().max())

    ax.axvline(0, color="0.5", ls="dashed", lw=0.5)

    lim = ((3 + np.ceil(lim + 0.1)) // 4) * 4

    ax.set_xlim(-lim, lim)
    if lim > 7:
        xticks = np.arange(-lim, lim + 1, 4)
    else:
        xticks = np.arange(-lim, lim + 1, 1)

    def nice_xtick(xt):
        if xt < 0:
            prefix = "1/"
        else:
            prefix = ""
        return prefix + str(int(2 ** abs(xt))) + "×"

    ax.set_xticks(xticks, labels=map(nice_xtick, xticks))

    # ax.set_xlabel(f"{feature_label} ratio", fontweight="bold")
    ax.annotate(
        feature_label.replace(" ", "\n"),
        xy=(1, 1),
        xycoords="axes fraction",
        ha="right",
        va="top",
        fontweight="bold",
        fontsize=14,
    )

    def _bold(s):
        return r"$\bf{" + s.replace(" ", r"\ ") + r"}$"

    ylabels = []
    for n, (trt, ctrl) in reversed(list(enumerate(pairs))):
        if print_key:
            print(
                r"\newcommand{\CmpKey",
                trt.replace("+", ""),
                "v",
                ctrl.replace("+", ""),
                "}{",
                n,
                "}",
                sep="",
            )
        if ablation:
            ylabels.append(_bold(trt))
        else:
            ylabels.append(_bold(trt) + " / " + _bold(ctrl))

    ax.set_yticks(
        np.arange(-len(pairs), 0),
        labels=ylabels,
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

    # fig.suptitle(title, fontweight="bold")

    fig.tight_layout()
    return fig, ax


# Comparisons among incremental step providers (4 suites x 4 metrics)

print_key = True
for (
    incremental,
    suite,
), g in comparisons.group_by("incremental", "suite"):
    sn = suite_order.index(suite)
    if incremental:
        prefix = "Fig"
    else:
        prefix = "Sup"
    for mn, (metric, metric_name) in enumerate(metrics):
        forest_plot(
            g,
            feature=metric,
            title=nice_suite[suite],
            feature_label=metric_name.replace(SECONDS_SUFFIX, ""),
            median_color="r",
            print_key=print_key,
            include={
                ("S", "U"),
                ("S+R", "U"),
                ("Cut", "U"),
                ("Cut+R", "U"),
                ("MIG", "U"),
                ("MIG+R", "U"),
                ("S+R", "S"),
            },
            prefix=f"incr{incremental}",
        )[0].save(
            f"out/{prefix}2-comparison/SHORT-incr{incremental}-{mn}{sn}-{suite}-{metric}-comparison.pdf"
        )

        # forest_plot(
        #     g,
        #     feature=metric,
        #     title=nice_suite[suite],
        #     feature_label=metric_name.replace(SECONDS_SUFFIX, ""),
        #     median_color="r",
        #     print_key=print_key,
        # )[0].save(
        #     f"out/{prefix}2-comparison/LONG-incr{incremental}-{mn}{sn}-{suite}-{metric}-comparison.pdf"
        # )

        print_key = False

# Comparison of timing information against non-incremental (all suites
# aggregated, so 1 x 2 timing metrics)

for metric, metric_name in metrics:
    if metric not in timing_metrics:
        continue
    forest_plot(
        comparisons.filter(
            pl.col("incremental"),
            ~pl.col("incremental_baseline"),
        ),
        feature=metric,
        title="All suites",
        feature_label=metric_name.replace(SECONDS_SUFFIX, ""),
        median_color="r",
        ablation=True,
        prefix="incablation",
    )[0].save(f"out/Fig3-incablation/{metric}-incablation.pdf")

# % % Make scalability analysis plot


def scalplot(
    df,
    *,
    x,
    y,
    order="order",
):
    fig, ax = plt.subplots(1, 1, figsize=(5, 3))

    for (label, color, marker), g in df.sort(order).group_by(
        "short",
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
    ax.set_ylabel("Duration" + SECONDS_SUFFIX, fontweight="bold")

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
    "out/Fig4-scal/scal.pdf",
    bbox_inches="tight",
)

# %% Size-controlled analysis

size_controlled = (
    comparisons.filter(
        pl.col("provider") == "AlphabeticalRelevant",
        pl.col("provider_baseline") == "AlphabeticalComplete",
        pl.col("incremental"),
    )
    .with_columns(multiplier=pl.col("decisions") / pl.col("decisions_baseline"))
    .select("multiplier", "or_count", "suite")
)

fig, ax = plt.subplots(1, 1, figsize=(4, 3))

suite_styles = {
    "manual": ("#44AA99", "s"),
    "random": ("#117733", "o"),
    "argusReduced": ("#999933", "^"),
    "aesop": ("#882255", "+"),
}


for s in suite_order:
    g = size_controlled.filter(pl.col("suite") == s)
    c, m = suite_styles[s]
    ax.scatter(
        g["or_count"],
        g["multiplier"],
        c=c,
        marker=m,
        label=nice_suite[s],
        alpha=0.3,
        clip_on=False,
        s=10,
    )

rho = size_controlled.select(
    pl.corr("or_count", "multiplier", method="spearman")
).item()

ax.annotate(
    r"Spearman’s $ \rho = " + f"{rho:.2f}$",
    xy=(0.05, 0.05),
    xycoords="axes fraction",
    ha="left",
    va="bottom",
)

ax.spines[["top", "right"]].set_visible(False)

ax.set_xlabel("Size (OR nodes)", fontweight="bold")
ax.set_ylabel("Decision count ratio", fontweight="bold")

ax.set_xlim(0, 220)
ax.set_xticks(np.arange(0, 221, 20))

ax.set_ylim(0, 1)
ax.set_yticks(np.arange(0, 1.01, 0.1))

fig.legend(
    # loc="upper center",
    # bbox_to_anchor=(0.5, 0.05),
    # ncol=len(suite_order),
)

fig.tight_layout()
fig.save(
    "out/Sup3-size-controlled/size-controlled.pdf",
    # bbox_inches="tight",
)

print(r"\newcommand{\SizeControlCorr}{" + f"{rho:.2f}" + r"}")
