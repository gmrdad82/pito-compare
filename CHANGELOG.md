# Changelog

## v1.4.0 — 2026-11-24

### Added

- `--width <PX>` (`-w`): `1080`, the default, draws exactly what it drew before; `540` draws every chart kind half as wide, with the pixel font at whole-number sizes so every text stays sharp, and refuses a text that does not fit that width, naming its field. Any other width is refused with one line naming the two allowed ones, exit code 2. The library has `render_at`, `render_each_at`, `paired_at`, `ratios_at` and `bars_at`.

## v1.3.3 — 2026-11-20

### Changed

- The docs bundle's skill page shows the `pcmp skill status` paths with `~`, so no home path is in the bundle; a test fails when any docs file holds a home path.

## v1.3.2 — 2026-11-20

### Changed

- `pcmp skill install` writes the skill as real files into `~/.claude/skills/pcmp` and `~/.agents/skills/pcmp`, and into Codex's (`~/.codex/skills/pcmp`) and Pi's (`~/.pi/agent/skills/pcmp`) folders when those tools are there, like every other tool's skill install. An older link to the store folder is replaced by files, and the store is no longer written. `pcmp skill status` says whether each folder is installed and matches this pcmp.

## v1.3.1 — 2026-11-20

### Changed

- The pcmp skill matches the command line again. It is made for pcmp: every command and flag it names is in `pcmp --help` (the short forms `-o`, `-k`, `-h` and `-V` too), every command it shows runs against `examples/`, every data file in it draws and every refusal it reads is one pcmp prints. It says what pcmp does that an agent would trip on: `--kind paired` is refused for one party, a chart drawn before a refusal stays on disk and is not printed, a zero value draws as bars but not on the ratio chart, and how many characters each text holds. `tests/skill_drift.rs` keeps it so.

## v1.3.0 — 2026-11-19

### Added

- pcmp comes with a Claude skill: a `SKILL.md` with its description and triggers, safety rules, command discovery through `pcmp --help`, a decision framework, example requests and what is out of scope, and topic guides on the JSON and TOML data file, the chart kinds, styling and output. It is built into the binary.
- `pcmp skill install` puts the skill in `~/.local/opt/pcmp/skills/pcmp` and links it into `~/.claude/skills/pcmp` and `~/.agents/skills/pcmp` (copied into both on Windows), replacing an older copy and removing files a new version drops; a `pcmp` folder or link there that pcmp did not place is left alone, with one line saying so. `pcmp skill status` says whether each folder is installed and matches this pcmp. A bare `pcmp` lists both.
- `bin/install` builds pcmp from a clone, puts it in `~/.local/opt/pcmp`, links it from `~/.local/bin` and installs the skill, and does the same again on every update.

### Changed

- The install lines in the README and the docs run `pcmp skill install` after unpacking, and running them again updates the binary and the skill together. The README and the docs say pcmp comes with a Claude skill and where it lands; the docs have a page for it, and the command line page lists the `skill` command.

## v1.2.0 — 2026-11-19

### Changed

- A bare `pcmp` prints a short summary and exits with 0, as the other PITO MD tools do: its version and what it does, then each way to call it with a line on what it does. `pcmp --help` keeps the full help, and a wrong command line still answers the way clap does, with exit code 2.

## v1.1.3 — 2026-11-18

### Changed

- The macOS `pcmp` archive holds a `pcmp` signed with the Developer ID and the hardened runtime and notarized by Apple.
- PNGs are written through pito-codecs 5.1.0 instead of the outside `png` crate; every file decodes to the same samples.

## v1.1.2 — 2026-10-22

### Added

- Data files saved with a byte-order mark (as some Windows editors save JSON) now load.

### Changed

- Two parties whose names match, ignoring case, are refused with one line naming both, instead of a misleading versus error.
- A chart built in code is held to the same limit of 6 decimals as a data file.
- The README and NOTICE name pcod (pito-codecs) for the GIF, and the README links to Discord for questions.

### Fixed

- With several parties, bars charts never share a file name any more: a name that slugs to nothing or repeats now gets a number that is free, so no PNG overwrites another.
- Axes with very small steps (a few millionths and below) showed rounded or zero tick labels; they now show the right values, in E notation below 1e-6.

## v1.1.1 — 2026-10-15

### Changed

- The README's speed numbers were measured again, the old and new release side by side: 5.5 to 7.1 ms per chart, 63× to 77× faster than matplotlib.
- CI keeps its build cache from main only, so branch and tag runs no longer crowd it out.

## v1.1.0 — 2026-10-15

### Added

- The library's `render_each` hands each chart over as soon as it is drawn, so only one is held in memory at a time; pcmp now saves each chart before drawing the next.
- The README's Licence section shows the PITO MD wordmark.

### Changed

- Charts are about 30% faster from start to PNG on disk (paired bars 9.3 to 6.6 ms): the PNGs are compressed at the default level, with the same pixels and about 1% more bytes.
- A data file is refused in one line when a value is above 1e15, a ratio is outside 1e-15 to 1e15, or it names more than 12 parties.

### Fixed

- A huge value with `texts` no longer makes pcmp hang and fill memory.
- pcmp refuses to write a chart through a symlink at its output path, so it never overwrites the file the link points to.

## v1.0.2 — 2026-10-14

### Added

- Each release carries pito-compare's documentation as docs-v1.0.2.tar.gz: an overview, install, eight guides with real commands and charts, the command line and data format reference, the library's API and this changelog.

## v1.0.1 — 2026-10-14

### Fixed

- A ratio chart's axis labels never draw over each other: when a wide spread puts the 2× SLOWER tick right beside 1× SAME, the 2× label gives way.

## v1.0.0 — 2026-10-14

pito-compare is 1.0: the charts, the data format and the library are the same as in 0.1.0.

### Changed

- The README shows every chart kind with an example, and pcmp's own numbers: 13 to 23 ms per chart from start to PNG, 39 to 64 times faster than matplotlib on the same data, a 1.2 MB binary and five direct dependencies.
- bench/speed.py measures pcmp against matplotlib, so anyone can run the comparison.
- The brand now reads PITO MD.

## v0.1.0 — 2026-10-11

### Added

- pcmp draws comparison charts as PNGs from a JSON or TOML file: paired bars for two or more parties, the balanced ratio chart and single bars, in the look of pito-codecs' benchmark charts.
- The pito_compare library draws the same charts from data built in code.
- Release binaries for Linux x86_64, Windows x86_64 and macOS on Apple Silicon, with SHA256SUMS.
