use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FILES: [&str; 5] = [
    "SKILL.md",
    "data-file.md",
    "charts.md",
    "styling.md",
    "output.md",
];

fn fresh(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("skill-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn pcmp(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("USERPROFILE", home)
        .output()
        .expect("pcmp runs")
}

fn said(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn source(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills/pcmp")
            .join(name),
    )
    .expect("a skill file")
}

fn front_matter(text: &str) -> Vec<(String, String)> {
    let body = text.strip_prefix("---\n").expect("opens with ---");
    let (head, _) = body.split_once("\n---\n").expect("closes with ---");
    let mut keys: Vec<(String, String)> = Vec::new();
    for line in head.lines() {
        if let Some(rest) = line.strip_prefix("  ") {
            let last = keys.last_mut().expect("a folded line follows a key");
            last.1.push(' ');
            last.1.push_str(rest.trim());
        } else {
            let (key, value) = line.split_once(':').expect("key: value");
            keys.push((
                key.trim().to_owned(),
                value.trim().trim_start_matches('>').trim().to_owned(),
            ));
        }
    }
    keys
}

fn names(dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(dir)
        .expect("a folder")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

fn wanted() -> Vec<String> {
    let mut all: Vec<String> = FILES.iter().map(|s| (*s).to_owned()).collect();
    all.sort();
    all
}

#[test]
fn the_front_matter_parses_with_a_name_a_description_and_triggers() {
    let keys = front_matter(&source("SKILL.md"));
    assert_eq!(
        keys.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
        ["name", "description"]
    );
    assert_eq!(keys[0].1, "pcmp");
    let description = &keys[1].1;
    assert!(
        (200..=1024).contains(&description.len()),
        "{}",
        description.len()
    );
    assert!(description.contains("Triggers: pcmp, pito-compare,"));
    for word in [
        "benchmark chart",
        "paired bars",
        "ratio chart",
        "JSON or TOML",
    ] {
        assert!(description.contains(word), "{word}");
    }
}

#[test]
fn the_skill_has_every_section_and_links_every_guide() {
    let text = source("SKILL.md");
    for heading in [
        "# pcmp Skill",
        "## Topic Guides",
        "## Command Discovery",
        "## Critical Safety Rules",
        "## Decision Framework",
        "## Example Requests",
        "## Out of Scope",
    ] {
        assert!(text.contains(&format!("\n{heading}\n")), "{heading}");
    }
    assert!(text.contains("pcmp --help"));
    for guide in &FILES[1..] {
        assert!(text.contains(&format!("]({guide})")), "{guide}");
        assert!(!source(guide).trim().is_empty());
    }
}

#[test]
fn status_says_not_installed_in_an_empty_home() {
    let home = fresh("empty");
    let out = pcmp(&home, &["skill", "status"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        said(&out),
        "Claude skill: not installed\nAgents skill: not installed\n"
    );
    assert!(!home.join(".claude").exists() && !home.join(".local").exists());
}

#[test]
fn install_lands_every_file_and_status_says_installed() {
    let home = fresh("install");
    let out = pcmp(&home, &["skill", "install"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = env!("CARGO_PKG_VERSION");
    let claude = home.join(".claude/skills/pcmp");
    let agents = home.join(".agents/skills/pcmp");
    let matches = format!(
        "Claude skill: installed at {}, matches pcmp {v}\nAgents skill: installed at {}, matches pcmp {v}\n",
        claude.display(),
        agents.display()
    );
    assert_eq!(said(&out), matches);
    assert_eq!(said(&pcmp(&home, &["skill", "status"])), matches);
    for dir in [&claude, &agents] {
        assert_eq!(names(dir), wanted());
        for name in FILES {
            assert_eq!(
                std::fs::read_to_string(dir.join(name)).expect("installed"),
                source(name),
                "{name}"
            );
        }
    }
    assert!(!home.join(".local").exists() && !home.join(".codex").exists());
    for dir in [&claude, &agents] {
        assert!(
            !std::fs::symlink_metadata(dir)
                .expect("there")
                .file_type()
                .is_symlink()
        );
    }
}

#[test]
fn a_second_install_changes_nothing() {
    let home = fresh("again");
    let first = said(&pcmp(&home, &["skill", "install"]));
    let second = said(&pcmp(&home, &["skill", "install"]));
    assert_eq!(first, second);
    assert_eq!(names(&home.join(".claude/skills/pcmp")), wanted());
}

#[test]
fn an_older_copy_is_replaced_and_the_files_a_new_version_drops_are_removed() {
    let home = fresh("older");
    let dir = home.join(".claude/skills/pcmp");
    std::fs::create_dir_all(dir.join("extra")).expect("old");
    std::fs::write(dir.join("SKILL.md"), "an older skill\n").expect("old");
    std::fs::write(dir.join("dropped.md"), "gone in the new one\n").expect("old");
    std::fs::write(dir.join("extra/deep.md"), "old\n").expect("old");
    std::fs::write(dir.join(".pcmp-skill"), "1.0.0\n").expect("old");
    let stale = said(&pcmp(&home, &["skill", "status"]));
    assert_eq!(stale.matches("does not match pcmp").count(), 1, "{stale}");
    let out = pcmp(&home, &["skill", "install"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        said(&out).matches(", matches pcmp").count(),
        2,
        "{}",
        said(&out)
    );
    assert_eq!(names(&dir), wanted());
    for name in FILES {
        assert_eq!(
            std::fs::read_to_string(dir.join(name)).expect("new"),
            source(name),
            "{name}"
        );
    }
}

#[cfg(unix)]
#[test]
fn an_old_link_is_replaced_by_files_and_the_store_it_pointed_at_is_untouched() {
    let home = fresh("link");
    let store = home.join(".local/opt/pcmp/skills/pcmp");
    std::fs::create_dir_all(&store).expect("store");
    std::fs::write(store.join("SKILL.md"), "the store's\n").expect("store");
    for rel in [".claude/skills", ".agents/skills"] {
        std::fs::create_dir_all(home.join(rel)).expect("home");
        std::os::unix::fs::symlink(&store, home.join(rel).join("pcmp")).expect("link");
    }
    let stale = said(&pcmp(&home, &["skill", "status"]));
    assert_eq!(stale.matches("does not match pcmp").count(), 2, "{stale}");
    let out = pcmp(&home, &["skill", "install"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        said(&out).matches(", matches pcmp").count(),
        2,
        "{}",
        said(&out)
    );
    for rel in [".claude/skills/pcmp", ".agents/skills/pcmp"] {
        let dir = home.join(rel);
        assert!(
            !std::fs::symlink_metadata(&dir)
                .expect("there")
                .file_type()
                .is_symlink()
        );
        assert_eq!(names(&dir), wanted());
    }
    assert_eq!(names(&store), ["SKILL.md"]);
    assert_eq!(
        std::fs::read_to_string(store.join("SKILL.md")).expect("kept"),
        "the store's\n"
    );
}

#[test]
fn codex_and_pi_get_the_skill_only_when_their_folders_exist() {
    let home = fresh("others");
    std::fs::create_dir_all(home.join(".codex")).expect("codex");
    let out = said(&pcmp(&home, &["skill", "install"]));
    assert!(out.contains("Codex skill: installed at "), "{out}");
    assert!(!out.contains("Pi skill"), "{out}");
    assert_eq!(names(&home.join(".codex/skills/pcmp")), wanted());
    assert!(!home.join(".pi").exists());
    std::fs::create_dir_all(home.join(".pi")).expect("pi");
    let out = said(&pcmp(&home, &["skill", "install"]));
    assert_eq!(out.matches(", matches pcmp").count(), 4, "{out}");
    assert_eq!(names(&home.join(".pi/agent/skills/pcmp")), wanted());
}

#[test]
fn a_changed_file_reads_as_not_matching() {
    let home = fresh("edited");
    pcmp(&home, &["skill", "install"]);
    std::fs::write(home.join(".claude/skills/pcmp/SKILL.md"), "edited\n").expect("edit");
    let text = said(&pcmp(&home, &["skill", "status"]));
    let mut lines = text.lines();
    let claude = lines.next().expect("claude line");
    assert!(
        claude.contains("does not match pcmp")
            && claude.ends_with("pcmp skill install refreshes it"),
        "{text}"
    );
    assert!(
        lines
            .next()
            .expect("agents line")
            .contains(", matches pcmp")
    );
}

#[test]
fn no_home_is_one_line_and_exit_1() {
    let out = Command::new(env!("CARGO_BIN_EXE_pcmp"))
        .args(["skill", "status"])
        .env_clear()
        .output()
        .expect("pcmp runs");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "pcmp: no home folder: set HOME\n"
    );
}

#[test]
fn the_skill_command_answers_help_and_refuses_what_it_does_not_know() {
    let home = fresh("help");
    let help = pcmp(&home, &["skill", "--help"]);
    assert_eq!(help.status.code(), Some(0));
    let text = said(&help);
    assert!(
        text.contains("install") && text.contains("status"),
        "{text}"
    );
    let bad = pcmp(&home, &["skill", "remove"]);
    assert_eq!(bad.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad.stderr).starts_with("error: "));
}

#[test]
fn the_readme_and_the_docs_say_pcmp_comes_with_a_skill_and_where_it_lands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in ["README.md", "docs/pages/skill.md", "docs/pages/install.md"] {
        let text = std::fs::read_to_string(root.join(file)).expect("a page");
        for needle in [
            "Claude skill",
            "pcmp skill install",
            "~/.claude/skills",
            "~/.agents/skills",
        ] {
            assert!(text.contains(needle), "{file} lacks {needle}");
        }
    }
}

#[test]
fn no_shipped_text_still_describes_the_store_and_links() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in [
        "README.md",
        "bin/docs-bundle",
        "skills/pcmp/SKILL.md",
        "skills/pcmp/reference.md",
    ] {
        let Ok(text) = std::fs::read_to_string(root.join(file)) else {
            continue;
        };
        for stale in [
            ".local/opt/pcmp/skills",
            "left alone",
            "no links",
            "linked from",
            "links it into",
        ] {
            assert!(!text.contains(stale), "{file} still says {stale}");
        }
    }
}

#[test]
fn no_docs_bundle_file_holds_a_home_path() {
    fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("a folder") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                walk(&path, found);
            } else {
                found.push(path);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![root.join("bin/docs-bundle"), root.join("CHANGELOG.md")];
    walk(&root.join("docs/pages"), &mut files);
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for home in ["/home/", "/Users/"] {
            assert!(!text.contains(home), "{} holds {home}", file.display());
        }
    }
}
