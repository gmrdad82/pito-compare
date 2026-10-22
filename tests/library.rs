use std::path::{Path, PathBuf};

use pito_compare::canvas::{BG, DIM, FASTER, SLOWER};
use pito_compare::draw::{LEFT, MAX_HEIGHT, ROW, WIDTH};
use pito_compare::{Better, Canvas, Chart, Color, Error, Group, Kind, Row, bars, paired, ratios};

fn chart(parties: &[&str], rows: &[&[f64]]) -> Chart {
    chart_by(Better::Higher, parties, rows)
}

fn chart_by(better: Better, parties: &[&str], rows: &[&[f64]]) -> Chart {
    let mut c = Chart::new("SPEED", "MB/S", better, parties);
    c.groups.push(Group {
        name: String::new(),
        rows: rows
            .iter()
            .enumerate()
            .map(|(i, v)| Row::new(&format!("ROW {i}"), v))
            .collect(),
    });
    c
}

fn names(out: &[(String, Canvas)]) -> Vec<&str> {
    out.iter().map(|(n, _)| n.as_str()).collect()
}

fn has(c: &Canvas, xs: std::ops::Range<usize>, ys: std::ops::Range<usize>, colour: Color) -> bool {
    ys.flat_map(|y| xs.clone().map(move |x| (x, y)))
        .any(|(x, y)| c.pixel(x, y) == Some(colour))
}

