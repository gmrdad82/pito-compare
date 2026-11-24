# The data file

pcmp reads one chart's data from a JSON or TOML file. A `.json` file is read as JSON and a `.toml` file as TOML; any other name is read as JSON when its first character is `{`, else as TOML. Both hold the same keys. A byte-order mark at the start is fine.

```toml
title = "PARSE SPEED"
unit = "MEGABYTES PER SECOND"
better = "higher"
subtitle = "SAMPLE DATA, 8-CORE LAPTOP"
date = "2026-10-11"
suffix = " MB/S"
notes = ["* NOT LIKE FOR LIKE: V1 SKIPS THE VALIDATION PASS."]
parties = [{ name = "PARSER V2" }, { name = "PARSER V1" }]

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

## The chart

| Key | Required | What it is |
| --- | --- | --- |
| `title` | yes | The chart's title. |
| `better` | yes | `"higher"` or `"lower"`: which way is better, so which side counts as faster. |
| `parties` | yes | Who is compared, in the order their bars are drawn: one to 12. |
| `rows` or `groups` | one of the two | The rows, with no headings, or groups of rows each with an optional heading. Both together are refused. |
| `unit` | no | What the values measure; the chart adds `HIGHER IS BETTER` or `LOWER IS BETTER`. |
| `subtitle`, `date` | no | The small line under the unit, joined with a comma. `date` is plain text, not checked as a date. |
| `versus` | no | The name of the party the first is compared with: a party other than the first, the second by default. |
| `notes` | no | Lines under the chart, wrapped at spaces to its width. |
| `suffix` | no | Written after each value, such as `" FPS"`. |
| `decimals` | no | Decimals for each value, 0 to 6. By default one below 100 and none from 100 up. |
| `words` | no | The words of the tags and the ratio chart (see [`styling.md`](styling.md)). |

Any other key is refused, with the keys it expected.

## A party

`name` (required) and `colour` (optional, see [`styling.md`](styling.md)). Two names that match ignoring case are refused.

## A group and a row

A group is `name` (optional heading) and `rows` (one or more). A file with `rows` and no groups has one unnamed group, so error paths read the same.

A row has:

- `label` (required) and `mark` (optional, such as `*` or `†`, written after the label for a note to explain).
- `values`: one number per party, in the parties' order, finite, zero or more and at most 1e15.
- `texts` (optional): one string per party, printed in place of the values; the bars still follow `values`.
- `ratio` (optional): the row's ratio in place of the one its values give; finite and above zero.

A row needs `values`, a `ratio` or both. The paired bars and single bars need `values` on every row; the ratio chart needs a `ratio` or two compared values above zero.

## How much text fits

Each text has the room of the 1080 px chart, and one that does not fit is refused, naming its field. With `--width 540` the room is smaller (the title holds 27 characters, the unit line and a row label 41): shorten the text, or draw at 1080.

| Text | Characters |
| --- | --- |
| `title` | 41 |
| `unit`, with the chart's `, HIGHER IS BETTER` after it | 37 |
| `subtitle` and `date` together | 83 |
| a group's `name` | 83 |
| a row's `label`, a space and its `mark` | 55 |
| a party's `name` | 53 |
| one word of a note (a note wraps at spaces) | 55 |

A value's text (the number and the `suffix`, or a row's `texts`) has to leave room for the bar beside it, so keep the suffix short. A chart over 32000 px tall, a few hundred rows, is refused: draw fewer rows or split the file.

## The font

Every text is drawn in a 5x7 pixel font: A to Z (lower case is drawn in capitals), 0 to 9, space and `. , : ; - + / ( ) % = < > ' _ | × * †`. Any other character is refused with the field it is in: write `EUR` for `€`, and keep double quotes, `&`, `#` and `@` out of the text.

## Writing it well

- Capitals, short labels, one unit. Put the machine and the date in `subtitle` and `date`.
- Group rows that belong together; the group heading is drawn above them.
- Mark a row that is not like for like with `mark = "*"` and say why in `notes`.
- Sample or made-up data says so in `notes`.
- Keep the values as measured; the chart rounds only what it shows.
