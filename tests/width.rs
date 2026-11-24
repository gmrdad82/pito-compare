use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use pito_compare::canvas::{BG, DIM, FASTER, OURS};
use pito_compare::{Better, Canvas, Chart, Format, Group, Kind, Row, render, render_at};

const EXAMPLES: [&str; 5] = [
    "examples/speed.toml",
    "examples/build.toml",
    "examples/frames.toml",
    "examples/latency.json",
    "examples/spread.toml",
];

fn load(path: &str) -> Chart {
    Chart::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("example loads")
}

fn pcmp(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("pcmp runs")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn png_width(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes[16..20].try_into().expect("ihdr"))
}

#[test]
fn every_kind_at_540_and_405_is_that_wide_and_the_same_bytes_every_run() {
    for width in [540, 405] {
        let mut kinds = Vec::new();
        for path in EXAMPLES {
            let chart = load(path);
            for kind in [Kind::Paired, Kind::Ratio, Kind::Bars] {
                let Ok(first) = render_at(&chart, kind, width) else {
                    continue;
                };
                let second = render_at(&chart, kind, width).expect("again");
                for ((name, a), (_, b)) in first.iter().zip(&second) {
                    assert_eq!(a.width(), width as usize, "{path} {name}");
                    assert_eq!(
                        a.png().expect("png"),
                        b.png().expect("png"),
                        "{path} {name}"
                    );
                    assert!(a.height() > 150, "{path} {name}");
                    kinds.push(name.split('-').next().unwrap_or("").to_string());
                }
            }
        }
        for kind in ["paired", "ratio", "bars"] {
            assert!(kinds.iter().any(|k| k == kind), "{width} {kind}");
        }
    }
}

#[test]
fn the_default_width_draws_exactly_what_1080_draws() {
    for path in EXAMPLES {
        let chart = load(path);
        let a = render(&chart, Kind::All).expect("default");
        let b = render_at(&chart, Kind::All, 1080).expect("1080");
        assert_eq!(a.len(), b.len());
        for ((na, ca), (nb, cb)) in a.iter().zip(&b) {
            assert_eq!(na, nb);
            assert_eq!(ca.width(), 1080);
            assert_eq!(
                ca.png().expect("png"),
                cb.png().expect("png"),
                "{path} {na}"
            );
        }
    }
}

#[test]
fn text_at_540_is_drawn_at_whole_pixel_sizes_without_blur() {
    let chart = load("examples/speed.toml");
    let out = render_at(&chart, Kind::Paired, 540).expect("540");
    let canvas = &out[0].1;
    let mut colours = std::collections::BTreeSet::new();
    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            colours.insert(canvas.pixel(x, y).expect("pixel"));
        }
    }
    assert!(colours.len() <= 12, "{}", colours.len());
}

#[test]
fn the_command_line_writes_405_540_and_1080_wide_pngs() {
    let dir = scratch("width-cli");
    let out = dir.to_str().expect("utf-8");
    let run = pcmp(&["examples/speed.toml", "--width", "540", "--out", out]);
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    let narrow = std::fs::read(dir.join("speed-paired.png")).expect("png");
    assert_eq!(png_width(&narrow), 540);
    let run = pcmp(&["examples/speed.toml", "-w", "405", "--out", out]);
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    let slim = std::fs::read(dir.join("speed-paired.png")).expect("png");
    assert_eq!(png_width(&slim), 405);
    let run = pcmp(&["examples/speed.toml", "-w", "1080", "--out", out]);
    assert_eq!(run.status.code(), Some(0));
    let wide = std::fs::read(dir.join("speed-paired.png")).expect("png");
    assert_eq!(png_width(&wide), 1080);
    let other = scratch("width-cli-default");
    let run = pcmp(&[
        "examples/speed.toml",
        "--out",
        other.to_str().expect("utf-8"),
    ]);
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(
        wide,
        std::fs::read(other.join("speed-paired.png")).expect("png")
    );
}

#[test]
fn a_width_it_cannot_draw_is_refused_with_one_line_and_exit_2() {
    let dir = scratch("width-refused");
    for width in ["700", "0", "1081", "2160", "-540", "406", "404", "-405"] {
        let run = pcmp(&[
            "examples/speed.toml",
            &format!("--width={width}"),
            "--out",
            dir.to_str().expect("utf-8"),
        ]);
        assert_eq!(run.status.code(), Some(2), "{width}");
        assert!(run.stdout.is_empty(), "{width}");
        let text = String::from_utf8(run.stderr).expect("utf-8");
        assert!(
            text.starts_with("pcmp: --width: ")
                && text.ends_with("the widths are 405, 540 and 1080\n"),
            "{text}"
        );
        assert_eq!(text.lines().count(), 1, "{text}");
    }
    assert!(!dir.join("speed-paired.png").exists());
    let run = pcmp(&["examples/speed.toml", "--width", "wide"]);
    assert_eq!(run.status.code(), Some(2));
}

