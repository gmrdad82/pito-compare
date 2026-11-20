# Styling

pcmp has one look: its colours, its pixel font, 1080 px wide. What you choose is below. There is no theme, size or font option.

## Party colours

A party's `colour` is `#RRGGBB` or a role. `color` is read too.

| Role | Colour | Default for |
| --- | --- | --- |
| `ours` | orange | the first party |
| `theirs` | blue | the second party |
| `dim` | grey | the third party |
| `text` | white | the fourth party |
| `faster` | green | |
| `slower` | red | |

```toml
parties = [
  { name = "VULKAN", colour = "#C678DD" },
  { name = "OPENGL", colour = "dim" },
]
```

The colour belongs to a party, not the chart. Past the fourth party the defaults start again from orange, so give each party its own colour; `examples/frames.toml` does. The ratio chart keeps green for faster and red for slower. A colour that is neither a hex nor a role is refused with the roles it knows.

## The words of the tags

The tags say `FASTER`, `SLOWER` and `SAME SPEED`. For sizes, scores or anything else, change them:

```toml
words = { better = "SMALLER", worse = "LARGER", same = "SAME SIZE" }
```

`better` is for rows the first party wins and the ratio chart's winning side; `worse` for rows it loses; `same` for rows whose ratio rounds to 1.0. The ratio chart writes them into its captions (`LARGER THAN <VERSUS>`) and its unit line (`TIMES SMALLER OR LARGER, LOG SCALE`), which have to fit, so keep each to a word or two.

## Numbers on the bars

- `suffix = " MB/S"` is written after every value. Include the leading space.
- `decimals = 0` to 6 sets the decimals of every value. By default a value below 100 gets one and one from 100 up gets none.
- `unit` names what is measured; the chart adds `HIGHER IS BETTER` or `LOWER IS BETTER` from `better`.

## Marks and notes

- A row's `mark = "*"` (or `†`) is written after its label.
- `notes = ["* NOT LIKE FOR LIKE: V1 SKIPS THE VALIDATION PASS."]` are lines under the chart, wrapped at spaces to its width. Use one to explain each mark, to say the data is a sample, and to say how it was measured.
- `subtitle` and `date` make the small line under the unit, joined with a comma, such as the machine and the day.

## Your own text and ratio

A row can print its own words and carry its own ratio:

```toml
parties = [{ name = "NEW" }, { name = "OLD" }]
rows = [
  { label = "OPEN WORLD", values = [240, 96], texts = ["240 FPS", "96 FPS"] },
  { label = "FILE SIZE", values = [1, 1], ratio = 4.6 },
]
```

- `texts` prints one string per party beside the bars in place of the values; the bars still follow `values`.
- `ratio` gives the row's ratio when its values do not share a unit. Above 1 means the first party wins whichever way `better` points. A row with a `ratio` and no `values` has no bars, so draw only `--kind ratio`; asking for the paired bars of it is refused.
