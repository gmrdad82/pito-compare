use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FOLDER: &str = ".claude/skills/pito-compare-dev";

const LOCAL: [&str; 2] = ["AGENTS.md", "CLAUDE.md"];

const REPO_DIRS: [&str; 9] = [
    "bin/",
    "src/",
    "tests/",
    "docs/",
    "skills/",
    "examples/",
    "bench/",
    "render/",
    "assets/",
];

const ROOT_FILES: [&str; 3] = [".md", ".toml", ".lock"];

const INTERNAL: [&str; 3] = ["/home/", "/Users/", ".dev/"];

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn folder() -> PathBuf {
    root().join(FOLDER)
}

fn text() -> String {
    std::fs::read_to_string(folder().join("SKILL.md")).expect("SKILL.md")
}

fn front_matter(text: &str) -> Vec<(String, String)> {
    let body = text.strip_prefix("---\n").expect("opens with ---");
    let (head, _) = body.split_once("\n---\n").expect("closes with ---");
    head.lines()
        .map(|line| {
            let (key, value) = line.split_once(':').expect("key: value");
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            (key.trim().to_owned(), value.to_owned())
        })
        .collect()
}

fn prose(text: &str) -> String {
    let mut out = String::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
        } else if !fenced {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn spans(text: &str) -> Vec<String> {
    prose(text)
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

fn links(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in prose(text).lines() {
        let mut rest = line;
        while let Some((_, after)) = rest.split_once("](") {
            let Some((target, tail)) = after.split_once(')') else {
                break;
            };
            found.push(target.to_owned());
            rest = tail;
        }
    }
    found
}

fn looks_like_a_path(token: &str) -> bool {
    if token.contains(['<', '>', '*', '|']) {
        return false;
    }
    REPO_DIRS.iter().any(|d| token.starts_with(d))
        || (!token.contains('/') && ROOT_FILES.iter().any(|e| token.ends_with(e)))
}

fn flags(words: &[&str]) -> Vec<String> {
    words
        .iter()
        .take_while(|w| **w != "--")
        .filter(|w| w.starts_with("--"))
        .map(|w| (*w).to_owned())
        .collect()
}

fn run(program: &str, args: &[&str], home: &Path) -> Output {
    Command::new(program)
        .args(args)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("NO_COLOR", "1")
        .current_dir(root())
        .output()
        .expect("the command runs")
}

fn said(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("dev-skill-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn the_front_matter_parses_and_the_name_is_the_folder() {
    let keys = front_matter(&text());
    assert_eq!(
        keys.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
        ["name", "description"]
    );
    let folder = folder();
    let dir_name = folder.file_name().and_then(|n| n.to_str()).expect("a name");
    assert_eq!(keys[0].1, dir_name);
    assert_eq!(keys[0].1, "pito-compare-dev");
    let description = &keys[1].1;
    assert!(
        (200..=1024).contains(&description.len()),
        "{}",
        description.len()
    );
    assert!(description.starts_with("Develop pito-compare itself"));
    for trigger in [
        "Triggers: pito-compare,",
        "pcmp",
        "bin/gate",
        "bin/images",
        "bin/docs-bundle",
        "src/chart.rs",
        "tests/look.rs",
        "docs/pages",
        "CHANGELOG",
    ] {
        assert!(description.contains(trigger), "{trigger}");
    }
    assert!(
        description.contains("Excludes drawing a chart with pcmp")
            && description.contains("skills/pcmp"),
        "{description}"
    );
}

#[test]
fn the_skill_has_the_shape_of_a_repo_dev_skill_and_stays_one_page() {
    let text = text();
    let mut at = 0;
    for heading in [
        "# pito-compare development",
        "## Layout",
        "## How work goes",
        "## Critical rules",
        "## Known pitfalls",
        "## Command discovery",
        "## Decision framework",
    ] {
        let found = text
            .find(&format!("\n{heading}\n"))
            .unwrap_or_else(|| panic!("{heading}"));
        assert!(found >= at, "{heading} is out of order");
        at = found;
    }
    let (_, discovery) = text
        .split_once("\n## Command discovery\n")
        .expect("section");
    let (table, after) = discovery.split_once("\n## ").expect("a next section");
    assert!(table.contains("| Command | What it does |"));
    assert!(table.contains("source of truth"));
    assert!(table.contains("`pcmp --help`"));
    assert!(after.starts_with("Decision framework"));
    assert!(text.contains("`AGENTS.md`") && text.contains("`CLAUDE.md`"));
    assert!(text.contains("`prompts/RESUME.md`"));
    let mut others: Vec<String> = std::fs::read_dir(folder())
        .expect("the skill folder")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != "SKILL.md")
        .collect();
    others.sort();
    if others.is_empty() {
        assert!(
            text.lines().count() <= 150,
            "{} lines",
            text.lines().count()
        );
    }
    for guide in others {
        assert!(
            text.contains(&format!("]({guide})")),
            "{guide} is not linked"
        );
    }
}

#[test]
fn every_link_resolves_to_a_file_in_the_repo() {
    let found = links(&text());
    assert!(found.len() >= 4, "{found:?}");
    for link in found {
        if link.starts_with("http") {
            continue;
        }
        let target = link.split('#').next().expect("a path");
        let path = folder().join(target);
        assert!(path.is_file(), "{link} is not a file");
        assert!(
            path.canonicalize()
                .expect("a path")
                .starts_with(root().canonicalize().expect("the root")),
            "{link} leaves the repo"
        );
    }
}

#[test]
fn every_path_the_skill_names_is_in_the_repo() {
    let mut checked = BTreeSet::new();
    for span in spans(&text()) {
        for token in span.split_whitespace().filter(|t| looks_like_a_path(t)) {
            if LOCAL.contains(&token) {
                continue;
            }
            assert!(root().join(token).exists(), "{token} is not in the repo");
            checked.insert(token.to_owned());
        }
    }
    assert!(checked.len() >= 30, "{checked:?}");
}

#[test]
fn every_command_the_skill_names_exists_and_its_flags_are_real() {
    let home = scratch("home");
    let pcmp = env!("CARGO_BIN_EXE_pcmp");
    let mut heads = BTreeSet::new();
    for span in spans(&text()) {
        let words: Vec<&str> = span.split_whitespace().collect();
        let Some(head) = words.first().copied() else {
            continue;
        };
        if head.starts_with("bin/") && !head.ends_with('/') {
            let path = root().join(head);
            assert!(path.is_file(), "{head} is not a script here");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = path.metadata().expect("a script").permissions().mode();
                assert_ne!(mode & 0o111, 0, "{head} is not executable");
            }
            let script = std::fs::read_to_string(&path).expect("a script");
            for flag in flags(&words) {
                assert!(
                    script.contains(&flag),
                    "{flag} in `{span}` is not in {head}"
                );
            }
            heads.insert(head.to_owned());
        } else if head == "pcmp" && !span.contains(['<', '[']) {
            let path: Vec<&str> = words[1..]
                .iter()
                .copied()
                .take_while(|w| !w.starts_with('-'))
                .collect();
            let mut args = path.clone();
            args.push("--help");
            let out = run(pcmp, &args, &home);
            assert_eq!(out.status.code(), Some(0), "`{span}`: {}", said(&out));
            let help = said(&out);
            for flag in flags(&words) {
                assert!(
                    help.contains(&flag),
                    "{flag} in `{span}` is not in the help"
                );
            }
            heads.insert("pcmp".to_owned());
        } else if head == "cargo" && !span.contains(['<', '[']) {
            let sub = words.get(1).copied().expect("a cargo command");
            let out = run(env!("CARGO"), &[sub, "--help"], &home);
            assert_eq!(out.status.code(), Some(0), "`{span}`: {}", said(&out));
            let help = said(&out);
            for flag in flags(&words[2..]) {
                assert!(
                    help.contains(&flag),
                    "{flag} in `{span}` is not in cargo's help"
                );
            }
            heads.insert("cargo".to_owned());
        } else if head == "git" {
            let out = run("git", &["--version"], &home);
            assert_eq!(out.status.code(), Some(0), "{}", said(&out));
            heads.insert("git".to_owned());
        }
    }
    for head in [
        "bin/gate",
        "bin/images",
        "bin/docs-bundle",
        "bin/install",
        "pcmp",
        "cargo",
        "git",
    ] {
        assert!(heads.contains(head), "the skill names no {head}");
    }
}

#[test]
fn the_quick_check_the_skill_describes_is_what_the_gate_runs() {
    let gate = std::fs::read_to_string(root().join("bin/gate")).expect("bin/gate");
    let skill = text();
    assert!(skill.contains("`bin/gate --fast`"));
    assert!(gate.contains("[ \"$1\" = \"--fast\" ]"));
    for step in [
        "shellcheck bin/gate",
        "cargo fmt --check",
        "cargo clippy --all-targets --locked -- -D warnings",
        "cargo clippy --lib --no-default-features --locked -- -D warnings",
        "cargo test --all-targets --locked",
        "echo \"$done_line\"",
        "done_line=\"gate: green\"",
    ] {
        assert!(gate.contains(step), "{step}");
    }
    for named in ["`cargo fmt --check`", "`cargo test --all-targets --locked`"] {
        assert!(skill.contains(named), "{named}");
    }
    assert!(skill.contains("ending `gate: green`"));
}

#[test]
fn the_local_install_row_is_what_bin_install_does() {
    let skill = text();
    let row = skill
        .lines()
        .find(|line| line.starts_with("| `bin/install` |"))
        .expect("a bin/install row in the command table");
    let installer = std::fs::read_to_string(root().join("bin/install")).expect("bin/install");
    for step in [
        "cargo build --release --locked --bin pcmp",
        "opt=\"$HOME/.local/opt/pcmp\"",
        "bin=\"$HOME/.local/bin\"",
        "ln -sfn \"$opt/pcmp\" \"$bin/pcmp\"",
        "\"$opt/pcmp\" skill install",
    ] {
        assert!(installer.contains(step), "{step}");
    }
    for named in [
        "local install",
        "release binary",
        "~/.local/opt/pcmp/pcmp",
        "~/.local/bin/pcmp",
        "`pcmp skill install`",
        "user skill",
        "~/.claude/skills/pcmp",
        "~/.agents/skills/pcmp",
        "no window app",
        "after a release",
        "a build that will be used",
    ] {
        assert!(row.contains(named), "{named}");
    }
    assert!(skill.contains("update the local install with `bin/install`"));
    let usage = Command::new("bash")
        .arg(root().join("bin/install"))
        .arg("--help")
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", scratch("install-usage"))
        .output()
        .expect("bash runs bin/install");
    assert_eq!(usage.status.code(), Some(2));
    assert!(said(&usage).starts_with("usage: bin/install"));
}

#[test]
fn the_skill_names_no_home_path_and_no_private_folder() {
    let text = text();
    for internal in INTERNAL {
        assert!(!text.contains(internal), "the skill names {internal}");
    }
}