#[test]
fn a_title_that_fits_1080_but_not_540_is_refused_naming_its_field() {
    let mut chart = load("examples/speed.toml");
    chart.title = "A".repeat(30);
    assert!(render_at(&chart, Kind::Paired, 1080).is_ok());
    let err = render_at(&chart, Kind::Paired, 540)
        .err()
        .expect("too long");
    assert!(
        err.message()
            .starts_with("title is too long for a 540 px wide chart"),
        "{err}"
    );
}

#[test]
fn the_library_refuses_a_width_too() {
    let chart = load("examples/speed.toml");
    let err = render_at(&chart, Kind::All, 720).err().expect("refused");
    assert!(err.message().ends_with("the widths are 405, 540 and 1080"));
}

const SLIM_LEFT: usize = 15;
const SLIM_MARGIN: usize = 12;

fn first(chart: &Chart, kind: Kind, width: i64) -> Canvas {
    render_at(chart, kind, width).expect("renders").remove(0).1
}

fn count(canvas: &Canvas, colour: [u8; 3]) -> usize {
    let mut n = 0;
    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            if canvas.pixel(x, y) == Some(colour) {
                n += 1;
            }
        }
    }
    n
}

fn blank_rows_below(canvas: &Canvas) -> usize {
    (0..canvas.height())
        .rev()
        .take_while(|&y| (0..canvas.width()).all(|x| canvas.pixel(x, y) == Some(BG)))
        .count()
}

fn legend_row(canvas: &Canvas, x: usize) -> usize {
    (0..canvas.height())
        .find(|&y| canvas.pixel(x, y) == Some(OURS))
        .expect("a swatch")
}

fn title_rows(canvas: &Canvas) -> usize {
    (SLIM_MARGIN..canvas.height())
        .take_while(|&y| (0..canvas.width()).any(|x| canvas.pixel(x, y).is_some_and(|c| c != BG)))
        .count()
}

#[test]
fn nothing_at_405_touches_a_margin_or_is_cut() {
    let mut drawn = 0;
    let data = EXAMPLES.iter().copied().chain([
        "bench/pcmp-vs-matplotlib.toml",
        "tests/golden/alert-two.toml",
        "tests/golden/alert-one.toml",
    ]);
    for path in data {
        let chart = load(path);
        for kind in [Kind::Paired, Kind::Ratio, Kind::Bars] {
            let Ok(out) = render_at(&chart, kind, 405) else {
                continue;
            };
            for (name, canvas) in out {
                drawn += 1;
                let (w, h) = (canvas.width(), canvas.height());
                for y in 0..h {
                    for x in 0..w {
                        let inside = x >= SLIM_LEFT
                            && x < w - SLIM_LEFT
                            && y >= SLIM_MARGIN
                            && y < h - SLIM_MARGIN;
                        assert!(
                            inside || canvas.pixel(x, y) == Some(BG),
                            "{path} {name} has ink in a margin at {x},{y}"
                        );
                    }
                }
            }
        }
    }
    assert!(drawn >= 12, "{drawn}");
}

#[test]
fn text_at_405_is_drawn_at_whole_pixel_sizes_without_blur() {
    for kind in [Kind::Paired, Kind::Ratio] {
        let canvas = first(&load("examples/speed.toml"), kind, 405);
        let mut colours = std::collections::BTreeSet::new();
        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                colours.insert(canvas.pixel(x, y).expect("pixel"));
            }
        }
        assert!(colours.len() <= 12, "{}", colours.len());
    }
}

#[test]
fn at_405_the_title_is_size_2_and_falls_back_to_size_1_when_2_does_not_fit() {
    let mut chart = load("examples/speed.toml");
    for kind in [Kind::Paired, Kind::Ratio] {
        chart.title = "A".repeat(31);
        assert_eq!(title_rows(&first(&chart, kind, 405)), 14, "{kind:?}");
        chart.title = "A".repeat(32);
        assert_eq!(title_rows(&first(&chart, kind, 405)), 7, "{kind:?}");
        chart.title = "A".repeat(41);
        assert_eq!(title_rows(&first(&chart, kind, 405)), 7, "{kind:?}");
    }
}

