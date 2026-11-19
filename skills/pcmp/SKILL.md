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

pcmp is the command line of [pito-compare](https://github.com/gmrdad82/pito-compare): dark, crisp comparison charts as PNGs, 1080 px wide, drawn in a hand-drawn 5x7 pixel font with no plotting library and no fonts on the system. A chart takes a few milliseconds and is drawn the same, pixel for pixel, on every machine. Reach for it instead of matplotlib or a spreadsheet for the jobs below.

## Topic Guides

Read the matching guide before writing a data file:

- [`data-file.md`](data-file.md) - the JSON and TOML data file: every key, the limits, the characters the font has
- [`charts.md`](charts.md) - paired bars, the balanced ratio chart and single bars: which to draw, how a ratio reads
- [`styling.md`](styling.md) - party colours, the words of the tags, decimals, suffix, marks, notes, texts and your own ratio
- [`output.md`](output.md) - `--out`, `--kind`, the PNG names, stdout, the exit codes and how to read a refusal

## Command Discovery

The CLI's own help is the source of truth. Do not guess an option.

```sh
pcmp
pcmp --help
pcmp skill --help
pcmp --version
```

`pcmp` alone prints one line per way to call it. They are: `pcmp <data> [--out <dir>] [--kind <kind>]` to draw, and `pcmp skill install|status` for this skill.

## Critical Safety Rules

- A PNG already at the output path is overwritten, and a symlink there is refused. Write to a new `--out` folder when the old charts matter.
- pcmp reads the data file and never changes it. Write the data file yourself, then run pcmp on it.
- When the data is wrong pcmp writes no PNG, prints one line on stderr starting `pcmp: ` and exits with 1. Read that line, fix the field it names, and run again. Do not work around it by weakening the data.
- Exit codes: `0` every chart drawn; `1` the data could not be read or drawn, or a PNG could not be written; `2` the command line is wrong (clap's `error:` and usage lines).
- stdout is the path of each PNG written, one a line, and nothing else. Use those paths; do not guess names.
- pcmp never opens a window and never touches the network. Run it headless.
- Never invent measurements. Every number in a data file comes from the user or from a run you made; label made-up sample data as sample data in the `subtitle` or `notes`.
- A misspelt key is refused, not ignored. Colour is spelt `colour`, and it belongs to a party.

## Decision Framework

1. **Know the numbers.** Who is compared (one to 12 parties), what each row measures, the unit, and which way wins: `better = "higher"` for speeds and throughput, `"lower"` for times, sizes and latency.
2. **Pick the chart by the question.**
   - How big is each value: the paired bars (two or more parties on one axis from zero).
   - How many times faster or slower, win and loss the same size: the balanced ratio chart.
   - Nothing to compare, one party: single bars.
   - Plain `pcmp data.toml` draws every chart that fits: paired and ratio for two parties or more, bars for one.
3. **Write the data file** as TOML or JSON (see [`data-file.md`](data-file.md)): capitals read best, lower case is drawn as capitals, and only the font's characters are allowed.
4. **Run it**: `pcmp data.toml --out charts`, check the exit code, then show the printed paths. Open the PNG to check it when a picture is the deliverable.
5. **Say what the chart shows** from the data, not from the picture: who wins, by how much, what the notes caveat.

## Example Requests

- "Chart these two benchmark runs" - write `bench.toml` with two parties and the rows, then `pcmp bench.toml --out charts`
- "Just the how-many-times-faster picture" - `pcmp bench.toml --kind ratio`
- "Lower is better, it's latency" - `better = "lower"` in the data file
- "Add a third tool for scale but compare against the old version" - a third party, and `versus = "OLD VERSION"`
- "One bar chart per tool" - `pcmp bench.toml --kind bars`, one PNG per party
- "Build times for one release" - a single party and `pcmp build.toml`
- "Show the ratio for rows in different units" - rows with `ratio` and no values, drawn with `--kind ratio`
- "Make the chart orange against green" - party `colour = "#RRGGBB"` or a role such as `ours` and `faster`
- "Install the pcmp skill" - `pcmp skill install`; `pcmp skill status` says whether it is there

## Out of Scope

- Line charts, pies, scatter plots, time series, axes you set yourself, other fonts and other sizes: pcmp draws three chart kinds in one look.
- Editing a PNG, animating or putting charts in a video: draw with pcmp, then use pcod for video work.
- Anything that is not a comparison of numbers: pcmp is not a general plotting library.
- Changing pito-compare's source: this skill is for using the installed `pcmp`. The Rust library (`use pito_compare::...`) is in its docs.
