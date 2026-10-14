import json
import math
import sys
import time
import tomllib

started = time.perf_counter()

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

imported = time.perf_counter()

BG = "#101319"
TEXT = "#e8ebf0"
DIM = "#8a92a0"
COLOURS = ["#dc6e22", "#4a8ee6", "#8a92a0", "#e8ebf0"]
FASTER = "#4ac97d"
SLOWER = "#eb5858"


def load(path):
    with open(path, "rb") as f:
        data = tomllib.load(f) if path.endswith(".toml") else json.load(f)
    rows = []
    for group in data.get("groups", [{"rows": data.get("rows", [])}]):
        for row in group["rows"]:
            label = row["label"] + row.get("mark", "")
            if group.get("name"):
                label = group["name"] + "\n" + label
            rows.append((label, row["values"]))
    return data, rows


def figure(data, rows, height):
    fig, ax = plt.subplots(figsize=(10.8, height), dpi=100)
    fig.patch.set_facecolor(BG)
    ax.set_facecolor(BG)
    ax.tick_params(colors=TEXT)
    for side in ax.spines.values():
        side.set_color(DIM)
    better = "HIGHER IS BETTER" if data["better"] == "higher" else "LOWER IS BETTER"
    ax.set_title(f"{data['title']}\n{data.get('unit', '')}, {better}", color=TEXT, loc="left")
    ax.set_yticks(range(len(rows)), [r[0] for r in rows], color=TEXT)
    ax.invert_yaxis()
    fig.text(0.01, 0.01, "\n".join(data.get("notes", [])), color=DIM, fontsize=8)
    return fig, ax


def paired(data, rows):
    fig, ax = figure(data, rows, 1.2 * len(rows) + 2)
    n = len(data["parties"])
    h = 0.8 / n
    for p, party in enumerate(data["parties"]):
        ys = [i + (p - (n - 1) / 2) * h for i in range(len(rows))]
        vals = [r[1][p] for r in rows]
        bars = ax.barh(ys, vals, height=h, color=COLOURS[p % 4], label=party["name"])
        ax.bar_label(bars, [f"{v:g}{data.get('suffix', '')}" for v in vals], color=TEXT, padding=4)
    ax.legend(facecolor=BG, labelcolor=TEXT)
    return fig


def ratio(data, rows):
    fig, ax = figure(data, rows, 0.6 * len(rows) + 2)
    sign = 1 if data["better"] == "higher" else -1
    logs = [sign * math.log2(r[1][0] / r[1][1]) for r in rows]
    bars = ax.barh(range(len(rows)), logs, color=[FASTER if l >= 0 else SLOWER for l in logs])
    ax.bar_label(bars, [f"{2 ** abs(l):.1f}×" for l in logs], color=TEXT, padding=4)
    top = max(abs(l) for l in logs) * 1.2
    ax.set_xlim(-top, top)
    ax.axvline(0, color=TEXT)
    return fig


def bars(data, rows):
    fig, ax = figure(data, rows, 0.8 * len(rows) + 2)
    vals = [r[1][0] for r in rows]
    drawn = ax.barh(range(len(rows)), vals, color=COLOURS[0])
    ax.bar_label(drawn, [f"{v:g}{data.get('suffix', '')}" for v in vals], color=TEXT, padding=4)
    return fig


path, kind, out = sys.argv[1:4]
data, rows = load(path)
fig = {"paired": paired, "ratio": ratio, "bars": bars}[kind](data, rows)
fig.savefig(out, facecolor=BG)
done = time.perf_counter()
print(f"{(imported - started) * 1000:.1f} {(done - imported) * 1000:.1f}")
