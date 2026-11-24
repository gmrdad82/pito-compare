# Chart kinds

pcmp draws three kinds, all 1080 px wide (540 or 405 with `--width`), in one look. `--kind all` (the default) draws every one that fits the data; `--kind paired`, `--kind ratio` and `--kind bars` draw one.

| `--kind` | Draws | Needs |
| --- | --- | --- |
| `all` | one party: the single bars; two or more: the paired bars and the ratio chart | what those need |
| `paired` | every party on one axis from zero, with a tag per row | two or more parties, `values` on every row |
| `ratio` | the balanced ratio chart | two or more parties, and on every row either two compared values above zero or a `ratio`; one party works only when every row has a `ratio` |
| `bars` | plain bars of one party | `values` on every row; several parties draw one chart each |

## Paired bars

Two or more parties (up to 12) on one linear axis from zero, the value beside each bar. On the right of each row a tag says how many times faster or slower the first party is than the party it is compared with (`versus`, the second party by default). A row whose ratio rounds to 1.0 reads `SAME SPEED`. A zero value is drawn as it is.

Use it when the sizes of the values matter: "parser v2 does 1210 MB/s, v1 does 388".

## The balanced ratio chart

1x in the middle, slower to the left and faster to the right, on a symmetric log scale: 3x slower is as long as 3x faster, so a win and a loss of the same size look the same size. It compares the first party with `versus`, whatever the other parties are. Green is faster and red is slower here whatever the parties' colours are.

Use it when the question is "how many times faster or slower, row by row", or when the values span orders of magnitude (0.4 against 160).

## Single bars

One party, plain bars, nothing to compare: build times per platform, sizes per release. With one party the plain command draws this, and `--kind paired` is refused.

`--kind bars` on several parties draws one chart per party, named after the party: `<stem>-bars-<party>.png`.

## How a row's ratio reads

- `better = "higher"`: the first party is `first / versus` times faster.
- `better = "lower"`: it is `versus / first` times faster.
- Either way, above 1 means the first party wins.
- A row's own `ratio` replaces the one its values give.
- The ratio chart divides the compared values, so both need to be above zero unless the row carries a `ratio`. A row with a zero therefore draws as paired bars but not on the ratio chart; `--kind all` writes the paired chart and then refuses the ratio chart.

## Rows in different units

Rows that cannot share an axis carry their own ratio and no values; only the ratio chart can draw them:

```toml ratio
title = "WHAT V2 CHANGED"
better = "higher"
parties = [{ name = "V2" }, { name = "V1" }]
rows = [
  { label = "PARSE SPEED", ratio = 4.6 },
  { label = "MEMORY USE", ratio = 1.8 },
  { label = "START UP", ratio = 0.7 },
]
```

Run it with `--kind ratio`. Plain `pcmp` on it is refused at the paired chart, and writes nothing.

## Three or more parties

The first party is always the one being judged. A third party can stand in for scale without being in the tags: `versus = "CACHE V1"` names the party the first is compared with, and has to be a party other than the first. Past the fourth party give each party its own `colour` (see [`styling.md`](styling.md)).
