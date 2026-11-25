use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use pito_compare::canvas::{BG, GRID, SLOWER, TEXT};
use pito_compare::{Canvas, Chart, Format, Kind, render_at};

const WIDTHS: [i64; 3] = [405, 540, 1080];
const SIDES: [(i64, usize); 3] = [(405, 15), (540, 20), (1080, 40)];

fn load(path: &str) -> Chart {
    Chart::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("loads")
}

fn first(chart: &Chart, kind: Kind, width: i64) -> Canvas {
    render_at(chart, kind, width).expect("renders").remove(0).1
}

fn bars(values: &[f64], max: Option<f64>) -> Chart {
    let rows: Vec<String> = values
        .iter()
        .enumerate()
        .map(|(i, v)| format!(r#"{{"label":"ROW {}","values":[{v}]}}"#, i + 1))
        .collect();
    let text = format!(
        r#"{{"title":"DISK USED","better":"lower","bare":true,"suffix":"%","decimals":0,{}"parties":[{{"name":"USED","colour":"slower"}}],"rows":[{}]}}"#,
        max.map_or(String::new(), |m| format!(r#""max":{m},"#)),
        rows.join(",")
    );
    Chart::parse(&text, Format::Json).expect("parses")
}

fn paired(values: &[[f64; 2]], max: Option<f64>) -> Chart {
    let rows: Vec<String> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            format!(
                r#"{{"label":"ROW {}","values":[{},{}]}}"#,
                i + 1,
                v[0],
                v[1]
            )
        })
        .collect();
    let text = format!(
        r#"{{"title":"DISK USED","better":"lower","bare":true,"suffix":"%","decimals":0,{}"parties":[{{"name":"NOW"}},{{"name":"BEFORE"}}],"rows":[{}]}}"#,
        max.map_or(String::new(), |m| format!(r#""max":{m},"#)),
        rows.join(",")
    );
    Chart::parse(&text, Format::Json).expect("parses")
}

fn grid_columns(canvas: &Canvas) -> BTreeSet<usize> {
    let mut at = BTreeSet::new();
    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            if canvas.pixel(x, y) == Some(GRID) {
                at.insert(x);
            }
        }
    }
    at
}

fn bar_lengths(canvas: &Canvas) -> Vec<usize> {
    let below = (0..canvas.height())
        .find(|&y| (0..canvas.width()).any(|x| canvas.pixel(x, y) == Some(GRID)))
        .expect("a grid line");
    let mut lengths = Vec::new();
    let mut last = 0;
    for y in below..canvas.height() {
        let n = (0..canvas.width())
            .filter(|&x| canvas.pixel(x, y) == Some(SLOWER))
            .count();
        if n > 0 && n != last {
            lengths.push(n);
        }
        last = n;
    }
    lengths
}

fn right_margin_is_empty(canvas: &Canvas, side: usize) -> bool {
    (canvas.width() - side..canvas.width())
        .all(|x| (0..canvas.height()).all(|y| canvas.pixel(x, y) == Some(BG)))
}

fn rightmost(canvas: &Canvas, colour: [u8; 3]) -> usize {
    (0..canvas.width())
        .rev()
        .find(|&x| (0..canvas.height()).any(|y| canvas.pixel(x, y) == Some(colour)))
        .expect("the colour is drawn")
}

#[test]
fn two_charts_with_the_same_max_share_one_scale_at_every_width() {
    let high = load("tests/golden/max-high.toml");
    let low = load("tests/golden/max-low.toml");
    let full = bars(&[100.0], Some(100.0));
    for width in WIDTHS {
        let (a, b, c) = (
            first(&high, Kind::Bars, width),
            first(&low, Kind::Bars, width),
            first(&full, Kind::Bars, width),
        );
        assert_eq!(grid_columns(&a), grid_columns(&b), "{width}");
        assert_eq!(grid_columns(&a), grid_columns(&c), "{width}");
        let axis = bar_lengths(&c)[0] as f64;
        let each = |canvas: &Canvas| bar_lengths(canvas);
        let expect = |values: &[f64]| -> Vec<usize> {
            values
                .iter()
                .map(|v| (v / 100.0 * axis).round() as usize)
                .collect()
        };
        assert_eq!(each(&a), expect(&[99.0, 42.0, 7.0]), "{width}");
        assert_eq!(each(&b), expect(&[20.0, 40.0, 40.0]), "{width}");
    }
}

#[test]
fn the_axis_runs_to_max_whatever_the_values_are() {
    for width in WIDTHS {
        let wide = first(&bars(&[100.0], Some(100.0)), Kind::Bars, width);
        for values in [[1.0], [0.0], [55.5]] {
            let other = first(&bars(&values, Some(100.0)), Kind::Bars, width);
            assert_eq!(grid_columns(&wide), grid_columns(&other), "{width}");
        }
        let end = grid_columns(&wide).into_iter().next_back().expect("ticks");
        assert_eq!(
            end - SIDES.iter().find(|s| s.0 == width).expect("side").1,
            bar_lengths(&wide)[0],
            "{width}"
        );
    }
}

#[test]
fn a_bar_that_reaches_max_leaves_room_for_its_text() {
    for (width, side) in SIDES {
        for text_values in [[100.0, 3.0], [100.0, 100.0]] {
            let c = first(&bars(&text_values, Some(100.0)), Kind::Bars, width);
            assert!(right_margin_is_empty(&c, side), "{width}");
            assert!(rightmost(&c, TEXT) < c.width() - side, "{width}");
        }
        let c = first(
            &paired(&[[100.0, 100.0], [57.0, 100.0]], Some(100.0)),
            Kind::Paired,
            width,
        );
        assert!(right_margin_is_empty(&c, side), "{width}");
    }
}

#[test]
fn a_bar_at_max_with_a_text_too_long_for_the_end_is_refused_not_cut() {
    let with = |text: &str| {
        let data = format!(
            r#"{{"title":"T","better":"lower","parties":[{{"name":"A"}}],"max":100,"rows":[{{"label":"R","values":[100],"texts":["{text}"]}}]}}"#
        );
        Chart::parse(&data, Format::Json).expect("parses")
    };
    for (width, side) in SIDES {
        let c = first(&with("ABCDEFGHIJ"), Kind::Bars, width);
        assert!(right_margin_is_empty(&c, side), "{width}");
        let m = render_at(&with("ABCDEFGHIJKLMN"), Kind::Bars, width)
            .err()
            .expect("refused")
            .to_string();
        assert!(
            m.starts_with(
                "row 1 (R) has a value text too long to fit beside its bar on an axis that ends at max 100"
            ),
            "{width}: {m}"
        );
    }
}

#[test]
fn without_the_key_the_axis_still_follows_the_values() {
    for width in WIDTHS {
        let a = first(&bars(&[100.0, 42.0], None), Kind::Bars, width);
        let b = first(&bars(&[60.0, 42.0], None), Kind::Bars, width);
        assert_ne!(grid_columns(&a), grid_columns(&b), "{width}");
    }
}

#[test]
fn a_max_the_data_already_reaches_draws_the_bytes_it_drew_without_one() {
    for width in WIDTHS {
        for (values, max) in [
            (&[100.0, 42.0, 7.0][..], 100.0),
            (&[50.0, 20.0][..], 50.0),
            (&[10.0, 2.5][..], 10.0),
            (&[200.0, 90.0][..], 200.0),
        ] {
            let with = first(&bars(values, Some(max)), Kind::Bars, width);
            let without = first(&bars(values, None), Kind::Bars, width);
            assert_eq!(
                with.png().expect("png"),
                without.png().expect("png"),
                "{values:?} {width}"
            );
        }
        let rows = [[100.0, 60.0], [42.0, 7.0]];
        let with = first(&paired(&rows, Some(100.0)), Kind::Paired, width);
        let without = first(&paired(&rows, None), Kind::Paired, width);
        assert_eq!(
            with.png().expect("png"),
            without.png().expect("png"),
            "{width}"
        );
    }
}

#[test]
fn every_party_of_a_bars_chart_shares_the_axis() {
    let chart = paired(&[[100.0, 40.0], [30.0, 80.0]], Some(100.0));
    for width in WIDTHS {
        let out = render_at(&chart, Kind::Bars, width).expect("renders");
        assert_eq!(out.len(), 2);
        let a = grid_columns(&out[0].1);
        assert_eq!(a, grid_columns(&out[1].1), "{width}");
        let only = paired(&[[10.0, 5.0], [8.0, 3.0]], Some(100.0));
        let small = render_at(&only, Kind::Bars, width).expect("renders");
        assert_eq!(a, grid_columns(&small[0].1), "{width}");
    }
}

#[test]
fn ticks_stay_round_and_the_last_one_is_max_when_it_falls_on_one() {
    for width in WIDTHS {
        let full = first(&bars(&[1.0], Some(100.0)), Kind::Bars, width);
        assert_eq!(grid_columns(&full).len(), 5, "{width}");
        let odd = first(&bars(&[70.0], Some(70.0)), Kind::Bars, width);
        assert_eq!(grid_columns(&odd).len(), 3, "{width}");
        let end = SIDES.iter().find(|s| s.0 == width).expect("side").1 + bar_lengths(&odd)[0];
        assert!(
            grid_columns(&odd).into_iter().next_back().expect("ticks") < end,
            "{width}"
        );
        let on = first(&bars(&[1.0], Some(60.0)), Kind::Bars, width);
        assert_eq!(grid_columns(&on).len(), 3, "{width}");
        assert_eq!(
            grid_columns(&on).into_iter().next_back(),
            grid_columns(&full).into_iter().next_back(),
            "{width}: a tick on max is at the axis end"
        );
    }
}

#[test]
fn the_ratio_chart_ignores_max() {
    let with = paired(&[[100.0, 60.0], [42.0, 7.0]], Some(100.0));
    let without = paired(&[[100.0, 60.0], [42.0, 7.0]], None);
    for width in WIDTHS {
        assert_eq!(
            first(&with, Kind::Ratio, width).png().expect("png"),
            first(&without, Kind::Ratio, width).png().expect("png"),
            "{width}"
        );
    }
}

#[test]
fn a_value_over_max_is_refused_and_draws_nothing() {
    let over = r#"{"title":"T","better":"lower","parties":[{"name":"NOW"},{"name":"BEFORE"}],"max":100,
"rows":[{"label":"FIRST","values":[5,6]},{"label":"SECOND","values":[7,100.5]}]}"#;
    let m = Chart::parse(over, Format::Json)
        .expect_err("refused")
        .to_string();
    assert_eq!(
        m,
        "groups[0].rows[1].values holds 100.5 for BEFORE on SECOND: values are at most max, 100"
    );
    let chart =
        Chart::parse(&over.replace("100.5", "100"), Format::Json).expect("max itself is fine");
    for width in WIDTHS {
        for kind in [Kind::Paired, Kind::Ratio, Kind::Bars] {
            let mut over = chart.clone();
            over.groups[0].rows[1].values[1] = 100.5;
            let e = render_at(&over, kind, width).err().expect("refused");
            assert!(
                e.to_string()
                    .starts_with("groups[0].rows[1].values holds 100.5"),
                "{e}"
            );
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn the_command_line_refuses_a_value_over_max_with_one_line_and_no_png() {
    let dir = scratch("max-cli");
    let data = dir.join("alert.toml");
    std::fs::write(
        &data,
        "title = \"DISK\"\nbetter = \"lower\"\nmax = 100\nparties = [{ name = \"USED\" }]\nrows = [{ label = \"DATA\", values = [101] }]\n",
    )
    .expect("data");
    let run = Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .arg(&data)
        .args(["--width", "405", "--out"])
        .arg(&dir)
        .output()
        .expect("pcmp runs");
    assert_eq!(run.status.code(), Some(1));
    assert!(run.stdout.is_empty());
    let line = String::from_utf8(run.stderr).expect("utf-8");
    assert_eq!(
        line.trim_end(),
        format!(
            "pcmp: {}: groups[0].rows[0].values holds 101 for USED on DATA: values are at most max, 100",
            data.display()
        )
    );
    assert!(!dir.join("alert-bars.png").exists());
    std::fs::write(
        &data,
        std::fs::read_to_string(&data)
            .expect("data")
            .replace("101", "100"),
    )
    .expect("data");
    let run = Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .arg(&data)
        .args(["--width", "405", "--out"])
        .arg(&dir)
        .output()
        .expect("pcmp runs");
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    assert!(dir.join("alert-bars.png").exists());
}
