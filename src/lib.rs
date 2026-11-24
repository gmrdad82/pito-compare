pub mod canvas;
pub mod chart;
pub mod draw;

pub use canvas::{Canvas, Color};
pub use chart::{Better, Chart, Format, Group, Party, Row, Words};
pub use draw::{WIDTHS, bars, bars_at, paired, paired_at, ratios, ratios_at};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(String);

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    All,
    Paired,
    Ratio,
    Bars,
}

pub fn render(chart: &Chart, kind: Kind) -> Result<Vec<(String, Canvas)>> {
    render_at(chart, kind, draw::WIDTH)
}

pub fn render_at(chart: &Chart, kind: Kind, width: i64) -> Result<Vec<(String, Canvas)>> {
    let mut out = Vec::new();
    render_each_at(chart, kind, width, |name, canvas| {
        out.push((name.to_string(), canvas));
        Ok(())
    })?;
    Ok(out)
}

pub fn render_each(
    chart: &Chart,
    kind: Kind,
    each: impl FnMut(&str, Canvas) -> Result<()>,
) -> Result<()> {
    render_each_at(chart, kind, draw::WIDTH, each)
}

pub fn render_each_at(
    chart: &Chart,
    kind: Kind,
    width: i64,
    mut each: impl FnMut(&str, Canvas) -> Result<()>,
) -> Result<()> {
    draw::Look::new(width)?;
    chart.check()?;
    let single = chart.parties.len() == 1;
    if kind == Kind::Paired || (kind == Kind::All && !single) {
        each("paired", paired_at(chart, width)?)?;
    }
    if kind == Kind::Ratio || (kind == Kind::All && !single) {
        each("ratio", ratios_at(chart, width)?)?;
    }
    if kind == Kind::Bars || (kind == Kind::All && single) {
        if single {
            each("bars", bars_at(chart, 0, width)?)?;
        } else {
            let mut used = Vec::new();
            for (i, p) in chart.parties.iter().enumerate() {
                let mut name = format!("bars-{}", slug(&p.name));
                if name == "bars-" || used.contains(&name) {
                    name = format!("bars-{}", i + 1);
                    let mut k = 2;
                    while used.contains(&name) {
                        name = format!("bars-{}-{k}", i + 1);
                        k += 1;
                    }
                }
                each(&name, bars_at(chart, i, width)?)?;
                used.push(name);
            }
        }
    }
    Ok(())
}

fn slug(name: &str) -> String {
    let mut s = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
    }
    s.trim_end_matches('-').to_string()
}
