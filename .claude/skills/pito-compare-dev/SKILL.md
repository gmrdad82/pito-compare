---
name: pito-compare-dev
description: "Develop pito-compare itself: the `pito_compare` Rust library and `pcmp`, its command line that draws paired, ratio and single-bar chart PNGs from a JSON or TOML file. Triggers: pito-compare, pito_compare, pcmp source, bin/gate, bin/images, bin/docs-bundle, bin/install, src/chart.rs, src/draw.rs, src/canvas.rs, src/skill.rs, a refusal message, the data format, the pixel font, the look of a chart, tests/look.rs, tests/skill_drift.rs, docs/pages, the docs bundle, CHANGELOG, a pcmp release, the local install. Excludes drawing a chart with pcmp (that is the pcmp skill, shipped from this repository's skills/pcmp folder)."
---

# pito-compare development

pito-compare is the `pito_compare` Rust library and `pcmp`, its command line. It reads one JSON or TOML data file and draws dark, 1080 px wide comparison charts (paired bars, balanced ratios, single bars) as PNGs in its own 5x7 pixel font, with no plotting library, so a chart is the same pixels on every machine and a wrong chart is refused, never drawn. It is one MIT crate (edition 2024, Rust 1.97 or newer) for Linux, macOS (Apple Silicon) and Windows, and this repository is public.

This skill is for changing pito-compare. Drawing a chart with the installed `pcmp` is the `pcmp` skill, whose files are [`skills/pcmp/`](../../../skills/pcmp/SKILL.md) here and are built into the binary.

The rules live in `AGENTS.md` (`CLAUDE.md` imports it), and the open work lives on its P99 board (space pito, project pito-compare): `p99 cards --space pito --json` lists it. Read both, and keep to the speed rules in `AGENTS.md`: they decide what to run. `AGENTS.md` is an untracked file of the main checkout, so a worktree has none: read it in the main checkout, which `git worktree list` names first. This skill adds the how-to and does not repeat them.

## Layout

- `src/lib.rs`: `Kind`, `Error`, `render` and `render_each` (which charts a kind and the party count draw, and the names of the bars files)
- `src/chart.rs`: the data model (`Chart`, `Party`, `Group`, `Row`), `load` and `parse` for JSON and TOML, the colour roles, and `Chart::check`, which holds most refusals
- `src/draw.rs`: `paired`, `ratios` and `bars`: layout, ticks, tags, and the refusals for text that does not fit
- `src/canvas.rs`: the colours, the glyph table, text and the PNG write through pito-codecs
- `src/main.rs`: the clap command line, the bare `pcmp` summary and the exit codes; `src/skill.rs`: `pcmp skill install` and `status`, which embed `skills/pcmp/`
- `tests/`: `chart.rs` (data model and refusals), `library.rs` (public API), `cli.rs` (the binary), `look.rs` (the examples drawn against `docs/images`), `skill.rs` and `skill_drift.rs` (the product skill against the command line), `installer.rs` (`bin/install`), `docs_bundle.rs` (`bin/docs-bundle`), `dev_skill.rs` (this skill)
- `examples/` and `bench/`: the data files `bin/images` draws into `docs/images/`; `docs/pages/`: the source of the docs site; `render/`: the README clip's recipe; `assets/marks/`: the wordmark `bin/marks` copies, never edited here
- [`README.md`](../../../README.md), [`CHANGELOG.md`](../../../CHANGELOG.md), [`NOTICE`](../../../NOTICE) (every non-code file and where it comes from) and [`INSPIRATION.md`](../../../INSPIRATION.md) are tracked and public

## How work goes

- `main` is the branch and is checked out in the main checkout. A piece of work from Dispatch runs in its own git worktree on `dispatch/<id>-<slug>`, commits there, signed and staged by name, and merges into `main` from the main checkout with `git -C <main checkout> merge <branch>`. It does not push unless its brief says so.
- A piece's brief says what its check judges and where to save the quick check's output. Run the quick check on the final commit, before the merge.
- A fault in pito-codecs (a git dependency pinned by tag in `Cargo.toml`), prec or pcod goes to that repository as a Dispatch plan, not as a workaround here.

## Critical rules

- **A chart is never wrong.** Bad data is refused with one line (`pcmp: <file>: <what>`, exit 1) and no PNG. Do not loosen a refusal to make a file draw. Its text lives in `src/chart.rs` or `src/draw.rs`; the refusals `skills/pcmp/output.md` lists are also pinned by a case in `tests/skill_drift.rs`, so change them together.
- **The look is pixel exact.** `tests/look.rs` compares the examples' charts with `docs/images`. After a change to colours, font, sizes or layout, run `bin/images` and commit the changed PNGs. Never edit a PNG by hand, and leave `docs/images/draw.gif` alone: it is made with prec and pcod from `render/terminal.toml`.
- **The command line and its skill move together.** `tests/skill_drift.rs` fails when a flag, command or kind of `pcmp --help` is missing from `skills/pcmp/`, or the other way round.
- **Run `bin/install` or `pcmp skill install` for real only as the local install row says.** `bin/install` replaces `~/.local/bin/pcmp`, and both write their skill folders under `~/.claude` and `~/.agents`, so never as a check or a try-out. The tests use a temporary `HOME` and `PCMP_BINARY`.
- **No home path in the docs.** `bin/docs-bundle` refuses a Linux or macOS home-folder path in any generated file, so docs and the changelog show `~`.
- **This is a public repository.** Tracked files, this skill included, hold nothing internal: no private repositories, paths on a machine, people, prices, keys or plans.
- Code has no comments and tests are deterministic: scratch folders under `CARGO_TARGET_TMPDIR`, fixed data, no sleeps.
- Dependencies stay few: clap (optional, the `cli` feature), serde, serde_json, toml and pito-codecs (PNG only). `Cargo.lock` is committed and every cargo call uses `--locked`, so a dependency or version change commits the lock too.
- The library builds with `--no-default-features` (no clap), and the gate lints that. keep `src/` free of Unix-only paths and shell calls, and mark a test that needs Unix `#![cfg(unix)]`.
- A release is the maintainer's: do not tag, push or publish unless asked, and follow the release steps in `AGENTS.md`, never improvise them.

