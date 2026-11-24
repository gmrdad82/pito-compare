# Output

## Where the charts go

```sh
pcmp examples/speed.toml
pcmp examples/speed.toml --out charts
pcmp examples/speed.toml -k ratio -o charts
```

The first writes `speed-paired.png` and `speed-ratio.png` in the working folder, the second the same in `charts/` (made if missing), the third only `charts/speed-ratio.png`. `--out` is `-o` and `--kind` is `-k`.

`--width <PX>` (`-w`) is how wide each PNG is: `1080` (the default), `540` or `405`, for a small picture such as one in a chat notification.

```sh
pcmp examples/speed.toml --width 540 --out charts
pcmp examples/speed.toml --width 405 --out charts
```

Another width is refused with one line (`pcmp: --width: the width 700 cannot be drawn well: the widths are 405, 540 and 1080`) and exit code 2, and nothing is drawn. At 540 and 405 the texts have less room, and one that does not fit is refused, naming its field, like at 1080. At 405 the title is drawn at size 2, or at size 1 when it does not fit at 2, and every other text at size 1.

Each chart is a PNG, 1080 px wide by default (`--width`), written to `<out>/<stem>-<kind>.png`: `<out>` is `--out` (`.` by default) and `<stem>` is the data file's name without its extension. A PNG already there is replaced; a symlink there is refused, never written through.

| `--kind` | One party | Two or more parties |
| --- | --- | --- |
| `all` (default) | `<stem>-bars.png` | `<stem>-paired.png`, `<stem>-ratio.png` |
| `paired` | refused | `<stem>-paired.png` |
| `ratio` | refused unless every row has a `ratio` | `<stem>-ratio.png` |
| `bars` | `<stem>-bars.png` | `<stem>-bars-<party>.png` for each party |

`<party>` is the party's name in lower case with dashes (`CACHE V2` is `cache-v2`); a name with no letters or digits, or one another party already took, is the party's place instead, from 1.

## stdout, stderr and the exit code

- stdout is the path of each PNG written, one a line, in the order above, and nothing else. With the default `--out` they are bare names (`speed-paired.png`), otherwise `<out>/<name>`. Read the paths from it.
- Exit `0`: every chart drawn, stderr empty.
- Exit `1`: one line on stderr, `pcmp: <file>: <what is wrong>`, naming the field (`groups[0].rows[2].values`) or the line and column where the file does not parse. A file that cannot be read is `pcmp: cannot read <file>: <why>`. Nothing is printed on stdout, but a chart drawn before the refusal is on disk; check `--out` after a refusal.
- Exit `2`: the command line is wrong; clap's `error: ...`, the usage line and `For more information, try '--help'.`

`groups[0].rows[0]` is the first row of the first group; a file with `rows` and no groups has them in one unnamed group.

## Reading a refusal

| The line says | Do this |
| --- | --- |
| `values holds -612: values are finite and zero or more` | a value is negative or over 1e15; values are measurements |
| `the chart font has no '€'` | write the character as letters (`EUR`) |
| `unknown field` | a key is misspelt; the line lists the keys it expected |
| `missing field` | add the key it names: `title`, `better` or `parties` |
| `values has 1 values for 2 parties` | one value per party, in the parties' order |
| `title is too long: at most 41 characters fit` | shorten the text, or move detail into `notes`; every text has a limit (see [`data-file.md`](data-file.md)) |
| `has a word longer than a line` | a word of a note is wider than the chart; break it up |
| `the value texts are too long to fit beside the bars` | shorten the suffix or the texts |
| `row 1 (FILE SIZE) has no values to draw as bars` | the row has only a `ratio`; draw `--kind ratio` only |
| `row 1 (R) has no ratio: the compared values must be above zero` | give the row a `ratio`, or draw only `--kind paired` |
| `the paired chart needs two parties or more` | one party draws as `--kind bars` |
| `the ratio chart needs two parties, or a ratio on every row` | one party draws as `--kind bars` |
| `give rows or groups, not both` | use one of the two |
| `parties[1].name repeats parties[0]` | party names differ, ignoring case |
| `parties has 13 parties: a chart draws at most 12` | split the parties across charts |
| `versus names "C", which is no party` | `versus` is a party's name |
| `versus must name a party other than the first` | the first party is the one compared; name another |
| `the ratio chart's unit line is too long` | the `words` are too long; a word or two each |
| `parties[0].colour is "orange": use #RRGGBB or a role` | write `#RRGGBB` or a role (see [`styling.md`](styling.md)) |
| `cannot read nope.toml` | the file is missing or unreadable; check the path |
| `invalid value 'nope' for '--kind <KIND>'` | exit 2; `--kind` is `all`, `paired`, `ratio` or `bars` |

## Checking the picture

When the PNG is the deliverable, open it and read it once: title, the tags' direction, the notes. For a quick check that nothing is off, `file charts/speed-paired.png` reports a 1080 px wide PNG (540 or 405 with `--width`).
