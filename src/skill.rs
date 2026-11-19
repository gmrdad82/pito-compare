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

const MARK: &str = ".pcmp-skill";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const HOMES: [(&str, &str); 2] = [
    ("Claude skill", ".claude/skills/pcmp"),
    ("Agents skill", ".agents/skills/pcmp"),
];

#[derive(Subcommand)]
pub enum Action {
    #[command(
        about = "put the pcmp skill in ~/.local/opt/pcmp/skills/pcmp and link it into ~/.claude/skills and ~/.agents/skills"
    )]
    Install,
    #[command(about = "show whether the pcmp skill is installed in each agent's folder")]
    Status,
}

#[derive(Debug, PartialEq, Eq)]
enum Place {
    Missing,
    Ours,
    Foreign,
}

fn home() -> Result<PathBuf, String> {
    ["HOME", "USERPROFILE"]
        .iter()
        .filter_map(std::env::var_os)
        .find(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "no home folder: set HOME".to_owned())
}

fn store(home: &Path) -> PathBuf {
    home.join(".local/opt/pcmp/skills/pcmp")
}

fn linked() -> bool {
    cfg!(unix)
}

fn at(path: &Path, e: io::Error) -> String {
    format!("{}: {e}", path.display())
}

fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

fn matched(dir: &Path) -> bool {
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
    want.push(MARK);
    want.sort_unstable();
    have == want
        && FILES
            .iter()
            .all(|(name, text)| fs::read_to_string(dir.join(name)).is_ok_and(|have| have == *text))
}

fn place(dir: &Path, store: &Path) -> Place {
    let Ok(meta) = fs::symlink_metadata(dir) else {
        return Place::Missing;
    };
    if meta.file_type().is_symlink() {
        return if fs::read_link(dir).is_ok_and(|to| to == store) {
            Place::Ours
        } else {
            Place::Foreign
        };
    }
    if meta.is_dir() && dir.join(MARK).is_file() {
        Place::Ours
    } else {
        Place::Foreign
    }
}

fn prune(dir: &Path) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| at(dir, e))?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == MARK || FILES.iter().any(|(kept, _)| *kept == name) {
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
    fs::write(dir.join(MARK), format!("{VERSION}\n")).map_err(|e| at(&dir.join(MARK), e))?;
    prune(dir)
}

#[cfg(unix)]
fn link(dir: &Path, store: &Path) -> Result<(), String> {
    if fs::symlink_metadata(dir).is_ok() {
        if is_link(dir) {
            return Ok(());
        }
        fs::remove_dir_all(dir).map_err(|e| at(dir, e))?;
    }
    if let Some(parent) = dir.parent() {
        fs::create_dir_all(parent).map_err(|e| at(parent, e))?;
    }
    std::os::unix::fs::symlink(store, dir).map_err(|e| at(dir, e))
}

#[cfg(not(unix))]
fn link(dir: &Path, _store: &Path) -> Result<(), String> {
    write(dir)
}

fn describe(label: &str, dir: &Path, store: &Path) -> String {
    match place(dir, store) {
        Place::Missing => format!("{label}: not installed"),
        Place::Foreign => format!(
            "{label}: {} is not the pcmp skill this install placed, left alone",
            dir.display()
        ),
        Place::Ours if matched(dir) => {
            format!(
                "{label}: installed at {}, matches pcmp {VERSION}",
                dir.display()
            )
        }
        Place::Ours => format!(
            "{label}: installed at {}, does not match pcmp {VERSION}; pcmp skill install refreshes it",
            dir.display()
        ),
    }
}

fn status_in(home: &Path) -> Vec<String> {
    let store = store(home);
    HOMES
        .iter()
        .map(|(label, rel)| describe(label, &home.join(rel), &store))
        .collect()
}

fn install_in(home: &Path) -> Result<Vec<String>, String> {
    let store = store(home);
    if linked() {
        write(&store)?;
    }
    for (_, rel) in HOMES {
        let dir = home.join(rel);
        if place(&dir, &store) != Place::Foreign {
            link(&dir, &store)?;
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
