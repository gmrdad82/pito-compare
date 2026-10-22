use std::path::{Path, PathBuf};

use pito_compare::canvas::{DIM, FASTER, OURS, SLOWER, TEXT, THEIRS};
use pito_compare::draw::{MAX_PARTIES, MAX_VALUE};
use pito_compare::{Better, Chart, Format, Group, Row, Words};

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("chart").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn chart() -> Chart {
    let mut c = Chart::new("Decode", "ms", Better::Lower, &["ours", "theirs"]);
    c.groups.push(Group {
        name: "Group".into(),
        rows: vec![Row::new("clip", &[1.0, 2.0])],
    });
    c
}

fn refusal(c: &Chart) -> String {
    c.check().expect_err("check refuses").message().to_string()
}

fn refused_with(c: &Chart, needle: &str) {
    let m = refusal(c);
    assert!(m.contains(needle), "{needle:?} not in {m:?}");
}

fn parse_err(text: &str, format: Format) -> String {
    Chart::parse(text, format)
        .expect_err("parse refuses")
        .message()
        .to_string()
}

const FULL_JSON: &str = r##"{
  "title": "Decode speed",
  "unit": "ms",
  "better": "lower",
  "subtitle": "1080p clips",
  "date": "2026-10-22",
  "parties": [
    {"name": "pcod", "colour": "#0A0b0C"},
    {"name": "ffmpeg", "color": "theirs"},
    {"name": "c", "colour": "OURS"},
    {"name": "d", "colour": "dim"},
    {"name": "e", "colour": "text"},
    {"name": "f", "colour": "faster"},
    {"name": "g", "colour": "slower"},
    {"name": "h"}
  ],
  "versus": "FFMPEG",
  "groups": [
    {"name": "H264", "rows": [
      {"label": "first frame", "mark": "*", "values": [1, 2, 3, 4, 5, 6, 7, 8],
       "texts": ["1", "2", "3", "4", "5", "6", "7", "8"]},
      {"label": "seek", "ratio": 3.5}
    ]},
    {"rows": [{"label": "plain", "values": [1, 2, 3, 4, 5, 6, 7, 8]}]}
  ],
  "notes": ["a note", "another"],
  "suffix": " ms",
  "decimals": 6,
  "words": {"better": "QUICKER", "worse": "LAGS", "same": "TIE"}
}"##;

#[test]
fn json_loads_every_optional_field() {
    let c = Chart::parse(FULL_JSON, Format::Json).expect("parses");
    assert_eq!(c.title, "Decode speed");
    assert_eq!(c.unit, "ms");
    assert_eq!(c.better, Better::Lower);
    assert_eq!(c.subtitle, "1080p clips");
    assert_eq!(c.date, "2026-10-22");
    let colours: Vec<_> = c.parties.iter().map(|p| p.colour).collect();
    assert_eq!(
        colours,
        vec![[10, 11, 12], THEIRS, OURS, DIM, TEXT, FASTER, SLOWER, TEXT]
    );
    assert_eq!(c.versus, 1);
    assert_eq!(c.groups.len(), 2);
    assert_eq!(c.groups[0].name, "H264");
    assert_eq!(c.groups[1].name, "");
    let first = &c.groups[0].rows[0];
    assert_eq!(first.mark, "*");
    assert_eq!(first.shown_label(), "first frame *");
    assert_eq!(first.texts.len(), 8);
    assert_eq!(c.groups[0].rows[1].ratio, Some(3.5));
    assert!(c.groups[0].rows[1].values.is_empty());
    assert_eq!(c.notes, vec!["a note", "another"]);
    assert_eq!(c.suffix, " ms");
    assert_eq!(c.decimals, Some(6));
    assert_eq!(
        c.words,
        Words {
            better: "QUICKER".into(),
            worse: "LAGS".into(),
            same: "TIE".into()
        }
    );
    assert_eq!(c.rows().count(), 3);
}