## Known pitfalls

- A fresh worktree has an empty `target/`, so its first gate builds everything, and pito-codecs comes from git or the cargo cache.
- `bin/gate --fast` and `bin/gate` run the same steps and every test; only the last line differs, so the cheap loop is `cargo fmt`, clippy and the one test file of what changed.
- `tests/docs_bundle.rs` runs `bin/docs-bundle` for real (cargo doc included) and is the slowest test.
- `bin/docs-bundle` also refuses a page without frontmatter, and an image a page uses that `docs/images` lacks or `NOTICE` does not name.
- `bin/images` writes into `docs/images` by default; `bin/images tmp/images` draws elsewhere to look first.
- Adding or dropping a file in `skills/pcmp/` means editing `FILES` in `src/skill.rs` and in `tests/skill.rs`, `tests/skill_drift.rs` and `tests/installer.rs`.
- A version appears in `Cargo.toml`, `Cargo.lock`, the top of `CHANGELOG.md` and the example output of `docs/pages/skill.md`.

## Command discovery

| Command | What it does |
| --- | --- |
| `bin/gate --fast` | the quick check, run on the final commit before a merge: ShellCheck when installed, `cargo fmt --check`, clippy with warnings denied for all targets and for the library alone, then `cargo test --all-targets --locked` |
| `bin/gate` | the full gate, the same steps ending `gate: green`; run it when the maintainer asks, as `AGENTS.md` says |
| `cargo test --locked --test chart` | one test file: `chart`, `cli`, `library`, `look`, `skill`, `skill_drift`, `installer`, `docs_bundle` or `dev_skill`; `cargo test --locked --lib` runs the unit tests in `src/` |
| `cargo fmt` and `cargo clippy --all-targets --locked -- -D warnings` | format and lint what changed |
| `cargo build --release --locked --bin pcmp` | the release binary at `target/release/pcmp` |
| `cargo run --locked --bin pcmp -- examples/speed.toml --out tmp/charts` | draw an example from the working tree into the git-ignored `tmp/` |
| `bin/images` | redraw `docs/images` from `examples/` and `bench/`; `PCMP` names a binary to use instead of building one |
| `bin/docs-bundle` | build `docs-vX.Y.Z.tar.gz` into `tmp/docs` from `docs/pages`, `pcmp --help`, the changelog and `cargo doc`, and print its path |
| `bin/install` | the local install, run from a clone's root: builds the release binary, puts it in `~/.local/opt/pcmp/pcmp`, links it from `~/.local/bin/pcmp` and runs `pcmp skill install` for the user skill (`~/.claude/skills/pcmp`, `~/.agents/skills/pcmp`); pcmp has no window app. Run it again to update: after a release (from `main` at the tag), or after a build that will be used. It takes no flag and has no `--help` |
| `pcmp --help`, `pcmp skill --help` | every flag, command and kind of the command line |

`pcmp --help` and `cargo <command> --help` are the source of truth for their options. The scripts in `bin/` are short and have no `--help`: read the script (`bin/gate` and `bin/install` print a usage line on a wrong argument, and nothing is installed then).

## Decision framework

1. **A refusal, limit or key of the data file** → `src/chart.rs` (`Chart::check`, the serde structs) or `src/draw.rs`, a test in `tests/chart.rs`, then the text in `skills/pcmp/output.md` and `tests/skill_drift.rs`, `docs/pages/guides/refusals.md` and `docs/pages/reference/formats.md`. The README's data format table changes with it. Quick check.
2. **How a chart looks or lays out** → `src/draw.rs` or `src/canvas.rs`, then `bin/images` and the changed PNGs in the same commit, then `cargo test --locked --test look`.
3. **A flag, command or chart kind** → `src/main.rs` (its `FORMS` and help text), `skills/pcmp/`, the README's Use section, `docs/pages/`, `tests/cli.rs`; the docs' command line page is generated from `pcmp --help`.
4. **The library's API** → `src/lib.rs` or `src/chart.rs`, `tests/library.rs`, the README's library snippet and `docs/pages/guides/library.md`.
5. **A docs page** → `docs/pages/` with its frontmatter, edited with the code it describes; check it with `bin/docs-bundle`.
6. **The product skill's text** → `skills/pcmp/`, then `cargo test --locked --test skill_drift --test skill --test installer`.
7. **This skill** → edit it, then `cargo test --locked --test dev_skill`.
8. **A release, only when asked** → bump the version everywhere the pitfalls list, write the changelog entry (`## vX.Y.Z — date` with `### Added`, `### Changed` or `### Fixed`), then follow `AGENTS.md`: it names the archives (Linux, macOS and Windows binaries, the docs bundle and `SHA256SUMS`) and how they are built and published. A release that ships a bug gets a quick fix release. After the release, update the local install with `bin/install`. Release from the main checkout with `PATH=$HOME/.cargo/bin:$PATH`: the system `/usr/bin/rustc` has no Windows target.
