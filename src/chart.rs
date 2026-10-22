use std::path::Path;

use serde::Deserialize;

use crate::canvas::{self, Canvas, Color};
use crate::draw::{MAX_PARTIES, MAX_VALUE};
use crate::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Better {
    Higher,
    Lower,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Words {
    pub better: String,
    pub worse: String,
    pub same: String,
}

impl Default for Words {
    fn default() -> Self {
        Self {
            better: "FASTER".into(),
            worse: "SLOWER".into(),
            same: "SAME SPEED".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Party {
    pub name: String,
    pub colour: Color,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub label: String,
    pub mark: String,
    pub values: Vec<f64>,
    pub texts: Vec<String>,
    pub ratio: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    pub name: String,
    pub rows: Vec<Row>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    pub title: String,
    pub unit: String,
    pub better: Better,
    pub subtitle: String,
    pub date: String,
    pub parties: Vec<Party>,
    pub versus: usize,
    pub groups: Vec<Group>,
    pub notes: Vec<String>,
    pub suffix: String,
    pub decimals: Option<usize>,
    pub words: Words,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Toml,
}

pub const ROLES: &[(&str, Color)] = &[
    ("ours", canvas::OURS),
    ("theirs", canvas::THEIRS),
    ("dim", canvas::DIM),
    ("text", canvas::TEXT),
    ("faster", canvas::FASTER),
    ("slower", canvas::SLOWER),
];

const DEFAULT_ROLES: [&str; 4] = ["ours", "theirs", "dim", "text"];

impl Row {
    pub fn new(label: &str, values: &[f64]) -> Self {
        Self {
            label: label.into(),
            values: values.to_vec(),
            ..Self::default()
        }
    }

    pub fn shown_label(&self) -> String {
        if self.mark.is_empty() {
            self.label.clone()
        } else {
            format!("{} {}", self.label, self.mark)
        }
    }
}

impl Chart {
    pub fn new(title: &str, unit: &str, better: Better, parties: &[&str]) -> Self {
        Self {
            title: title.into(),
            unit: unit.into(),
            better,
            subtitle: String::new(),
            date: String::new(),
            parties: parties
                .iter()
                .enumerate()
                .map(|(i, name)| Party {
                    name: (*name).into(),
                    colour: role(DEFAULT_ROLES[i % DEFAULT_ROLES.len()]).unwrap_or(canvas::TEXT),
                })
                .collect(),
            versus: 1,
            groups: Vec::new(),
            notes: Vec::new(),
            suffix: String::new(),
            decimals: None,
            words: Words::default(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
        let format = match path.extension().and_then(|e| e.to_str()) {
            Some(e) if e.eq_ignore_ascii_case("json") => Format::Json,
            Some(e) if e.eq_ignore_ascii_case("toml") => Format::Toml,
            _ if text
                .strip_prefix('\u{feff}')
                .unwrap_or(&text)
                .trim_start()
                .starts_with('{') =>
            {
                Format::Json
            }
            _ => Format::Toml,
        };
        Self::parse(&text, format).map_err(|e| Error::new(format!("{}: {e}", path.display())))
    }

    pub fn parse(text: &str, format: Format) -> Result<Self> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let spec: Spec = match format {
            Format::Json => serde_json::from_str(text).map_err(|e| Error::new(e.to_string()))?,
            Format::Toml => toml::from_str(text).map_err(|e| toml_error(text, &e))?,
        };
        let chart = spec.into_chart()?;
        chart.check()?;
        Ok(chart)
    }

    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.groups.iter().flat_map(|g| g.rows.iter())
    }

    pub fn ratio(&self, row: &Row) -> Option<f64> {
        if let Some(r) = row.ratio {
            return Some(r);
        }
        let ours = *row.values.first()?;
        let theirs = *row.values.get(self.versus)?;
        if ours <= 0.0 || theirs <= 0.0 {
            return None;
        }
        Some(match self.better {
            Better::Higher => ours / theirs,
            Better::Lower => theirs / ours,
        })
    }

    pub fn value_text(&self, row: &Row, party: usize) -> String {
        if let Some(t) = row.texts.get(party) {
            return t.clone();
        }
        let v = row.values.get(party).copied().unwrap_or(0.0);
        let n = match self.decimals {
            Some(d) => format!("{v:.d$}"),
            None if v < 100.0 => format!("{v:.1}"),
            None => format!("{v:.0}"),
        };
        format!("{n}{}", self.suffix)
    }

    pub fn unit_line(&self) -> String {
        let better = match self.better {
            Better::Higher => "HIGHER IS BETTER",
            Better::Lower => "LOWER IS BETTER",
        };
        if self.unit.is_empty() {
            better.into()
        } else {
            format!("{}, {better}", self.unit)
        }
    }

    pub fn subtitle_line(&self) -> String {
        [self.subtitle.as_str(), self.date.as_str()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn check(&self) -> Result<()> {
        fits("title", &self.title, 4)?;
        if self.title.trim().is_empty() {
            return Err(Error::new("title is empty"));
        }
        fits("unit", &self.unit_line(), 3)?;
        fits("subtitle and date", &self.subtitle_line(), 2)?;
        if self.parties.is_empty() {
            return Err(Error::new("parties is empty: name one party or more"));
        }
        if self.parties.len() > MAX_PARTIES {
            return Err(Error::new(format!(
                "parties has {} parties: a chart draws at most {MAX_PARTIES}",
                self.parties.len()
            )));
        }
        for (i, p) in self.parties.iter().enumerate() {
            let at = format!("parties[{i}].name");
            if p.name.trim().is_empty() {
                return Err(Error::new(format!("{at} is empty")));
            }
            if let Some(j) = self.parties[..i]
                .iter()
                .position(|q| q.name.eq_ignore_ascii_case(&p.name))
            {
                return Err(Error::new(format!("{at} repeats parties[{j}]")));
            }
            glyphs(&at, &p.name)?;
            if Canvas::text_width(&p.name, 3) > crate::draw::WIDTH - 2 * crate::draw::LEFT - 42 {
                return Err(Error::new(format!("{at} is too long for the legend")));
            }
        }
        if self.decimals.is_some_and(|d| d > 6) {
            return Err(Error::new("decimals is at most 6"));
        }
        if self.parties.len() > 1 && (self.versus == 0 || self.versus >= self.parties.len()) {
            return Err(Error::new(
                "versus must name a party other than the first, which is the one compared",
            ));
        }
        for (k, w) in [
            ("words.better", &self.words.better),
            ("words.worse", &self.words.worse),
            ("words.same", &self.words.same),
        ] {
            if w.trim().is_empty() {
                return Err(Error::new(format!("{k} is empty")));
            }
            glyphs(k, w)?;
        }
        if self.rows().next().is_none() {
            return Err(Error::new("there are no rows to draw"));
        }
        for (gi, g) in self.groups.iter().enumerate() {
            fits(&format!("groups[{gi}].name"), &g.name, 2)?;
            if g.rows.is_empty() {
                return Err(Error::new(format!("groups[{gi}] has no rows")));
            }
            for (ri, row) in g.rows.iter().enumerate() {
                let at = format!("groups[{gi}].rows[{ri}]");
                fits(&format!("{at}.label"), &row.shown_label(), 3)?;
                if !row.values.is_empty() && row.values.len() != self.parties.len() {
                    return Err(Error::new(format!(
                        "{at}.values has {} values for {} parties",
                        row.values.len(),
                        self.parties.len()
                    )));
                }
                if let Some(v) = row.values.iter().find(|v| !v.is_finite() || **v < 0.0) {
                    return Err(Error::new(format!(
                        "{at}.values holds {v}: values are finite and zero or more"
                    )));
                }
                if let Some(v) = row.values.iter().find(|v| **v > MAX_VALUE) {
                    return Err(Error::new(format!(
                        "{at}.values holds {v:e}: values are at most {MAX_VALUE:e}"
                    )));
                }
                if !row.texts.is_empty() && row.texts.len() != self.parties.len() {
                    return Err(Error::new(format!(
                        "{at}.texts has {} texts for {} parties",
                        row.texts.len(),
                        self.parties.len()
                    )));
                }
                for t in &row.texts {
                    glyphs(&format!("{at}.texts"), t)?;
                }
                if let Some(r) = row.ratio
                    && !(r.is_finite() && r > 0.0)
                {
                    return Err(Error::new(format!(
                        "{at}.ratio is {r}: a ratio is finite and above zero"
                    )));
                }
                if let Some(r) = self.ratio(row)
                    && !(1.0 / MAX_VALUE..=MAX_VALUE).contains(&r)
                {
                    let from = if row.ratio.is_some() {
                        "ratio is"
                    } else {
                        "values give a ratio of"
                    };
                    return Err(Error::new(format!(
                        "{at}.{from} {r:e}: a ratio is between {:e} and {MAX_VALUE:e}",
                        1.0 / MAX_VALUE
                    )));
                }
                if row.values.is_empty() && row.ratio.is_none() {
                    return Err(Error::new(format!("{at} has neither values nor a ratio")));
                }
            }
        }
        glyphs("suffix", &self.suffix)?;
        for (i, n) in self.notes.iter().enumerate() {
            let at = format!("notes[{i}]");
            glyphs(&at, n)?;
            let max = ((crate::draw::WIDTH - 2 * crate::draw::LEFT) / 18) as usize;
            if let Some(w) = n.split_whitespace().find(|w| w.chars().count() > max) {
                return Err(Error::new(format!(
                    "{at} has a word longer than a line ({max} characters): {w}"
                )));
            }
        }
        Ok(())
    }
}

fn glyphs(at: &str, text: &str) -> Result<()> {
    match text.chars().find(|c| !canvas::has_glyph(*c)) {
        Some(c) => Err(Error::new(format!(
            "{at}: the chart font has no {c:?}; it draws A to Z, 0 to 9, space and . , : ; - + / ( ) % = < > ' _ | × * †"
        ))),
        None => Ok(()),
    }
}

fn fits(at: &str, text: &str, scale: usize) -> Result<()> {
    glyphs(at, text)?;
    let room = crate::draw::WIDTH - 2 * crate::draw::LEFT;
    if Canvas::text_width(text, scale) > room {
        let max = (room + scale as i64) / (6 * scale as i64);
        return Err(Error::new(format!(
            "{at} is too long: at most {max} characters fit"
        )));
    }
    Ok(())
}

pub fn role(name: &str) -> Option<Color> {
    ROLES
        .iter()
        .find(|(r, _)| r.eq_ignore_ascii_case(name))
        .map(|(_, c)| *c)
}

pub fn colour(text: &str) -> Option<Color> {
    if let Some(c) = role(text) {
        return Some(c);
    }
    let hex = text.strip_prefix('#')?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

fn toml_error(text: &str, e: &toml::de::Error) -> Error {
    let message = e.message().trim().to_string();
    match e.span() {
        Some(span) => {
            let before = text.get(..span.start).unwrap_or(text);
            let line = before.matches('\n').count() + 1;
            let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
            Error::new(format!("{message} at line {line} column {column}"))
        }
        None => Error::new(message),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    title: String,
    #[serde(default)]
    unit: String,
    better: BetterSpec,
    #[serde(default)]
    subtitle: String,
    #[serde(default)]
    date: String,
    parties: Vec<PartySpec>,
    #[serde(default)]
    versus: Option<String>,
    #[serde(default)]
    groups: Vec<GroupSpec>,
    #[serde(default)]
    rows: Vec<RowSpec>,
    #[serde(default)]
    notes: Vec<String>,
    #[serde(default)]
    suffix: String,
    #[serde(default)]
    decimals: Option<usize>,
    #[serde(default)]
    words: WordsSpec,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum BetterSpec {
    Higher,
    Lower,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartySpec {
    name: String,
    #[serde(default, alias = "color")]
    colour: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupSpec {
    #[serde(default)]
    name: String,
    rows: Vec<RowSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowSpec {
    label: String,
    #[serde(default)]
    mark: String,
    #[serde(default)]
    values: Vec<f64>,
    #[serde(default)]
    texts: Vec<String>,
    #[serde(default)]
    ratio: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct WordsSpec {
    better: Option<String>,
    worse: Option<String>,
    same: Option<String>,
}

impl RowSpec {
    fn into_row(self) -> Row {
        Row {
            label: self.label,
            mark: self.mark,
            values: self.values,
            texts: self.texts,
            ratio: self.ratio,
        }
    }
}

impl Spec {
    fn into_chart(self) -> Result<Chart> {
        if !self.groups.is_empty() && !self.rows.is_empty() {
            return Err(Error::new("give rows or groups, not both"));
        }
        let mut parties = Vec::new();
        for (i, p) in self.parties.into_iter().enumerate() {
            let colour = match &p.colour {
                Some(c) => colour(c).ok_or_else(|| {
                    Error::new(format!(
                        "parties[{i}].colour is {c:?}: use #RRGGBB or a role (ours, theirs, dim, text, faster, slower)"
                    ))
                })?,
                None => role(DEFAULT_ROLES[i % DEFAULT_ROLES.len()]).unwrap_or(canvas::TEXT),
            };
            parties.push(Party {
                name: p.name,
                colour,
            });
        }
        let versus = match &self.versus {
            None => 1,
            Some(name) => parties
                .iter()
                .position(|p| p.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| Error::new(format!("versus names {name:?}, which is no party")))?,
        };
        let groups = if self.rows.is_empty() {
            self.groups
                .into_iter()
                .map(|g| Group {
                    name: g.name,
                    rows: g.rows.into_iter().map(RowSpec::into_row).collect(),
                })
                .collect()
        } else {
            vec![Group {
                name: String::new(),
                rows: self.rows.into_iter().map(RowSpec::into_row).collect(),
            }]
        };
        let defaults = Words::default();
        Ok(Chart {
            title: self.title,
            unit: self.unit,
            better: match self.better {
                BetterSpec::Higher => Better::Higher,
                BetterSpec::Lower => Better::Lower,
            },
            subtitle: self.subtitle,
            date: self.date,
            parties,
            versus,
            groups,
            notes: self.notes,
            suffix: self.suffix,
            decimals: self.decimals,
            words: Words {
                better: self.words.better.unwrap_or(defaults.better),
                worse: self.words.worse.unwrap_or(defaults.worse),
                same: self.words.same.unwrap_or(defaults.same),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::Error as _;

    #[test]
    fn role_matches_every_name_in_any_case() {
        for (name, c) in ROLES {
            assert_eq!(role(name), Some(*c));
            assert_eq!(role(&name.to_uppercase()), Some(*c));
        }
        assert_eq!(role("purple"), None);
        assert_eq!(role(""), None);
    }

    #[test]
    fn colour_reads_hex_and_roles() {
        assert_eq!(colour("#000000"), Some([0, 0, 0]));
        assert_eq!(colour("#ff8001"), Some([255, 128, 1]));
        assert_eq!(colour("#FF8001"), Some([255, 128, 1]));
        assert_eq!(colour("Faster"), Some(canvas::FASTER));
        for bad in [
            "", "#", "ff8001", "#ff800", "#ff80011", "#gg0000", "#+f8001", "#ff 001", "#éé00",
            "nope",
        ] {
            assert_eq!(colour(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn glyphs_accepts_the_font_and_names_the_first_miss() {
        assert!(glyphs("at", "abc XYZ 0-9 (1.5%) × † |").is_ok());
        assert!(glyphs("at", "").is_ok());
        let m = glyphs("notes[2]", "oké~").expect_err("é").to_string();
        assert!(m.starts_with("notes[2]: the chart font has no 'é'"), "{m}");
    }

    #[test]
    fn fits_stops_at_the_room() {
        assert!(fits("title", &"A".repeat(41), 4).is_ok());
        assert_eq!(
            fits("title", &"A".repeat(42), 4)
                .expect_err("42")
                .to_string(),
            "title is too long: at most 41 characters fit"
        );
        assert!(fits("x", &"A".repeat(83), 2).is_ok());
        assert!(fits("x", &"A".repeat(84), 2).is_err());
        assert!(fits("x", "", 3).is_ok());
        assert!(
            fits("x", "ü", 3)
                .expect_err("glyph")
                .to_string()
                .contains("font has no")
        );
    }

    #[test]
    fn toml_error_counts_lines_and_characters() {
        let text = "a = 1\nb = \"éé\" x\n";
        let e = toml::from_str::<toml::Table>(text).expect_err("broken");
        let m = toml_error(text, &e).to_string();
        let column: usize = m
            .rsplit("column ")
            .next()
            .and_then(|c| c.parse().ok())
            .unwrap_or_else(|| panic!("{m}"));
        assert!(m.contains(" at line 2 column "), "{m}");
        let span = e.span().expect("span");
        let bytes = span.start - text.find('\n').expect("newline");
        assert_eq!(column, bytes - 2, "{m}");
    }

    #[test]
    fn toml_error_without_a_span_keeps_the_message() {
        let e = toml::de::Error::custom("  plain  ");
        assert_eq!(toml_error("x", &e).to_string(), "plain");
    }
}