#[test]
fn toml_loads_flat_rows_into_one_unnamed_group_with_defaults() {
    let text = r##"
title = "Encode"
better = "higher"
rows = [
  { label = "a", values = [10.0, 5.0] },
  { label = "b", values = [1.0, 4.0] },
]

[[parties]]
name = "ours"

[[parties]]
name = "theirs"
color = "#FFFFFF"

[words]
same = "EVEN"
"##;
    let c = Chart::parse(text, Format::Toml).expect("parses");
    assert_eq!(c.better, Better::Higher);
    assert_eq!(c.unit, "");
    assert_eq!(c.subtitle, "");
    assert_eq!(c.date, "");
    assert_eq!(c.parties[0].colour, OURS);
    assert_eq!(c.parties[1].colour, [255, 255, 255]);
    assert_eq!(c.versus, 1);
    assert_eq!(c.groups.len(), 1);
    assert_eq!(c.groups[0].name, "");
    assert_eq!(c.groups[0].rows.len(), 2);
    assert_eq!(c.decimals, None);
    assert_eq!(c.words.better, "FASTER");
    assert_eq!(c.words.worse, "SLOWER");
    assert_eq!(c.words.same, "EVEN");
}

#[test]
fn parse_defaults_cycle_colour_roles_past_four_parties() {
    let text = r#"{"title":"T","better":"higher","parties":[{"name":"a"},{"name":"b"},{"name":"c"},{"name":"d"},{"name":"e"},{"name":"f"}],"rows":[{"label":"r","values":[1,1,1,1,1,1]}]}"#;
    let c = Chart::parse(text, Format::Json).expect("parses");
    let colours: Vec<_> = c.parties.iter().map(|p| p.colour).collect();
    assert_eq!(colours, vec![OURS, THEIRS, DIM, TEXT, OURS, THEIRS]);
}

#[test]
fn load_reads_by_extension_and_sniffs_the_rest() {
    let dir = scratch("load");
    let json = r#"{"title":"T","better":"lower","parties":[{"name":"a"}],"rows":[{"label":"r","values":[1]}]}"#;
    let toml = "title = \"T\"\nbetter = \"lower\"\nparties = [{ name = \"a\" }]\nrows = [{ label = \"r\", values = [1.0] }]\n";
    for (name, text) in [
        ("a.json", json),
        ("b.JSON", json),
        ("c.toml", toml),
        ("d.Toml", toml),
        ("e.txt", json),
        ("f", toml),
        ("g.chart", toml),
    ] {
        let path = dir.join(name);
        std::fs::write(&path, text).expect("writes");
        let c = Chart::load(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(c.title, "T", "{name}");
        assert_eq!(c.parties.len(), 1, "{name}");
    }
}

#[test]
fn load_refuses_a_missing_file_naming_it() {
    let path = scratch("missing").join("nope.json");
    let m = Chart::load(&path).expect_err("missing").to_string();
    assert!(m.starts_with("cannot read "), "{m}");
    assert!(m.contains("nope.json"), "{m}");
}

#[test]
fn load_prefixes_parse_errors_with_the_path() {
    let dir = scratch("broken");
    let path = dir.join("broken.json");
    std::fs::write(&path, "{\"title\": ").expect("writes");
    let m = Chart::load(&path).expect_err("broken").to_string();
    assert!(m.starts_with(&format!("{}: ", path.display())), "{m}");
    let path = dir.join("broken.toml");
    std::fs::write(&path, "title = \"T\"\nbetter = \"lower\"\nbogus = 1\n").expect("writes");
    let m = Chart::load(&path).expect_err("broken").to_string();
    assert!(m.starts_with(&format!("{}: ", path.display())), "{m}");
    assert!(m.contains("at line 3 column 1"), "{m}");
}

#[test]
fn a_toml_extension_does_not_sniff_json() {
    let dir = scratch("nosniff");
    let path = dir.join("x.toml");
    std::fs::write(
        &path,
        r#"{"title":"T","better":"lower","parties":[{"name":"a"}],"rows":[{"label":"r","values":[1]}]}"#,
    )
    .expect("writes");
    assert!(Chart::load(&path).is_err());
}

#[test]
fn broken_json_is_refused() {
    assert!(!parse_err("{\"title\": ", Format::Json).is_empty());
    assert!(parse_err("{\"title\":\"T\"}", Format::Json).contains("better"));
    assert!(
        parse_err(
            r#"{"title":"T","better":"sideways","parties":[],"rows":[]}"#,
            Format::Json
        )
        .contains("sideways")
    );
    assert!(
        parse_err(
            r#"{"title":"T","better":"lower","parties":[{"name":"a","shade":"x"}]}"#,
            Format::Json
        )
        .contains("shade")
    );
}

#[test]
fn broken_toml_names_its_line_and_column() {
    let m = parse_err("title = \"T\"\nbetter = \"lower\"\nbogus = 1\n", Format::Toml);
    assert!(m.contains("bogus"), "{m}");
    assert!(m.ends_with("at line 3 column 1"), "{m}");
    let m = parse_err("title = \"T\"\n\nbetter = = 1\n", Format::Toml);
    assert!(m.contains("at line 3 column "), "{m}");
}

#[test]
fn rows_and_groups_together_are_refused() {
    let text = r#"{"title":"T","better":"lower","parties":[{"name":"a"}],
        "rows":[{"label":"r","values":[1]}],
        "groups":[{"rows":[{"label":"s","values":[1]}]}]}"#;
    assert_eq!(parse_err(text, Format::Json), "give rows or groups, not both");
}

