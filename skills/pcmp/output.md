# Output

## Where the charts go

```sh
pcmp data.toml                    # PNGs in the current folder
pcmp data.toml --out charts       # in charts/, made if missing
pcmp data.toml --kind ratio       # only the ratio chart
```

Each chart is a PNG, 1080 px wide, written to `<out>/<stem>-<kind>.png`: `<out>` is `--out` (`.` by default) and `<stem>` is the data file's name without its extension. A PNG already there is overwritten; a symlink there is refused, never written through.

| `--kind` | One party | Two or more parties |
| --- | --- | --- |
| `all` (default) | `<stem>-bars.png` | `<stem>-paired.png`, `<stem>-ratio.png` |
| `paired` | `<stem>-paired.png` | `<stem>-paired.png` |
| `ratio` | `<stem>-ratio.png` | `<stem>-ratio.png` |
| `bars` | `<stem>-bars.png` | `<stem>-bars-<party>.png` for each party |

`<party>` is the party's name in lower case with dashes; a name with no letters or digits, or one another party already took, is the party's place instead, from 1.

The same data draws the same chart, pixel for pixel, on every machine.

## stdout, stderr and the exit code

- stdout is the path of each PNG written, one a line, in the order above, and nothing else. Read the paths from it.
- Exit `0`: every chart drawn, stderr empty.
- Exit `1`: one line on stderr, `pcmp: <file>: <what is wrong>`, naming the field (`groups[0].rows[2].values`) or the line and column where the file does not parse. No PNG is written for wrong data.
- Exit `2`: the command line is wrong; clap's `error: ...`, the usage line and `For more information, try '--help'.`

## Reading a refusal

| The line says | Do this |
| --- | --- |
| `values holds -612: values are finite and zero or more` | a value is negative or too large; values are measurements |
| `the chart font has no '€'` | write the character as letters (`EUR`) |
| `unknown field` and the keys it expected | fix the key's spelling: `colour`, on a party |
| `values has 1 values for 2 parties` | one value per party, in the parties' order |
| `is too long` (a title, label, party name or note word) | shorten it, or move detail into `notes` |
| `the value texts are too long to fit beside the bars` | shorten the suffix or the texts |
| `row 2 (FILE SIZE) has no values to draw as bars` | draw `--kind ratio` only |
| `has no ratio: the compared values must be above zero` | give the row a `ratio`, or drop `--kind ratio` |
| `invalid value 'nope' for '--kind <KIND>'` | `--kind` is all, paired, ratio or bars |

`groups[0].rows[0]` is the first row of the first group; a file with `rows` and no groups has them in one unnamed group.

## Checking the picture

When the PNG is the deliverable, open it and read it once: title, the tags' direction, the notes. For a quick check that nothing is off, `file chart.png` reports a 1080 px wide PNG.
