<p align="center"><img src="docs/images/draw.gif" width="800" alt="A terminal: cat demo.toml shows a small TOML file with a title, a unit, higher is better, two parties and three rows; pcmp demo.toml --out charts prints charts/demo-paired.png and charts/demo-ratio.png; pcmp demo.toml --kind bars --out charts prints one PNG per party; file shows a 1080 px wide PNG."></p>

# pito-compare

[![CI](https://github.com/gmrdad82/pito-compare/actions/workflows/ci.yml/badge.svg)](https://github.com/gmrdad82/pito-compare/actions/workflows/ci.yml) [![Version](https://img.shields.io/github/v/tag/gmrdad82/pito-compare)](https://github.com/gmrdad82/pito-compare/tags)

pito-compare draws comparison charts as PNGs, part of [PITO](https://pitomd.com). You write the numbers in a small JSON or TOML file, and `pcmp` draws them as dark, crisp charts in a hand-drawn pixel font: two or more parties side by side on one axis, how many times faster or slower one is than another, or plain bars.

## What it is

Benchmark results read best as a picture, and a picture that looks the same every time. pito-compare is the chart drawer of [pito-codecs](https://github.com/gmrdad82/pito-codecs)' benchmarks made into a tool ([INSPIRATION.md](INSPIRATION.md)), with its exact look: the colours, the 5x7 pixel font, 1080 px wide.

- **Paired bars:** every party's value on one shared linear axis from zero, the value printed beside each bar, and per row how many times faster or slower the first party is than the one it is compared with.
- **Balanced ratios:** 1× in the middle, slower to the left and faster to the right on a symmetric log scale, so 3× slower is as long as 3× faster.
- **Single bars:** one party's values, when there is nothing to compare.
- **Groups and notes:** rows fall under headings, a row can carry a mark such as `*` or `†`, and notes under the chart explain them.
- **No plotting library:** the canvas, the font and the PNG writer are a few hundred lines of Rust, so a chart is drawn the same on every machine.

pito-compare is also a Rust library (`use pito_compare::…`) that draws the same charts from data you build in code.

![Paired bars: parse speed of parser v2 against parser v1 in two groups, with 2.5× faster, same speed and 3.2× slower tags](docs/images/speed-paired.png)

![The balanced ratio chart of the same data: green bars to the right of 1× for faster rows, red to the left for slower ones](docs/images/speed-ratio.png)

![Paired bars with three parties, lower is better: request latency of cache v2, cache v1 and no cache](docs/images/latency-paired.png)

![Single bars: cold build time per platform for one party](docs/images/build-bars.png)

These charts are drawn by pcmp from the sample data in [examples/](examples/); the numbers are made up.

## Install

pcmp runs on Linux (x86_64), Windows (x86_64) and macOS (Apple Silicon). Each [release](https://github.com/gmrdad82/pito-compare/releases/latest) carries a binary for all three, with a `SHA256SUMS` file.

**Linux:**

```
v=$(basename "$(curl -fsSLo /dev/null -w '%{url_effective}' https://github.com/gmrdad82/pito-compare/releases/latest)")
mkdir -p ~/.local/bin && curl -fsSL "https://github.com/gmrdad82/pito-compare/releases/download/$v/pcmp-$v-linux-x86_64.tar.gz" | tar -xz -C ~/.local/bin pcmp
```

**macOS:**

```
v=$(basename "$(curl -fsSLo /dev/null -w '%{url_effective}' https://github.com/gmrdad82/pito-compare/releases/latest)")
mkdir -p ~/.local/bin && curl -fsSL "https://github.com/gmrdad82/pito-compare/releases/download/$v/pcmp-$v-macos-arm64.tar.gz" | tar -xz -C ~/.local/bin pcmp
```

On Linux and macOS, `~/.local/bin` needs to be on your `PATH`.

**Windows** (PowerShell):

```
$v = (Invoke-RestMethod https://api.github.com/repos/gmrdad82/pito-compare/releases/latest).tag_name
Invoke-WebRequest "https://github.com/gmrdad82/pito-compare/releases/download/$v/pcmp-$v-windows-x86_64.zip" -OutFile "$env:TEMP\pcmp.zip"
Expand-Archive "$env:TEMP\pcmp.zip" "$env:LOCALAPPDATA\pcmp" -Force
[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ";$env:LOCALAPPDATA\pcmp", 'User')
```

Open a new terminal afterwards, so it sees the new `PATH`.

**From source**, on any of the three, with Rust 1.97 or newer:

```
cargo install --locked --git https://github.com/gmrdad82/pito-compare --tag vX.Y.Z pito-compare
```

Replace `vX.Y.Z` with a [tag](https://github.com/gmrdad82/pito-compare/tags). pcmp needs nothing else to run.

## Use

```
pcmp <DATA> [--out DIR] [--kind all|paired|ratio|bars]
```

- `DATA` is a `.json` or `.toml` file (any other name is read as JSON when it starts with `{`, else as TOML).
- `--out` is the folder the PNGs go to, made if missing; the current folder by default.
- `--kind all` (the default) draws single bars for one party, and the paired and ratio charts for two or more. `paired`, `ratio` and `bars` draw one kind; `bars` with several parties draws one chart per party.

Each chart is written to `DIR/<the data file's name>-<kind>.png`, overwriting one already there, and its path printed on its own line: `speed-paired.png`, `speed-ratio.png`, `build-bars.png`, or `speed-bars-parser-v2.png` per party.

### Exit codes

- `0`: every chart was drawn.
- `1`: the data file couldn't be read or drawn, or a PNG couldn't be written, with one line on stderr starting `pcmp: ` that names the file and, where it can, the field, line and column.
- `2`: the command line is wrong. pcmp answers the way clap-based tools do: `error: …`, the usage line and `For more information, try '--help'.`

## The data format

```toml
title = "PARSE SPEED"
unit = "MEGABYTES PER SECOND"
better = "higher"
subtitle = "SAMPLE DATA, 8-CORE LAPTOP"
date = "2026-10-11"
suffix = " MB/S"
notes = ["* NOT LIKE FOR LIKE: V1 SKIPS THE VALIDATION PASS."]

[[parties]]
name = "PARSER V2"

[[parties]]
name = "PARSER V1"

[[groups]]
name = "SMALL FILES"
rows = [
  { label = "CONFIG FILES", values = [612, 240] },
  { label = "DEEP NESTING", values = [130, 410], mark = "*" },
]
```

The same in JSON is an object with the same keys: `{"title": "PARSE SPEED", "better": "higher", "parties": [{"name": "PARSER V2"}, …], …}`. [examples/](examples/) has a TOML and a JSON file of each shape.

| Key | Required | What it is |
|---|---|---|
| `title` | yes | The chart's title, at most 41 characters. |
| `unit` | no | What the values measure; the chart adds "HIGHER IS BETTER" or "LOWER IS BETTER". |
| `better` | yes | `"higher"` or `"lower"`: which way is better, so which side counts as faster. |
| `subtitle`, `date` | no | The small line under the unit, such as the machine and the date, joined with a comma. |
| `parties` | yes | Who is compared, in the order their bars are drawn. Each has a `name` and an optional `colour`. |
| `versus` | no | The name of the party the first one is compared with; the second party by default. |
| `rows` | one of the two | The rows, with no group headings. |
| `groups` | one of the two | Groups of rows, each with an optional `name` drawn as a heading and its `rows`. |
| `notes` | no | Lines under the chart, wrapped to its width. |
| `suffix` | no | Written after each value, such as `" FPS"` or `"×"`. |
| `decimals` | no | Decimals for each value, 0 to 6. By default one below 100 and none from 100 up. |
| `words` | no | `{ better = "FASTER", worse = "SLOWER", same = "SAME SPEED" }`, the words of the tags and the ratio chart. |

Each row has a `label`, its `values` (one per party, zero or more) and optionally a `mark` written after the label, `texts` to print in place of the values (one per party), and a `ratio` to use in place of the one the values give.

A party's `colour` is `#RRGGBB` or one of the chart's roles: `ours` (orange, the first party's default), `theirs` (blue, the second's), `dim` (grey, the third's), `text` (white, the fourth's), `faster` (green) and `slower` (red).

How the comparison reads: with `better = "higher"`, the first party is `first / versus` times faster; with `"lower"`, `versus / first`. A row whose ratio rounds to 1.0 says "SAME SPEED". The ratio chart needs both compared values above zero, or a `ratio`.

The chart font draws A to Z (lower case is drawn in capitals), 0 to 9, space and `. , : ; - + / ( ) % = < > ' _ | × * †`. Any other character, an unknown key, a value that is negative or not a number, or text too long for its line is refused with the field it is in, so a chart never comes out wrong.

### The library

```toml
[dependencies]
pito-compare = { git = "https://github.com/gmrdad82/pito-compare", tag = "vX.Y.Z", default-features = false }
```

```rust
use pito_compare::{Better, Chart, Group, Kind, Row};

let mut chart = Chart::new("DECODE SPEED", "FRAMES PER SECOND", Better::Higher, &["OURS", "THEIRS"]);
chart.suffix = " FPS".into();
chart.groups.push(Group { name: "CPU".into(), rows: vec![Row::new("H.264 1080P", &[412.0, 298.0])] });
for (kind, canvas) in pito_compare::render(&chart, Kind::All)? {
    canvas.save(format!("decode-{kind}.png").as_ref())?;
}
```

`Chart::load` and `Chart::parse` read the data format above; `paired`, `ratios` and `bars` draw one chart each and return a `Canvas`, which `png()` encodes and `save` writes. `default-features = false` leaves out the command line and its clap.

## Contributing

Issues are welcome at [github.com/gmrdad82/pito-compare/issues](https://github.com/gmrdad82/pito-compare/issues): a chart that comes out wrong, with the data file that draws it, or an idea for a chart pcmp should draw. Pull requests aren't taken. Please read the [code of conduct](CODE_OF_CONDUCT.md) first, and report a security issue privately as [SECURITY.md](SECURITY.md) says.

## Build and test

```
cargo build --release
bin/gate
```

`bin/gate` runs what CI runs on Linux, Windows and macOS: ShellCheck where it is installed, rustfmt, clippy with warnings as errors, and the tests, which include drawing the examples and comparing them pixel for pixel with the README's images. `bin/images` draws those again after a change to the look. CI runs the gate on all three on every push and nightly, and builds the Linux release binary; a tag builds the three release binaries. On Windows it runs in Git Bash. The README's clip is a prec shot in `render/terminal.toml`, made a GIF by pconv ([NOTICE](NOTICE)).

## Licence

The code is MIT licensed ([LICENSE](LICENSE)), © Catalin Ilinca. The pito-compare, pcmp and PITO names and their logos are © Catalin Ilinca, all rights reserved, and are not covered by the MIT licence ([TRADEMARKS.md](TRADEMARKS.md)).
