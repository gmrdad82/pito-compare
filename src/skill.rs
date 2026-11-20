use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::Subcommand;

pub const FILES: [(&str, &str); 5] = [
    ("SKILL.md", include_str!("../skills/pcmp/SKILL.md")),
    ("data-file.md", include_str!("../skills/pcmp/data-file.md")),
    ("charts.md", include_str!("../skills/pcmp/charts.md")),
    ("styling.md", include_str!("../skills/pcmp/styling.md")),
    ("output.md", include_str!("../skills/pcmp/output.md")),
];

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HOMES: [(&str, &str, Option<&str>); 4] = [
    ("Claude skill", ".claude/skills/pcmp", None),
    ("Agents skill", ".agents/skills/pcmp", None),
    ("Codex skill", ".codex/skills/pcmp", Some(".codex")),
    ("Pi skill", ".pi/agent/skills/pcmp", Some(".pi")),
];

#[derive(Subcommand)]
pub enum Action {
    #[command(
        about = "write the pcmp skill as files into ~/.claude/skills/pcmp and ~/.agents/skills/pcmp, and into Codex's and Pi's skill folders when present"
    )]
    Install,
    #[command(about = "show whether the pcmp skill is installed in each agent's folder")]
    Status,
}

fn home() -> Result<PathBuf, String> {
    ["HOME", "USERPROFILE"]
        .iter()
        .filter_map(std::env::var_os)
        .find(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "no home folder: set HOME".to_owned())
}

fn at(path: &Path, e: io::Error) -> String {
    format!("{}: {e}", path.display())
}

fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

fn matched(dir: &Path) -> bool {
    if is_link(dir) {
        return false;
    }
    let mut have: Vec<String> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    have.sort();
    let mut want: Vec<&str> = FILES.iter().map(|(name, _)| *name).collect();
    want.sort_unstable();
    have == want
        && FILES
            .iter()
            .all(|(name, text)| fs::read_to_string(dir.join(name)).is_ok_and(|have| have == *text))
}

fn prune(dir: &Path) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| at(dir, e))?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if FILES.iter().any(|(kept, _)| *kept == name) {
            continue;
        }
        let path = entry.path();
        let gone = if path.is_dir() && !is_link(&path) {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
        gone.map_err(|e| at(&path, e))?;
    }
    Ok(())
}

fn write(dir: &Path) -> Result<(), String> {
    if is_link(dir) {
        fs::remove_file(dir).map_err(|e| at(dir, e))?;
    }
    fs::create_dir_all(dir).map_err(|e| at(dir, e))?;
    for (name, text) in FILES {
        fs::write(dir.join(name), text).map_err(|e| at(&dir.join(name), e))?;
    }
    prune(dir)
}

fn describe(label: &str, dir: &Path) -> String {
    if matched(dir) {
        format!(
            "{label}: installed at {}, matches pcmp {VERSION}",
            dir.display()
        )
    } else if fs::symlink_metadata(dir).is_ok() {
        format!(
            "{label}: installed at {}, does not match pcmp {VERSION}; pcmp skill install refreshes it",
            dir.display()
        )
    } else {
        format!("{label}: not installed")
    }
}

fn present(home: &Path, gate: Option<&str>) -> bool {
    gate.is_none_or(|gate| home.join(gate).is_dir())
}

fn status_in(home: &Path) -> Vec<String> {
    HOMES
        .iter()
        .filter(|(_, rel, gate)| {
            present(home, *gate) || fs::symlink_metadata(home.join(rel)).is_ok()
        })
        .map(|(label, rel, _)| describe(label, &home.join(rel)))
        .collect()
}

fn install_in(home: &Path) -> Result<Vec<String>, String> {
    for (_, rel, gate) in HOMES {
        if present(home, gate) {
            write(&home.join(rel))?;
        }
    }
    Ok(status_in(home))
}

pub fn run(action: &Action) -> Result<Vec<String>, String> {
    let home = home()?;
    match action {
        Action::Install => install_in(&home),
        Action::Status => Ok(status_in(&home)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_has_text() {
        assert!(FILES.iter().all(|(_, text)| !text.trim().is_empty()));
    }

    #[test]
    fn the_embedded_files_are_the_files_on_disk() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("skills/pcmp");
        let mut on_disk: Vec<String> = fs::read_dir(&dir)
            .expect("skills/pcmp")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        on_disk.sort();
        let mut embedded: Vec<&str> = FILES.iter().map(|(name, _)| *name).collect();
        embedded.sort_unstable();
        assert_eq!(on_disk, embedded);
    }
}
