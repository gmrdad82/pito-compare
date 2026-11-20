use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FILES: [&str; 5] = [
    "SKILL.md",
    "data-file.md",
    "charts.md",
    "styling.md",
    "output.md",
];

const BASE: &str = "title = \"T\"\nbetter = \"higher\"\nparties = [{ name = \"A\" }, { name = \"B\" }]\nrows = [{ label = \"R\", values = [1, 2] }]\n";

const ONE: &str = "title = \"T\"\nbetter = \"higher\"\nparties = [{ name = \"A\" }]\nrows = [{ label = \"R\", values = [1] }]\n";

struct Fence {
    info: String,
    body: String,
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn source(name: &str) -> String {
    std::fs::read_to_string(root().join("skills/pcmp").join(name)).expect("a skill file")
}

fn everything() -> String {
    FILES
        .iter()
        .map(|f| source(f))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fresh(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("skill-drift-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn pcmp(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .args(args)
        .current_dir(dir)
        .env_clear()
        .env("HOME", dir.join("home"))
        .env("USERPROFILE", dir.join("home"))
        .output()
        .expect("pcmp runs")
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn parts(text: &str) -> (Vec<Fence>, String) {
    let mut fences = Vec::new();
    let mut prose = String::new();
    let mut open: Option<Fence> = None;
    for line in text.lines() {
        if let Some(info) = line.strip_prefix("```") {
            match open.take() {
                Some(done) => fences.push(done),
                None => {
                    open = Some(Fence {
                        info: info.trim().to_owned(),
                        body: String::new(),
                    });
                }
            }
        } else if let Some(fence) = open.as_mut() {
            fence.body.push_str(line);
            fence.body.push('\n');
        } else {
            prose.push_str(line);
            prose.push('\n');
        }
    }
    assert!(open.is_none(), "a code fence is left open");
    (fences, prose)
}

fn spans(prose: &str) -> Vec<String> {
    prose
        .lines()
        .flat_map(|line| {
            line.split('`')
                .skip(1)
                .step_by(2)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn snippets(text: &str) -> Vec<String> {
    let (fences, prose) = parts(text);
    let mut found = spans(&prose);
    for fence in fences.iter().filter(|f| f.info == "sh") {
        found.extend(fence.body.lines().map(str::to_owned));
    }
    found
}

fn all_snippets() -> Vec<String> {
    FILES.iter().flat_map(|f| snippets(&source(f))).collect()
}

fn is_command(snippet: &str) -> bool {
    snippet == "pcmp" || snippet.starts_with("pcmp ")
}

fn runnable(snippet: &str) -> bool {
    is_command(snippet) && !snippet.contains(['<', '[', '|'])
}

fn clean(token: &str) -> &str {
    token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-')
}

fn flags(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(clean)
        .filter(|t| {
            t.starts_with('-')
                && t.chars()
                    .nth(1)
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '-')
        })
        .map(str::to_owned)
        .collect()
}

fn names(help: &str, flag: &str) -> bool {
    let joined = |c: char| c.is_ascii_alphanumeric() || c == '-';
    help.match_indices(flag).any(|(at, _)| {
        !help[..at].chars().next_back().is_some_and(joined)
            && !help[at + flag.len()..].chars().next().is_some_and(joined)
    })
}

fn help(dir: &Path, args: &[&str]) -> String {
    let o = pcmp(dir, args);
    assert_eq!(o.status.code(), Some(0), "{args:?}");
    out(&o)
}

fn commands_of(help: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut in_commands = false;
    for line in help.lines() {
        if line == "Commands:" {
            in_commands = true;
        } else if in_commands && line.starts_with("  ") {
            if let Some(name) = line.split_whitespace().next().filter(|n| *n != "help") {
                found.push(name.to_owned());
            }
        } else {
            in_commands = false;
        }
    }
    found
}

fn kinds_of(help: &str) -> Vec<String> {
    let (_, rest) = help
        .split_once("[possible values: ")
        .expect("kinds in help");
    let (list, _) = rest.split_once(']').expect("closed list");
    list.split(", ").map(str::to_owned).collect()
}

fn all_help(dir: &Path) -> String {
    [
        help(dir, &["--help"]),
        help(dir, &["skill", "--help"]),
        help(dir, &["skill", "install", "--help"]),
        help(dir, &["skill", "status", "--help"]),
    ]
    .join("\n")
}

#[test]
fn every_flag_and_command_the_skill_names_is_in_the_help() {
    let dir = fresh("help");
    let helps = all_help(&dir);
    let top = help(&dir, &["--help"]);
    let skill = help(&dir, &["skill", "--help"]);
    let kinds = kinds_of(&top);
    let subcommands = commands_of(&skill);
    for snippet in all_snippets() {
        for flag in flags(&snippet) {
            assert!(
                names(&helps, &flag),
                "{flag} in `{snippet}` is not in the help"
            );
        }
        let words: Vec<&str> = snippet.split_whitespace().collect();
        for pair in words.windows(2) {
            if matches!(clean(pair[0]), "--kind" | "-k") && !pair[1].starts_with('<') {
                for kind in clean(pair[1]).split('|') {
                    assert!(
                        kinds.iter().any(|k| k == kind),
                        "{kind} in `{snippet}` is not a kind"
                    );
                }
            }
        }
        if is_command(&snippet) && words.get(1) == Some(&"skill") {
            for word in words.iter().skip(2).filter(|w| !w.starts_with(['-', '<'])) {
                for sub in clean(word).split('|') {
                    assert!(
                        subcommands.iter().any(|s| s == sub),
                        "{sub} in `{snippet}` is not a skill command"
                    );
                }
            }
        }
    }
}

#[test]
fn every_flag_command_and_kind_in_the_help_is_named_by_the_skill() {
    let dir = fresh("named");
    let top = help(&dir, &["--help"]);
    let skill = help(&dir, &["skill", "--help"]);
    let snippets = all_snippets();
    let named: BTreeSet<String> = snippets.iter().flat_map(|s| flags(s)).collect();
    for flag in flags(&top).into_iter().chain(flags(&skill)) {
        assert!(
            named.contains(&flag),
            "{flag} is in the help, not in the skill"
        );
    }
    let text = everything();
    for sub in commands_of(&top) {
        assert!(text.contains(&format!("pcmp {sub}")), "pcmp {sub}");
    }
    for sub in commands_of(&skill) {
        assert!(
            text.contains(&format!("pcmp skill {sub}")),
            "pcmp skill {sub}"
        );
    }
    for kind in kinds_of(&top) {
        assert!(text.contains(&format!("--kind {kind}")), "--kind {kind}");
    }
}

#[test]
fn the_version_flag_prints_what_the_skill_says_it_does() {
    let dir = fresh("version");
    let version = help(&dir, &["--version"]);
    assert_eq!(version, format!("pcmp {}\n", env!("CARGO_PKG_VERSION")));
    assert_eq!(help(&dir, &["-V"]), version);
    assert_eq!(help(&dir, &["-h"]), help(&dir, &["--help"]));
}

#[test]
fn every_command_in_the_skill_runs_as_written_and_writes_the_files_it_names() {
    let dir = fresh("run");
    std::fs::create_dir_all(dir.join("examples")).expect("examples");
    for entry in std::fs::read_dir(root().join("examples"))
        .expect("examples")
        .flatten()
    {
        std::fs::copy(entry.path(), dir.join("examples").join(entry.file_name())).expect("copy");
    }
    let mut ran = BTreeSet::new();
    for snippet in all_snippets().into_iter().filter(|s| runnable(s)) {
        if !ran.insert(snippet.clone()) {
            continue;
        }
        let words: Vec<&str> = snippet.split_whitespace().skip(1).collect();
        let o = pcmp(&dir, &words);
        assert_eq!(o.status.code(), Some(0), "`{snippet}`: {}", err(&o));
        assert!(o.stderr.is_empty(), "`{snippet}`: {}", err(&o));
        for line in out(&o).lines().filter(|l| l.ends_with(".png")) {
            assert!(dir.join(line).is_file(), "`{snippet}` printed {line}");
        }
    }
    assert!(ran.len() >= 10, "{ran:?}");
    let mut named = 0;
    for snippet in all_snippets() {
        let Some(path) = snippet
            .split_whitespace()
            .next_back()
            .filter(|p| p.ends_with(".png") && !p.contains(['<', '>']))
        else {
            continue;
        };
        let bytes = std::fs::read(dir.join(path))
            .unwrap_or_else(|_| panic!("{path} is named but not drawn"));
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"), "{path}");
        named += 1;
    }
    assert!(named >= 10, "{named}");
}

#[test]
fn every_data_file_in_the_skill_draws() {
    let dir = fresh("blocks");
    let mut drawn = 0;
    for file in FILES {
        let (fences, _) = parts(&source(file));
        for (n, fence) in fences.iter().enumerate() {
            let mut info = fence.info.split_whitespace();
            let (lang, kind) = (info.next().unwrap_or(""), info.next().unwrap_or("all"));
            let (name, text) = match lang {
                "json" => {
                    assert!(fence.body.contains("\"title\""), "{file} block {n}");
                    (format!("{file}-{n}.json"), fence.body.clone())
                }
                "toml" if fence.body.contains("title") => {
                    (format!("{file}-{n}.toml"), fence.body.clone())
                }
                "toml" => {
                    let mut doc = String::from("title = \"T\"\nbetter = \"higher\"\n");
                    if !fence.body.contains("rows") && !fence.body.contains("groups") {
                        doc.push_str("rows = [{ label = \"R\", values = [1, 2] }]\n");
                    }
                    if !fence.body.contains("parties") {
                        doc.push_str("parties = [{ name = \"A\" }, { name = \"B\" }]\n");
                    }
                    doc.push_str(&fence.body);
                    (format!("{file}-{n}.toml"), doc)
                }
                _ => continue,
            };
            std::fs::write(dir.join(&name), text).expect("write");
            let o = pcmp(&dir, &[&name, "--kind", kind, "--out", "charts"]);
            assert_eq!(o.status.code(), Some(0), "{name}: {}", err(&o));
            assert!(!out(&o).is_empty(), "{name}");
            drawn += 1;
        }
    }
    assert!(drawn >= 6, "{drawn}");
}

fn with(from: &str, to: &str) -> String {
    assert!(BASE.contains(from), "{from}");
    BASE.replace(from, to)
}

fn many() -> String {
    let parties: Vec<String> = (0..13).map(|i| format!("{{ name = \"P{i}\" }}")).collect();
    let values: Vec<String> = (1..=13).map(|i| i.to_string()).collect();
    format!(
        "title = \"T\"\nbetter = \"higher\"\nparties = [{}]\nrows = [{{ label = \"R\", values = [{}] }}]\n",
        parties.join(", "),
        values.join(", ")
    )
}

struct Case {
    says: &'static str,
    data: Option<String>,
    args: &'static [&'static str],
    code: i32,
}

fn case(says: &'static str, data: String, args: &'static [&'static str]) -> Case {
    Case {
        says,
        data: Some(data),
        args,
        code: 1,
    }
}

fn cases() -> Vec<Case> {
    let long = |n: usize| "A".repeat(n);
    vec![
        case(
            "values holds -612: values are finite and zero or more",
            with("[1, 2]", "[-612, 2]"),
            &[],
        ),
        case("the chart font has no '€'", with("\"R\"", "\"R€\""), &[]),
        case(
            "unknown field",
            with("{ name = \"A\" }", "{ name = \"A\", colur = \"dim\" }"),
            &[],
        ),
        case(
            "missing field",
            "title = \"T\"\nbetter = \"higher\"\nrows = [{ label = \"R\", values = [1] }]\n".into(),
            &[],
        ),
        case(
            "values has 1 values for 2 parties",
            with("[1, 2]", "[1]"),
            &[],
        ),
        case(
            "title is too long: at most 41 characters fit",
            with("\"T\"", &format!("\"{}\"", long(42))),
            &[],
        ),
        case(
            "has a word longer than a line",
            format!("{BASE}notes = [\"{}\"]\n", long(56)),
            &[],
        ),
        case(
            "the value texts are too long to fit beside the bars",
            format!("{BASE}suffix = \" {}\"\n", long(50)),
            &[],
        ),
        case(
            "row 1 (FILE SIZE) has no values to draw as bars",
            with(
                "rows = [{ label = \"R\", values = [1, 2] }]",
                "rows = [{ label = \"FILE SIZE\", ratio = 4.6 }]",
            ),
            &[],
        ),
        case(
            "row 1 (R) has no ratio: the compared values must be above zero",
            with("[1, 2]", "[0, 2]"),
            &["--kind", "ratio"],
        ),
        case(
            "the paired chart needs two parties or more",
            ONE.into(),
            &["--kind", "paired"],
        ),
        case(
            "the ratio chart needs two parties, or a ratio on every row",
            ONE.into(),
            &["--kind", "ratio"],
        ),
        case(
            "give rows or groups, not both",
            format!("{BASE}[[groups]]\nrows = [{{ label = \"R\", values = [1, 2] }}]\n"),
            &[],
        ),
        case(
            "parties[1].name repeats parties[0]",
            with("{ name = \"B\" }", "{ name = \"a\" }"),
            &[],
        ),
        case(
            "parties has 13 parties: a chart draws at most 12",
            many(),
            &[],
        ),
        case(
            "versus names \"C\", which is no party",
            format!("{BASE}versus = \"C\"\n"),
            &[],
        ),
        case(
            "versus must name a party other than the first",
            format!("{BASE}versus = \"A\"\n"),
            &[],
        ),
        case(
            "the ratio chart's unit line is too long",
            format!(
                "{BASE}words = {{ better = \"{}\", worse = \"{}\" }}\n",
                long(20),
                long(20)
            ),
            &["--kind", "ratio"],
        ),
        case(
            "parties[0].colour is \"orange\": use #RRGGBB or a role",
            with("{ name = \"A\" }", "{ name = \"A\", colour = \"orange\" }"),
            &[],
        ),
        Case {
            says: "cannot read nope.toml",
            data: None,
            args: &[],
            code: 1,
        },
        Case {
            says: "invalid value 'nope' for '--kind <KIND>'",
            data: Some(BASE.into()),
            args: &["--kind", "nope"],
            code: 2,
        },
    ]
}

fn refusals_in_the_guide() -> Vec<String> {
    let text = source("output.md");
    let (_, rest) = text
        .split_once("## Reading a refusal\n")
        .expect("the refusal table");
    let table = rest.split("\n## ").next().expect("a section");
    table
        .lines()
        .filter(|l| l.starts_with("| `"))
        .map(|l| l.split('`').nth(1).expect("a first column span").to_owned())
        .collect()
}

#[test]
fn every_refusal_the_guide_reads_is_what_pcmp_says() {
    let dir = fresh("refusals");
    let guide = refusals_in_the_guide();
    let cases = cases();
    for case in &cases {
        let mut args: Vec<&str> = Vec::new();
        match &case.data {
            Some(data) => {
                std::fs::write(dir.join("case.toml"), data).expect("write");
                args.push("case.toml");
            }
            None => args.push("nope.toml"),
        }
        args.extend_from_slice(case.args);
        let o = pcmp(&dir, &args);
        let said = err(&o);
        assert_eq!(o.status.code(), Some(case.code), "{}: {said}", case.says);
        assert!(o.stdout.is_empty(), "{}", case.says);
        assert!(said.contains(case.says), "{}: {said}", case.says);
        let start = if case.code == 2 { "error: " } else { "pcmp: " };
        assert!(said.starts_with(start), "{said}");
        assert!(
            guide.iter().any(|g| g == case.says),
            "{} is not in the refusal table",
            case.says
        );
    }
    for line in &guide {
        assert!(
            cases.iter().any(|c| c.says == line),
            "{line} is in the refusal table and no case draws it"
        );
    }
}
