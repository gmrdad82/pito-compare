# Chart kinds

pcmp draws three kinds, all 1080 px wide, in one look. `--kind all` (the default) draws every one that fits the data; `--kind paired|ratio|bars` draws one.

## Paired bars

Two or more parties (up to 12) on one linear axis from zero, the value beside each bar. On the right of each row, a tag says how many times faster or slower the first party is than the party it is compared with (`versus`, the second party by default). A row whose ratio rounds to 1.0 reads `SAME SPEED`.

Use it when the sizes of the values matter: "parser v2 does 1210 MB/s, v1 does 388".

## The balanced ratio chart

1x in the middle, slower to the left and faster to the right, on a symmetric log scale: 3x slower is as long as 3x faster, so a win and a loss of the same size look the same size. It compares the first party with `versus`, whatever the other parties are. Green is faster and red is slower on this chart whatever the parties' colours are.

Use it when the question is "how many times faster or slower, row by row".

## Single bars

One party, plain bars, nothing to compare: build times per platform, sizes per release. With one party, the plain command draws this.

`--kind bars` on several parties draws one chart per party, named after the party: `<stem>-bars-<party>.png`.

## How a row's ratio reads

- `better = "higher"`: the first party is `first / versus` times faster.
- `better = "lower"`: it is `versus / first` times faster.
- Either way, above 1 means the first party wins.
- A row's own `ratio` replaces the one its values give.
- The ratio chart divides the compared values, so both need to be above zero unless the row carries a `ratio`. The paired bars draw a zero as it is.

## Which chart for what

| The question | Draw |
| --- | --- |
| How fast is each one? | paired bars |
| How many times faster is the new one? | the balanced ratio chart |
| Both, for a README | the plain command: paired and ratio |
| Several tools, judged against one | three or more parties and `versus`, paired bars |
| One tool, many measurements | one party, single bars |
| Rows in different units | rows with a `ratio` and no values, `--kind ratio` |
| One picture per tool | `--kind bars` on several parties |

## Three or more parties

The first party is always the one being judged. A third party can stand in for scale without being in the tags: `versus = "CACHE V1"` names the party the first is compared with.