fn bar_length(c: &Canvas, y: usize, colour: Color) -> usize {
    (LEFT as usize..c.width())
        .take_while(|&x| c.pixel(x, y) == Some(colour))
        .count()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn each_kind_renders_what_it_names_for_one_party_and_for_several() {
    let one = chart(&["OURS"], &[&[3.0]]);
    let two = chart(&["OURS", "THEIRS"], &[&[3.0, 2.0]]);
    let cases: [(&Chart, Kind, &[&str]); 8] = [
        (&one, Kind::All, &["bars"]),
        (&one, Kind::Bars, &["bars"]),
        (&two, Kind::All, &["paired", "ratio"]),
        (&two, Kind::Paired, &["paired"]),
        (&two, Kind::Ratio, &["ratio"]),
        (&two, Kind::Bars, &["bars-ours", "bars-theirs"]),
        (&one, Kind::Paired, &[]),
        (&one, Kind::Ratio, &[]),
    ];
    for (c, kind, want) in cases {
        match pito_compare::render(c, kind) {
            Ok(out) => {
                assert_eq!(names(&out), want, "{kind:?}");
                for (_, canvas) in &out {
                    assert_eq!(canvas.width(), WIDTH as usize);
                    assert_eq!(canvas.pixel(0, 0), Some(BG));
                    assert_eq!(
                        canvas.pixel(canvas.width() - 1, canvas.height() - 1),
                        Some(BG)
                    );
                }
                if kind == Kind::Bars {
                    for ((_, canvas), p) in out.iter().zip(&c.parties) {
                        let y = 250 + 32 + 10;
                        assert_eq!(
                            canvas.pixel(LEFT as usize + 1, y),
                            Some(p.colour),
                            "{}",
                            p.name
                        );
                    }
                }
            }
            Err(e) => {
                assert!(want.is_empty(), "{kind:?}: {e}");
                let why = if kind == Kind::Ratio {
                    "a ratio on every row"
                } else {
                    "two parties"
                };
                assert!(e.message().contains(why), "{kind:?}: {e}");
            }
        }
    }
}

#[test]
fn render_each_hands_over_the_same_canvases_as_render() {
    let c = chart(&["OURS", "THEIRS"], &[&[3.0, 2.0]]);
    let mut seen = Vec::new();
    pito_compare::render_each(&c, Kind::All, |name, canvas| {
        seen.push((name.to_string(), canvas.png()?));
        Ok(())
    })
    .expect("renders");
    let all = pito_compare::render(&c, Kind::All).expect("renders");
    assert_eq!(seen.len(), all.len());
    for ((n1, png), (n2, canvas)) in seen.iter().zip(&all) {
        assert_eq!(n1, n2);
        assert_eq!(*png, canvas.png().expect("encodes"));
    }
}

#[test]
fn a_ratio_only_chart_with_one_party_draws_its_ratio_against_the_other() {
    let mut c = chart(&["OURS"], &[&[]]);
    c.groups[0].rows[0].ratio = Some(3.0);
    let out = pito_compare::render(&c, Kind::Ratio).expect("renders");
    assert_eq!(names(&out), ["ratio"]);
    let err = pito_compare::render(&c, Kind::Bars)
        .err()
        .expect("no values to draw as bars");
    assert!(err.message().contains("no values"), "{err}");
}

#[test]
fn bars_files_are_named_by_slug_and_fall_back_to_the_party_number() {
    let c = chart(
        &["C++", "c", "C+", "%%", "OURS (V2)"],
        &[&[1.0, 2.0, 3.0, 4.0, 5.0]],
    );
    let out = pito_compare::render(&c, Kind::Bars).expect("renders");
    assert_eq!(
        names(&out),
        ["bars-c", "bars-2", "bars-3", "bars-4", "bars-ours-v2"]
    );
}

#[test]
fn bars_file_names_never_repeat() {
    for parties in [
        &["2", "%%"][..],
        &["3", "A", "3."],
        &["%%", "1"],
        &["2", "2-2", "%%"],
        &["2", "%%", "2 2"],
    ] {
        let values: Vec<f64> = (1..=parties.len()).map(|v| v as f64).collect();
        let c = chart(parties, &[&values]);
        let out = pito_compare::render(&c, Kind::Bars).expect("renders");
        let mut n = names(&out);
        assert_eq!(n.len(), parties.len(), "{parties:?}");
        n.sort();
        n.dedup();
        assert_eq!(n.len(), parties.len(), "{parties:?}: {:?}", names(&out));
    }
    let c = chart(&["2", "%%"], &[&[1.0, 2.0]]);
    let out = pito_compare::render(&c, Kind::Bars).expect("renders");
    assert_eq!(names(&out), ["bars-2", "bars-2-2"]);
}

#[test]
fn render_each_stops_at_the_first_error_and_returns_it() {
    let c = chart(&["A", "B", "C"], &[&[1.0, 2.0, 3.0]]);
    let mut calls = 0;
    let err = pito_compare::render_each(&c, Kind::Bars, |_, _| {
        calls += 1;
        if calls == 2 {
            Err(Error::new("disk full"))
        } else {
            Ok(())
        }
    })
    .expect_err("the closure fails");
    assert_eq!(calls, 2);
    assert_eq!(err, Error::new("disk full"));
}

#[test]
fn an_invalid_chart_never_reaches_the_closure() {
    let c = chart(&["A", "B"], &[&[1.0]]);
    let mut called = false;
    let err = pito_compare::render_each(&c, Kind::All, |_, _| {
        called = true;
        Ok(())
    })
    .expect_err("values do not match parties");
    assert!(!called);
    assert!(err.message().contains("values"), "{err}");
}

#[test]
fn error_message_and_display_agree() {
    let e = Error::new("row 1 is wrong");
    assert_eq!(e.message(), "row 1 is wrong");
    assert_eq!(e.to_string(), e.message());
    let boxed: Box<dyn std::error::Error> = Box::new(e.clone());
    assert_eq!(boxed.to_string(), "row 1 is wrong");
}

#[test]
fn the_canvas_grows_by_a_row_for_each_row_and_shows_each_party_colour() {
    let one = paired(&chart(&["A", "B"], &[&[3.0, 2.0]])).expect("draws");
    let two = paired(&chart(&["A", "B"], &[&[3.0, 2.0], &[1.0, 4.0]])).expect("draws");
    assert_eq!(two.height() - one.height(), ROW as usize);
    let c = chart(&["A", "B"], &[&[3.0, 2.0]]);
    for (i, p) in c.parties.iter().enumerate() {
        let y = 250 + 32 + 32 * i + 10;
        assert_eq!(one.pixel(LEFT as usize + 1, y), Some(p.colour), "party {i}");
    }
}

#[test]
fn a_pixel_out_of_range_is_none() {
    let c = Canvas::new(4, 3);
    assert_eq!(c.pixel(3, 2), Some(BG));
    assert_eq!(c.pixel(4, 0), None);
    assert_eq!(c.pixel(0, 3), None);
    assert_eq!(c.pixel(usize::MAX, usize::MAX), None);
}

#[test]
fn the_png_decodes_back_to_the_canvas() {
    let mut c = Canvas::new(7, 5);
    c.rect(1, 1, 2, 2, FASTER);
    let bytes = c.png().expect("encodes");
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    let mut reader = png::Decoder::new(std::io::Cursor::new(&bytes))
        .read_info()
        .expect("a PNG");
    let mut buf = vec![0; reader.output_buffer_size().expect("a size")];
    let info = reader.next_frame(&mut buf).expect("a frame");
    assert_eq!((info.width, info.height), (7, 5));
    assert_eq!(info.color_type, png::ColorType::Rgb);
    let at = (7 + 1) * 3;
    assert_eq!(buf[at..at + 3], FASTER);
    assert_eq!(buf[..3], BG);
}

#[test]
fn save_writes_the_png_and_makes_its_folder() {
    let dir = scratch("library-save");
    let path = dir.join("deep/er/chart.png");
    let c = Canvas::new(3, 2);
    c.save(&path).expect("saves");
    assert_eq!(
        std::fs::read(&path).expect("written"),
        c.png().expect("encodes")
    );
}

#[test]
fn save_refuses_a_path_under_a_file() {
    let dir = scratch("library-under-file");
    std::fs::write(dir.join("file"), "x").expect("write");
    let err = Canvas::new(2, 2)
        .save(&dir.join("file/chart.png"))
        .expect_err("cannot make a folder in a file");
    assert!(err.message().starts_with("cannot create "), "{err}");
}

#[cfg(unix)]
#[test]
fn save_refuses_a_symlink_target() {
    let dir = scratch("library-link");
    let target = dir.join("keep.txt");
    std::fs::write(&target, "keep").expect("write");
    let link = dir.join("chart.png");
    std::os::unix::fs::symlink(&target, &link).expect("link");
    let err = Canvas::new(2, 2).save(&link).expect_err("refused");
    assert!(err.message().contains("symlink"), "{err}");
    assert_eq!(std::fs::read_to_string(&target).expect("read"), "keep");
}

#[test]
fn the_ratio_tag_takes_the_tone_of_the_comparison() {
    let band = |c: &Canvas, colour| has(c, 700..1040, 250..271, colour);
    let faster = paired(&chart(&["A", "B"], &[&[2.0, 1.0]])).expect("draws");
    assert!(band(&faster, FASTER) && !band(&faster, SLOWER));
    let slower = paired(&chart(&["A", "B"], &[&[1.0, 2.0]])).expect("draws");
    assert!(band(&slower, SLOWER) && !band(&slower, FASTER));
    let lower = paired(&chart_by(Better::Lower, &["A", "B"], &[&[1.0, 2.0]])).expect("draws");
    assert!(band(&lower, FASTER) && !band(&lower, SLOWER));
    let tie = paired(&chart(&["A", "B"], &[&[5.0, 5.0]])).expect("draws");
    assert!(band(&tie, DIM) && !band(&tie, FASTER) && !band(&tie, SLOWER));
}

#[test]
fn a_label_too_long_for_its_tag_pushes_the_tag_below_it() {
    let mut c = chart(&["A", "B"], &[&[2.0, 1.0]]);
    c.groups[0].rows[0].label = "A VERY LONG ROW LABEL THAT TAKES MOST OF THE LINE".into();
    let drawn = paired(&c).expect("draws");
    assert!(!has(&drawn, 700..1040, 250..271, FASTER));
    assert!(has(&drawn, 700..1040, 284..305, FASTER));
    let short = paired(&chart(&["A", "B"], &[&[2.0, 1.0]])).expect("draws");
    assert_eq!(drawn.height(), short.height());
}

#[test]
fn a_long_suffix_or_text_shortens_the_bars() {
    let plain = chart(&["A", "B"], &[&[2.0, 1.0]]);
    let mut suffixed = plain.clone();
    suffixed.suffix = " FRAMES PER SECOND".into();
    suffixed.decimals = Some(2);
    let mut texted = plain.clone();
    texted.groups[0].rows[0].texts = vec!["TWO MEGABYTES A SECOND".into(), "ONE".into()];
    let y = 250 + 32 + 10;
    let colour = plain.parties[0].colour;
    let long = bar_length(&paired(&plain).expect("draws"), y, colour);
    assert!(long > 600, "{long}");
    for c in [&suffixed, &texted] {
        let short = bar_length(&paired(c).expect("draws"), y, colour);
        assert!(short < long, "{short} vs {long}");
        assert!(short >= 300, "{short}");
    }
}

#[test]
fn texts_too_long_to_sit_beside_the_bars_are_refused() {
    let mut c = chart(&["A", "B"], &[&[2.0, 1.0]]);
    c.groups[0].rows[0].texts = vec![
        "A VALUE TEXT FAR TOO LONG TO SIT BESIDE ITS BAR".into(),
        "B".into(),
    ];
    let err = paired(&c).err().expect("too long");
    assert!(err.message().contains("too long to fit"), "{err}");
}

#[test]
fn a_zero_value_draws_as_bars_but_has_no_ratio() {
    let c = chart(&["A", "B"], &[&[0.0, 3.0]]);
    let drawn = paired(&c).expect("draws");
    let y = 250 + 32 + 10;
    assert_eq!(bar_length(&drawn, y, c.parties[0].colour), 2);
    assert!(!has(&drawn, 700..1040, 250..271, FASTER));
    let err = ratios(&c).err().expect("no ratio");
    assert!(
        err.message().contains("row 1 (ROW 0) has no ratio"),
        "{err}"
    );
    let all = pito_compare::render(&chart(&["A"], &[&[0.0]]), Kind::All).expect("all zero draws");
    assert_eq!(names(&all), ["bars"]);
}

#[test]
fn small_fractional_values_draw_an_axis_with_labels() {
    let band = 250 - 40..250 - 40 + 14;
    for v in [&[0.3, 0.1][..], &[0.0004, 0.0001], &[1e-300, 5e-301]] {
        let drawn = paired(&chart(&["A", "B"], &[v])).expect("draws");
        assert_eq!(drawn.width(), WIDTH as usize);
        assert!(
            has(
                &drawn,
                LEFT as usize + 40..WIDTH as usize,
                band.clone(),
                DIM
            ),
            "{v:?}"
        );
    }
}

#[test]
fn a_value_too_small_for_an_axis_is_refused() {
    let err = bars(&chart(&["A"], &[&[5e-324]]), 0)
        .err()
        .expect("no axis");
    assert!(err.message().contains("an axis is drawn up to"), "{err}");
}

#[test]
fn a_chart_too_tall_is_refused() {
    let rows: Vec<&[f64]> = vec![&[2.0, 1.0]; 300];
    let c = chart(&["A", "B"], &rows);
    let err = paired(&c).err().expect("too tall");
    assert!(
        err.message().contains(&format!("over {MAX_HEIGHT}")),
        "{err}"
    );
    assert!(ratios(&c).is_ok());
    let rows: Vec<&[f64]> = vec![&[2.0, 1.0]; 500];
    let err = ratios(&chart(&["A", "B"], &rows)).err().expect("too tall");
    assert!(err.message().contains("px tall"), "{err}");
}

#[test]
fn a_wide_legend_wraps_and_makes_the_chart_taller() {
    let short: Vec<String> = (0..12).map(|i| format!("P{i}")).collect();
    let long: Vec<String> = (0..12).map(|i| format!("PARTY NUMBER {i:02}")).collect();
    let height = |names: &[String]| {
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let values: Vec<f64> = (1..=12).map(f64::from).collect();
        paired(&chart(&names, &[&values])).expect("draws").height()
    };
    assert!(height(&long) > height(&short) + 100);
}

#[test]
fn a_ratio_row_without_values_is_refused_as_bars() {
    let mut c = chart(&["A", "B"], &[&[2.0, 1.0], &[]]);
    c.groups[0].rows[1].ratio = Some(2.0);
    let err = paired(&c).err().expect("no values");
    assert_eq!(err.message(), "row 2 (ROW 1) has no values to draw as bars");
    assert!(ratios(&c).is_ok());
}

#[test]
fn single_party_charts_refuse_what_they_cannot_draw() {
    let one = chart(&["A"], &[&[2.0]]);
    assert!(
        paired(&one)
            .err()
            .expect("one party")
            .message()
            .contains("two parties")
    );
    assert!(
        ratios(&one)
            .err()
            .expect("one party")
            .message()
            .contains("a ratio on every row")
    );
    let err = bars(&one, 1).err().expect("no party 1");
    assert_eq!(err.message(), "there is no party 1; the chart has 1");
}

#[test]
fn ratio_bars_go_right_when_better_and_left_when_worse() {
    let c = chart(&["A", "B"], &[&[2.0, 1.0], &[1.0, 2.0]]);
    let drawn = ratios(&c).expect("draws");
    let cx = (WIDTH / 2) as usize;
    assert_eq!(drawn.pixel(cx + 20, 270 + 35), Some(FASTER));
    assert_eq!(drawn.pixel(cx - 20, 336 + 35), Some(SLOWER));
    assert_ne!(drawn.pixel(cx - 20, 270 + 35), Some(FASTER));
}

#[test]
fn a_huge_spread_is_clamped_to_the_axis() {
    let c = chart(&["A", "B"], &[&[1e6, 1.0], &[1.0, 1e6]]);
    let drawn = ratios(&c).expect("draws");
    let cx = WIDTH / 2;
    let half = cx - LEFT - 90;
    assert_eq!(
        drawn.pixel((cx + half - 5) as usize, 270 + 35),
        Some(FASTER)
    );
    assert_eq!(drawn.pixel((cx + half + 5) as usize, 270 + 35), Some(BG));
    assert_eq!(
        drawn.pixel((cx - half + 5) as usize, 336 + 35),
        Some(SLOWER)
    );
}

#[test]
fn ratio_captions_and_unit_line_that_do_not_fit_are_refused() {
    let mut c = chart(&["A", "B"], &[&[2.0, 1.0]]);
    c.words.better = "MUCH MUCH MUCH MUCH MUCH MUCH MUCH FASTER".into();
    let err = ratios(&c).err().expect("captions too long");
    assert!(err.message().contains("captions are too long"), "{err}");
    c.words.better = "A GREAT DEAL QUICKER NOW".into();
    c.words.worse = "A GREAT DEAL SLOWER NOW".into();
    let err = ratios(&c).err().expect("unit too long");
    assert!(err.message().contains("unit line is too long"), "{err}");
    assert!(paired(&c).is_ok());
}

#[test]
fn an_empty_canvas_cannot_become_a_png() {
    let empty = Canvas::new(0, 0);
    let err = empty.png().expect_err("no pixels");
    assert!(err.message().starts_with("cannot write the PNG: "), "{err}");
    let dir = scratch("library-empty");
    let path = dir.join("empty.png");
    assert!(empty.save(&path).is_err());
    assert!(!path.exists());
}

#[test]
fn save_refuses_a_path_that_is_a_folder() {
    let c = Canvas::new(2, 2);
    let dir = scratch("library-folder");
    let err = c.save(&dir).expect_err("a folder");
    assert!(err.message().starts_with("cannot write "), "{err}");
    let err = c.save(Path::new("/")).expect_err("the root");
    assert!(err.message().starts_with("cannot write "), "{err}");
}
