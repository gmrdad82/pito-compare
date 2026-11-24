use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use pito_compare::{Chart, Kind, render, render_at};

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
fn every_kind_at_540_is_540_wide_and_the_same_bytes_every_run() {
    let mut kinds = Vec::new();
    for path in EXAMPLES {
        let chart = load(path);
        for kind in [Kind::Paired, Kind::Ratio, Kind::Bars] {
            let Ok(first) = render_at(&chart, kind, 540) else {
                continue;
            };
            let second = render_at(&chart, kind, 540).expect("again");
            for ((name, a), (_, b)) in first.iter().zip(&second) {
                assert_eq!(a.width(), 540, "{path} {name}");
                assert_eq!(
                    a.png().expect("png"),
                    b.png().expect("png"),
                    "{path} {name}"
                );
                assert!(a.height() > 300, "{path} {name}");
                kinds.push(name.split('-').next().unwrap_or("").to_string());
            }
        }
    }
    for kind in ["paired", "ratio", "bars"] {
        assert!(kinds.iter().any(|k| k == kind), "{kind}");
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
fn the_command_line_writes_540_and_1080_wide_pngs() {
    let dir = scratch("width-cli");
    let out = dir.to_str().expect("utf-8");
    let run = pcmp(&["examples/speed.toml", "--width", "540", "--out", out]);
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    let narrow = std::fs::read(dir.join("speed-paired.png")).expect("png");
    assert_eq!(png_width(&narrow), 540);
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
    for width in ["700", "0", "1081", "2160", "-540"] {
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
            text.starts_with("pcmp: --width: ") && text.ends_with("the widths are 540 and 1080\n"),
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
    assert!(err.message().ends_with("the widths are 540 and 1080"));
}
