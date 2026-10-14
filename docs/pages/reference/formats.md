---
title: Formats
description: "The data file pcmp reads, every field and its limits, and what pcmp writes: PNG names, stdout and stderr."
sidebar:
  order: 2
---

## The data file

pcmp reads one chart's data from a JSON or TOML file. A `.json` file is read as JSON and a `.toml` file as TOML; any other name is read as JSON when its first character is `{`, else as TOML. Both hold the same keys.

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

```json
{
  "title": "PARSE SPEED",
  "better": "higher",
  "parties": [{ "name": "PARSER V2" }, { "name": "PARSER V1" }],
  "rows": [{ "label": "CONFIG FILES", "values": [612, 240] }]
}
```

### The chart

| Key | Required | What it is |
|---|---|---|
| `title` | yes | The chart's title, at most 41 characters. |
| `unit` | no | What the values measure; the chart adds `HIGHER IS BETTER` or `LOWER IS BETTER`. |
| `better` | yes | `"higher"` or `"lower"`: which way is better, so which side counts as faster. |
| `subtitle`, `date` | no | The small line under the unit, such as the machine and the date, joined with a comma. |
| `parties` | yes | Who is compared, in the order their bars are drawn: one or more. |
| `versus` | no | The name of the party the first one is compared with; the second party by default. |
| `rows` | one of the two | The rows, with no group headings. |
| `groups` | one of the two | Groups of rows, each with an optional heading. |
| `notes` | no | Lines under the chart, wrapped at spaces to its width. |
| `suffix` | no | Written after each value, such as `" FPS"` or `"×"`. |
| `decimals` | no | Decimals for each value, 0 to 6. By default one below 100 and none from 100 up. |
| `words` | no | The words of the tags and the ratio chart. |

Any other key is refused, so a misspelt key never goes unnoticed.

### A party

| Key | Required | What it is |
|---|---|---|
| `name` | yes | The party's name in the legend; it has to fit the legend's width. |
| `colour` | no | `#RRGGBB`, or a role: `ours` (orange), `theirs` (blue), `dim` (grey), `text` (white), `faster` (green) or `slower` (red). The first four parties default to `ours`, `theirs`, `dim` and `text`. |

### A group

| Key | Required | What it is |
|---|---|---|
| `name` | no | The heading drawn above its rows. |
| `rows` | yes | One row or more. |

### A row

| Key | Required | What it is |
|---|---|---|
| `label` | yes | The row's label. |
| `values` | see below | One number per party, in the parties' order: finite and zero or more. |
| `mark` | no | Written after the label, such as `*` or `†`, for a note to explain. |
| `texts` | no | One string per party, printed in place of the values; the bars still follow `values`. |
| `ratio` | see below | The row's ratio in place of the one its values give: finite and above zero. Above 1 means the first party wins. |

A row needs `values`, a `ratio` or both. The paired and single bars need `values`; the ratio chart needs a `ratio` or two compared values above zero.

### Words

| Key | Default | Where it shows |
|---|---|---|
| `better` | `FASTER` | the tags of rows the first party wins, and the ratio chart's winning side |
| `worse` | `SLOWER` | the tags of rows the first party loses, and the ratio chart's losing side |
| `same` | `SAME SPEED` | the tags of rows whose ratio rounds to 1.0 |

### How a row's ratio reads

With `better = "higher"`, the first party is `first / versus` times faster; with `"lower"`, `versus / first`. A row whose ratio rounds to 1.0 is tagged with `words.same`.

### The font

Every text in the chart is drawn in a 5x7 pixel font: A to Z (lower case is drawn in capitals), 0 to 9, space and `. , : ; - + / ( ) % = < > ' _ | × * †`. Any other character is refused with the field it is in. Text too long for its line is refused the same way; notes wrap at spaces, so only a single word longer than a line is refused.

## What pcmp writes

### The PNGs

Each chart is a PNG, 1080 px wide, written to `<out>/<stem>-<kind>.png`, where `<out>` is `--out` (the current folder by default, made if missing) and `<stem>` is the data file's name without its extension. A PNG already there is overwritten.

| `--kind` | One party | Two or more parties |
|---|---|---|
| `all` (default) | `<stem>-bars.png` | `<stem>-paired.png`, `<stem>-ratio.png` |
| `paired` | `<stem>-paired.png` | `<stem>-paired.png` |
| `ratio` | `<stem>-ratio.png` | `<stem>-ratio.png` |
| `bars` | `<stem>-bars.png` | `<stem>-bars-<party>.png` for each party |

`<party>` is the party's name in lower case with dashes; a name with no letters or digits, or one another party already took, is the party's place instead, from 1.

The same data draws the same chart, pixel for pixel, on every machine.

### stdout

The path of each PNG written, one a line, in the order above, and nothing else.

### stderr and the exit code

| Code | stderr |
|---|---|
| `0` | nothing |
| `1` | one line, `pcmp: <file>: <what is wrong>`, naming the field (`groups[0].rows[2].values`) or the line and column where the file doesn't parse; when the data is wrong, no PNG is written |
| `2` | clap's answer to a command line it can't parse: `error: …`, the usage line, `For more information, try '--help'.` |

[Read a refusal](../../guides/refusals/) shows real examples of each.