fn with_decimals(d: usize) -> String {
    format!(
        r#"{{"title":"T","better":"lower","parties":[{{"name":"a"}}],"decimals":{d},"rows":[{{"label":"r","values":[1]}}]}}"#
    )
}

#[test]
fn decimals_stop_at_six() {
    assert_eq!(
        Chart::parse(&with_decimals(6), Format::Json)
            .expect("six")
            .decimals,
        Some(6)
    );
    assert_eq!(
        parse_err(&with_decimals(7), Format::Json),
        "decimals is at most 6"
    );
}

#[test]
fn bad_colours_are_refused_naming_the_party() {
    for bad in ["#GGGGGG", "#FFF", "#FFFFFFF", "FFFFFF", "purple", "", "#"] {
        let text = format!(
            r#"{{"title":"T","better":"lower","parties":[{{"name":"a"}},{{"name":"b","colour":"{bad}"}}],"rows":[{{"label":"r","values":[1,2]}}]}}"#
        );
        let m = parse_err(&text, Format::Json);
        assert!(m.starts_with("parties[1].colour is "), "{bad}: {m}");
        assert!(m.contains("#RRGGBB"), "{bad}: {m}");
    }
}

#[test]
fn versus_by_name_and_its_refusals() {
    let base = |versus: &str| {
        format!(
            r#"{{"title":"T","better":"lower","parties":[{{"name":"a"}},{{"name":"b"}},{{"name":"c"}}],"versus":"{versus}","rows":[{{"label":"r","values":[1,2,3]}}]}}"#
        )
    };
    assert_eq!(
        Chart::parse(&base("C"), Format::Json)
            .expect("names c")
            .versus,
        2
    );
    assert!(parse_err(&base("z"), Format::Json).contains("versus names \"z\""));
    assert!(parse_err(&base("a"), Format::Json).starts_with("versus must name"));
}

#[test]
fn toml_values_run_through_check() {
    let text = "title = \"T\"\nbetter = \"lower\"\nparties = [{ name = \"a\" }, { name = \"b\" }]\nrows = [{ label = \"r\", values = [inf, 1.0] }]\n";
    assert_eq!(
        parse_err(text, Format::Toml),
        "groups[0].rows[0].values holds inf: values are finite and zero or more"
    );
    let text = text.replace("inf", "nan");
    assert!(parse_err(&text, Format::Toml).contains("values holds NaN"));
}

#[test]
fn the_base_chart_passes() {
    chart().check().expect("valid");
}

#[test]
fn title_refusals() {
    let mut c = chart();
    c.title = "  ".into();
    assert_eq!(refusal(&c), "title is empty");
    c.title = "A".repeat(41);
    c.check().expect("41 fit");
    c.title = "A".repeat(42);
    assert_eq!(refusal(&c), "title is too long: at most 41 characters fit");
    c.title = "Décode".into();
    refused_with(&c, "title: the chart font has no 'é'");
}