#[test]
fn what_cannot_be_drawn_well_at_405_is_refused_naming_it() {
    let mut chart = load("examples/speed.toml");
    chart.date.clear();
    chart.subtitle = "A".repeat(62);
    assert!(render_at(&chart, Kind::Paired, 405).is_ok());
    chart.subtitle = "A".repeat(63);
    assert!(render_at(&chart, Kind::Paired, 1080).is_ok());
    let err = render_at(&chart, Kind::Paired, 405).err().expect("refused");
    assert_eq!(
        err.message(),
        "subtitle and date is too long for a 405 px wide chart: at most 62 characters fit"
    );

    let mut huge = Chart::new("T", "", Better::Higher, &["A", "B"]);
    huge.groups.push(Group {
        name: String::new(),
        rows: vec![Row::new("X", &[1e7, 1.0])],
    });
    assert!(render_at(&huge, Kind::Ratio, 1080).is_ok());
    let err = render_at(&huge, Kind::Ratio, 405).err().expect("refused");
    assert!(
        err.message()
            .starts_with("row 1 (X) has a tag too wide for a 405 px wide chart"),
        "{err}"
    );

    let mut long = Chart::new("T", "", Better::Higher, &["A", "B"]);
    long.words.better = "MUCH MUCH MUCH MUCH MUCH FASTER".into();
    long.groups.push(Group {
        name: String::new(),
        rows: vec![Row::new("A LONG LABEL ABOUT THIRTY CHARS", &[9.0, 1.0])],
    });
    assert!(render_at(&long, Kind::Paired, 540).is_ok());
    let err = render_at(&long, Kind::Paired, 405).err().expect("refused");
    assert!(
        err.message()
            .contains("and its tag do not fit one line of a 405 px wide chart"),
        "{err}"
    );

    let mut wide = Chart::new("T", "", Better::Higher, &["A", "B"]);
    wide.groups.push(Group {
        name: String::new(),
        rows: vec![Row::new("X", &[1e12, 5e11])],
    });
    assert!(render_at(&wide, Kind::Paired, 1080).is_ok());
    let err = render_at(&wide, Kind::Paired, 405).err().expect("refused");
    assert_eq!(
        err.message(),
        "the axis labels overlap on a 405 px wide chart: use smaller values"
    );
}

const EMPTY: &str = r#"
title = "T"
better = "higher"
parties = [{ name = "A" }, { name = "B" }]
rows = [{ label = "ONE", values = [900, 100] }, { label = "TWO", values = [5, 6] }]
"#;

fn parse(text: &str) -> Chart {
    Chart::parse(text, Format::Toml).expect("parses")
}

#[test]
fn a_bare_chart_leaves_no_gap_where_an_empty_unit_subtitle_and_notes_would_be() {
    let plain = parse(EMPTY);
    let bare = parse(&format!("bare = true\n{EMPTY}"));
    for (width, shift) in [(405, 28), (540, 40), (1080, 74)] {
        for kind in [Kind::Paired, Kind::Bars] {
            let (a, b) = (first(&plain, kind, width), first(&bare, kind, width));
            let x = if width == 405 {
                15
            } else {
                40 * width as usize / 1080
            };
            if kind == Kind::Paired {
                assert_eq!(legend_row(&a, x) - legend_row(&b, x), shift, "{width}");
            }
            assert!(a.height() > b.height() + shift, "{width} {kind:?}");
        }
    }
    let (a, b) = (
        first(&plain, Kind::Paired, 405),
        first(&bare, Kind::Paired, 405),
    );
    assert_eq!(blank_rows_below(&b), SLIM_MARGIN);
    assert!(blank_rows_below(&a) > 30, "{}", blank_rows_below(&a));
}

#[test]
fn bare_changes_nothing_when_nothing_is_empty() {
    for width in [405, 540, 1080] {
        for (path, kind) in [
            ("examples/speed.toml", Kind::Paired),
            ("examples/speed.toml", Kind::Ratio),
            ("examples/build.toml", Kind::Bars),
        ] {
            let mut chart = load(path);
            let plain = first(&chart, kind, width).png().expect("png");
            chart.bare = true;
            let bare = first(&chart, kind, width).png().expect("png");
            assert_eq!(plain, bare, "{path} {kind:?} {width}");
        }
    }
}

const DASHED: &str = r##"{"title":"JOBS TODAY","better":"higher","decimals":0,
"words":{"better":"-","worse":"-","same":"-"},
"parties":[{"name":"DONE","colour":"#2E7D32"},{"name":"FAILED","colour":"#C62828"}],
"rows":[{"label":"APOCALYPSE-2","values":[1,1],"ratio":1},{"label":"OTHER","values":[4,2]}]}"##;

fn dashed(bare: bool, words: &str) -> Chart {
    let json = DASHED.replace(r#""words":{"better":"-","worse":"-","same":"-"}"#, words);
    let mut chart = Chart::parse(&json, Format::Json).expect("parses");
    chart.bare = bare;
    chart
}

#[test]
fn a_bare_chart_leaves_the_dash_of_its_words_out_of_the_tags() {
    let all = r#""words":{"better":"-","worse":"-","same":"-"}"#;
    let plain = first(&dashed(false, all), Kind::Paired, 405);
    let bare = first(&dashed(true, all), Kind::Paired, 405);
    assert_eq!(count(&plain, DIM) - count(&bare, DIM), 5);
    assert_eq!(count(&plain, FASTER) - count(&bare, FASTER), 5);
    for width in [540, 1080] {
        let plain = first(&dashed(false, all), Kind::Paired, width);
        let bare = first(&dashed(true, all), Kind::Paired, width);
        assert!(count(&plain, DIM) > count(&bare, DIM), "{width}");
        assert!(count(&plain, FASTER) > count(&bare, FASTER), "{width}");
    }
    let some = r#""words":{"same":"-"}"#;
    let plain = first(&dashed(false, some), Kind::Paired, 405);
    let bare = first(&dashed(true, some), Kind::Paired, 405);
    assert_eq!(count(&plain, DIM), count(&bare, DIM));
    assert_eq!(count(&plain, FASTER), count(&bare, FASTER));
}
