---
name: pcmp
description: >
  Use pcmp, pito-compare's command line, to draw a comparison or benchmark chart as a PNG from a
  JSON or TOML data file: paired bars of two or more parties on one axis, the balanced ratio chart
  (how many times faster or slower, 1x in the middle), or single bars. Use it for a README or
  release-notes picture of benchmark results, a before and after, a new version against an old one,
  or one tool against another. Triggers: pcmp, pito-compare, benchmark chart, comparison chart,
  bar chart PNG, paired bars, ratio chart, faster or slower than, versus chart, chart for a README,
  matplotlib replacement, chart from JSON or TOML.
---

# pcmp Skill

pcmp is the command line of [pito-compare](https://github.com/gmrdad82/pito-compare). It reads one JSON or TOML data file and draws dark comparison charts as PNGs, 1080 px wide, in its own 5x7 pixel font: no plotting library, no fonts on the system, a few milliseconds a chart, and the same pixels on every machine. Reach for it instead of matplotlib or a spreadsheet for a README or release-notes picture of results.

## Topic Guides

Read the matching guide before writing a data file:

- [`data-file.md`](data-file.md) - the JSON and TOML data file: every key, a fixed axis with `max`, how much text fits, the characters the font has
- [`charts.md`](charts.md) - paired bars, the balanced ratio chart and single bars: which to draw, how a ratio reads
- [`styling.md`](styling.md) - party colours, the words of the tags, decimals, suffix, marks, notes, texts and your own ratio
- [`output.md`](output.md) - `--out`, `--kind`, the PNG names, stdout, the exit codes and how to read a refusal

## Command Discovery

The CLI's own help is the source of truth. Do not guess an option.

```sh
pcmp
pcmp --help
pcmp skill --help
pcmp skill install --help
pcmp skill status --help
pcmp --version
```

A bare `pcmp` prints its version and one line for each way to call it, and exits 0. `pcmp --help` (`-h`) has every option and a sample data file, and `pcmp --version` (`-V`) the version. The ways to call it are:

- `pcmp <data> [--out <dir>] [--kind <kind>] [--width <px>]` draws the charts of a data file. `--out` (`-o`) is the folder, `.` by default; `--kind` (`-k`) is `all`, `paired`, `ratio` or `bars`, `all` by default; `--width` (`-w`) is `1080` by default, or `540` or `405` for a small picture.
- `pcmp skill install` and `pcmp skill status` put this skill in place and check it.

The examples here use the data files in pito-compare's `examples/`. In another project, run the same commands on your own file.

## Critical Safety Rules

- pcmp reads the data file and never changes it. It writes only `<out>/<stem>-<kind>.png` and makes `--out` when it is missing. A PNG already at that path is replaced and a symlink there is refused, so write to a new `--out` folder when the old charts matter.
- Wrong data exits 1 with one stderr line, `pcmp: <file>: <what is wrong>`, naming the field. Read it, fix that field and run again; do not weaken the data to get past it. A chart drawn before the one that failed stays on disk but is not printed: with a zero value and no `ratio`, `--kind all` writes the paired chart and then refuses the ratio chart.
- Exit codes: `0` every chart drawn; `1` the data could not be read or drawn, or a PNG could not be written; `2` the command line is wrong (clap's `error:` and usage lines).
- stdout is the path of each PNG written, one a line, and nothing else. Use those paths and do not guess names.
- A misspelt key is refused with the keys it expected. `colour` belongs to a party, not to the chart.
- Every number in a data file comes from the user or from a run you made. Say in `subtitle` or `notes` when it is sample data.
- `pcmp skill install` writes the skill as real files into `~/.claude/skills/pcmp` and `~/.agents/skills/pcmp`, and into Codex's and Pi's skill folders when they exist. An older link there is replaced by files. Run it when asked.

## Decision Framework

1. **Know the numbers.** Who is compared (one to 12 parties), what each row measures, the unit, and which way wins: `better = "higher"` for speeds and throughput, `"lower"` for times, sizes and latency.
2. **Pick the chart by the question** (see [`charts.md`](charts.md)): how big is each value, paired bars; how many times faster or slower, the balanced ratio chart; one party, single bars. The plain command draws every chart that fits: paired and ratio for two parties or more, bars for one.
3. **Write the data file** as TOML or JSON (see [`data-file.md`](data-file.md)): capitals read best, lower case is drawn as capitals, and only the font's characters are allowed.
4. **Run it** on your file, as `pcmp examples/speed.toml --out charts` does on the example, check the exit code, then use the printed paths.
5. **Say what the chart shows** from the data, not from the picture: who wins, by how much, what the notes caveat. Open the PNG once when a picture is the deliverable.

## Example Requests

| Request | Run |
| --- | --- |
| "Chart these two benchmark runs for the README" | `pcmp examples/speed.toml --out charts` writes `charts/speed-paired.png` and `charts/speed-ratio.png` |
| "Just the how-many-times-faster picture" | `pcmp examples/speed.toml --kind ratio --out charts` writes `charts/speed-ratio.png` |
| "Latency, lower is better, a third tool for scale" | `pcmp examples/latency.json --out charts` writes `charts/latency-paired.png` and `charts/latency-ratio.png`; its `versus = "CACHE V1"` names the party the first one is compared with |
| "One picture per tool" | `pcmp examples/latency.json --kind bars --out charts` writes `charts/latency-bars-cache-v2.png`, `charts/latency-bars-cache-v1.png` and `charts/latency-bars-no-cache.png` |
| "Build times for one release" | `pcmp examples/build.toml` draws one party as `build-bars.png` in the working folder |
| "Four graphics APIs, each in its own colour" | `pcmp examples/frames.toml --out charts` writes `charts/frames-paired.png` and `charts/frames-ratio.png` |
| "One row is 400 times apart, another nearly equal" | `pcmp examples/spread.toml --kind ratio --out charts` writes `charts/spread-ratio.png`, whose log scale keeps both ends readable |
| "Rows in different units" | rows with a `ratio` and no `values`, drawn with `--kind ratio` (see [`charts.md`](charts.md)) |
| "Install the pcmp skill" | `pcmp skill install`; `pcmp skill status` says whether it is there and matches this pcmp |

## Out of Scope

- Line charts, pies, scatter plots, time series, axes you set yourself, other fonts and other sizes: pcmp draws three chart kinds in one look.
- Editing a PNG, or putting charts in a video: draw with pcmp, then use pcod for video work.
- Changing pcmp itself: this skill is for the installed `pcmp`. The Rust library (`pito_compare::Chart`) is in its docs.