#[test]
fn unit_refusals() {
    let mut c = chart();
    c.unit = "A".repeat(55 - ", LOWER IS BETTER".len());
    c.check().expect("unit fits");
    c.unit.push('A');
    assert_eq!(refusal(&c), "unit is too long: at most 55 characters fit");
    c.unit = "µs".into();
    refused_with(&c, "unit: the chart font has no");
}

#[test]
fn subtitle_and_date_refusals() {
    let mut c = chart();
    c.subtitle = "A".repeat(40);
    c.date = "B".repeat(41);
    c.check().expect("83 fit");
    c.date.push('B');
    assert_eq!(
        refusal(&c),
        "subtitle and date is too long: at most 83 characters fit"
    );
    c.date = "2026—10".into();
    refused_with(&c, "subtitle and date: the chart font has no");
}

#[test]
fn party_count_refusals_and_the_boundary() {
    let mut c = chart();
    c.parties.clear();
    assert_eq!(refusal(&c), "parties is empty: name one party or more");
    let names: Vec<String> = (0..=MAX_PARTIES).map(|i| format!("P{i}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut c = Chart::new("T", "", Better::Higher, &refs[..MAX_PARTIES]);
    c.groups.push(Group {
        name: String::new(),
        rows: vec![Row::new("r", &[1.0; MAX_PARTIES])],
    });
    c.check().expect("MAX_PARTIES fit");
    let mut c = Chart::new("T", "", Better::Higher, &refs);
    c.groups.push(Group {
        name: String::new(),
        rows: vec![Row::new("r", &[1.0; MAX_PARTIES + 1])],
    });
    assert_eq!(
        refusal(&c),
        format!("parties has 13 parties: a chart draws at most {MAX_PARTIES}")
    );
}

#[test]
fn party_name_refusals() {
    let mut c = chart();
    c.parties[1].name = " ".into();
    assert_eq!(refusal(&c), "parties[1].name is empty");
    c.parties[1].name = "Ärger".into();
    refused_with(&c, "parties[1].name: the chart font has no 'Ä'");
    c.parties[1].name = "A".repeat(53);
    c.check().expect("53 fit the legend");
    c.parties[1].name = "A".repeat(54);
    assert_eq!(refusal(&c), "parties[1].name is too long for the legend");
}

#[test]
fn versus_refusals() {
    let mut c = chart();
    c.versus = 0;
    refused_with(&c, "versus must name a party other than the first");
    c.versus = 2;
    refused_with(&c, "versus must name");
    let mut single = Chart::new("T", "", Better::Lower, &["solo"]);
    single.groups.push(Group {
        name: String::new(),
        rows: vec![Row::new("r", &[3.0])],
    });
    assert_eq!(single.versus, 1);
    single.check().expect("one party ignores versus");
    assert_eq!(single.ratio(&single.groups[0].rows[0]), None);
}

#[test]
fn words_refusals() {
    let mut c = chart();
    c.words.better = String::new();
    assert_eq!(refusal(&c), "words.better is empty");
    let mut c = chart();
    c.words.worse = "  ".into();
    assert_eq!(refusal(&c), "words.worse is empty");
    let mut c = chart();
    c.words.same = "ÉGAL".into();
    refused_with(&c, "words.same: the chart font has no");
}

#[test]
fn row_and_group_refusals() {
    let mut c = chart();
    c.groups.clear();
    assert_eq!(refusal(&c), "there are no rows to draw");
    let mut c = chart();
    c.groups.push(Group::default());
    assert_eq!(refusal(&c), "groups[1] has no rows");
    let mut c = chart();
    c.groups[0].name = "A".repeat(84);
    refused_with(&c, "groups[0].name is too long");
    let mut c = chart();
    c.groups[0].rows[0].label = "A".repeat(54);
    c.groups[0].rows[0].mark = "*".into();
    refused_with(&c, "groups[0].rows[0].label is too long");
    let mut c = chart();
    c.groups[0].rows.push(Row::new("short", &[1.0]));
    assert_eq!(
        refusal(&c),
        "groups[0].rows[1].values has 1 values for 2 parties"
    );
    let mut c = chart();
    c.groups[0].rows[0].texts = vec!["1".into()];
    assert_eq!(
        refusal(&c),
        "groups[0].rows[0].texts has 1 texts for 2 parties"
    );
    let mut c = chart();
    c.groups[0].rows[0].texts = vec!["1".into(), "€".into()];
    refused_with(&c, "groups[0].rows[0].texts: the chart font has no");
    let mut c = chart();
    c.groups[0].rows[0] = Row::new("empty", &[]);
    assert_eq!(
        refusal(&c),
        "groups[0].rows[0] has neither values nor a ratio"
    );
}

#[test]
fn value_refusals_and_the_max() {
    for (bad, shown) in [
        (-1.0, "-1"),
        (f64::NAN, "NaN"),
        (f64::INFINITY, "inf"),
        (f64::NEG_INFINITY, "-inf"),
    ] {
        let mut c = chart();
        c.groups[0].rows[0].values[1] = bad;
        assert_eq!(
            refusal(&c),
            format!("groups[0].rows[0].values holds {shown}: values are finite and zero or more")
        );
    }
    let mut c = chart();
    c.groups[0].rows[0].values = vec![MAX_VALUE, MAX_VALUE];
    c.check().expect("MAX_VALUE fits");
    c.groups[0].rows[0].values = vec![MAX_VALUE, 0.0];
    c.check().expect("zero is no ratio");
    c.groups[0].rows[0].values = vec![2e15, 2e15];
    assert_eq!(
        refusal(&c),
        "groups[0].rows[0].values holds 2e15: values are at most 1e15"
    );
}

#[test]
fn given_ratio_refusals() {
    for (bad, shown) in [
        (0.0, "0"),
        (-2.0, "-2"),
        (f64::NAN, "NaN"),
        (f64::INFINITY, "inf"),
    ] {
        let mut c = chart();
        c.groups[0].rows[0].ratio = Some(bad);
        assert_eq!(
            refusal(&c),
            format!("groups[0].rows[0].ratio is {shown}: a ratio is finite and above zero")
        );
    }
    for bad in [1e16, 1e-16] {
        let mut c = chart();
        c.groups[0].rows[0].ratio = Some(bad);
        let m = refusal(&c);
        assert!(m.starts_with("groups[0].rows[0].ratio is "), "{m}");
        assert!(m.contains("a ratio is between 1e-15 and 1e15"), "{m}");
    }
    for ok in [MAX_VALUE, 1.0 / MAX_VALUE] {
        let mut c = chart();
        c.groups[0].rows[0].ratio = Some(ok);
        c.check().expect("boundary ratio fits");
    }
}

#[test]
fn computed_ratio_refusals() {
    let mut c = chart();
    c.groups[0].rows[0].values = vec![0.5, 1e15];
    assert_eq!(
        refusal(&c),
        "groups[0].rows[0].values give a ratio of 2e15: a ratio is between 1e-15 and 1e15"
    );
    c.better = Better::Higher;
    let m = refusal(&c);
    assert!(
        m.starts_with("groups[0].rows[0].values give a ratio of 5e-16"),
        "{m}"
    );
    c.groups[0].rows[0].values = vec![1.0, 1e15];
    c.check().expect("exactly 1e-15 fits");
}

#[test]
fn suffix_and_note_refusals() {
    let mut c = chart();
    c.suffix = "°".into();
    refused_with(&c, "suffix: the chart font has no");
    let mut c = chart();
    c.notes = vec!["fine".into(), "ü".into()];
    refused_with(&c, "notes[1]: the chart font has no");
    let mut c = chart();
    c.notes = vec![format!("ok {}", "A".repeat(55))];
    c.check().expect("55 fit a line");
    c.notes = vec![format!("ok {}", "A".repeat(56))];
    refused_with(&c, "notes[0] has a word longer than a line (55 characters)");
}

#[test]
fn new_builds_defaults() {
    let c = Chart::new("T", "u", Better::Higher, &["a", "b", "c", "d", "e"]);
    assert_eq!(c.title, "T");
    assert_eq!(c.unit, "u");
    assert_eq!(c.versus, 1);
    assert!(c.groups.is_empty() && c.notes.is_empty());
    assert_eq!(c.decimals, None);
    assert_eq!(c.words, Words::default());
    let colours: Vec<_> = c.parties.iter().map(|p| p.colour).collect();
    assert_eq!(colours, vec![OURS, THEIRS, DIM, TEXT, OURS]);
    assert_eq!(c.parties[4].name, "e");
    assert_eq!(refusal(&c), "there are no rows to draw");
}

#[test]
fn row_new_and_shown_label() {
    let mut r = Row::new("seek", &[1.0, 2.5]);
    assert_eq!(r.values, vec![1.0, 2.5]);
    assert!(r.mark.is_empty() && r.texts.is_empty() && r.ratio.is_none());
    assert_eq!(r.shown_label(), "seek");
    r.mark = "†".into();
    assert_eq!(r.shown_label(), "seek †");
}

#[test]
fn rows_flattens_groups_in_order() {
    let mut c = chart();
    c.groups.push(Group {
        name: "Two".into(),
        rows: vec![Row::new("x", &[1.0, 1.0]), Row::new("y", &[1.0, 1.0])],
    });
    let labels: Vec<_> = c.rows().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, vec!["clip", "x", "y"]);
}

#[test]
fn ratio_says_how_much_faster_ours_is() {
    let mut c = chart();
    let lower_ours_wins = Row::new("r", &[100.0, 200.0]);
    assert_eq!(c.ratio(&lower_ours_wins), Some(2.0));
    let lower_ours_loses = Row::new("r", &[200.0, 100.0]);
    assert_eq!(c.ratio(&lower_ours_loses), Some(0.5));
    c.better = Better::Higher;
    assert_eq!(c.ratio(&lower_ours_wins), Some(0.5));
    assert_eq!(c.ratio(&lower_ours_loses), Some(2.0));
    assert_eq!(c.ratio(&Row::new("r", &[0.0, 1.0])), None);
    assert_eq!(c.ratio(&Row::new("r", &[1.0, 0.0])), None);
    assert_eq!(c.ratio(&Row::new("r", &[])), None);
    let mut given = Row::new("r", &[1.0, 9.0]);
    given.ratio = Some(4.0);
    assert_eq!(c.ratio(&given), Some(4.0));
}

#[test]
fn ratio_compares_against_versus() {
    let mut c = Chart::new("T", "", Better::Lower, &["a", "b", "c"]);
    let row = Row::new("r", &[1.0, 2.0, 8.0]);
    assert_eq!(c.ratio(&row), Some(2.0));
    c.versus = 2;
    assert_eq!(c.ratio(&row), Some(8.0));
}

#[test]
fn unit_and_subtitle_lines() {
    let mut c = chart();
    assert_eq!(c.unit_line(), "ms, LOWER IS BETTER");
    c.unit.clear();
    c.better = Better::Higher;
    assert_eq!(c.unit_line(), "HIGHER IS BETTER");
    assert_eq!(c.subtitle_line(), "");
    c.date = "2026".into();
    assert_eq!(c.subtitle_line(), "2026");
    c.subtitle = "clips".into();
    assert_eq!(c.subtitle_line(), "clips, 2026");
    c.date.clear();
    assert_eq!(c.subtitle_line(), "clips");
}

#[test]
fn value_text_prefers_texts_then_formats() {
    let mut c = chart();
    let mut r = Row::new("r", &[12.34, 150.6]);
    assert_eq!(c.value_text(&r, 0), "12.3");
    assert_eq!(c.value_text(&r, 1), "151");
    assert_eq!(c.value_text(&r, 5), "0.0");
    c.decimals = Some(2);
    c.suffix = "x".into();
    assert_eq!(c.value_text(&r, 0), "12.34x");
    r.texts = vec!["fast".into(), "slow".into()];
    assert_eq!(c.value_text(&r, 1), "slow");
}
