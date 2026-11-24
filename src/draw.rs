use crate::canvas::{Canvas, Color, DIM, FASTER, GRID, SEP, SLOWER, TEXT};
use crate::chart::{Chart, Words};
use crate::{Error, Result};

pub const WIDTH: i64 = 1080;
pub const WIDTHS: [i64; 3] = [405, 540, 1080];
pub const LEFT: i64 = 40;
pub const ROW: i64 = 124;
pub const RATIO_ROW: i64 = 66;
pub const GROUP: i64 = 50;
pub const NOTE: i64 = 30;
pub const BAR_STEP: i64 = 32;
pub const MAX_HEIGHT: i64 = 32_000;
pub const MAX_PARTIES: usize = 12;
pub const MAX_VALUE: f64 = 1e15;

#[derive(Clone, Copy, Debug)]
pub struct Look {
    half: bool,
    native: bool,
    wide: i64,
    w: i64,
    left: i64,
    margin: i64,
    title: usize,
    body: usize,
    small: usize,
    shrink: i64,
    unit_y: i64,
    unit_pitch: i64,
    sub_y: i64,
    sub_pitch: i64,
    legend_y: i64,
    legend_step: i64,
    swatch_w: i64,
    swatch_h: i64,
    swatch_gap: i64,
    item_gap: i64,
    top: i64,
    tick_up: i64,
    off: i64,
    step: i64,
    bar_h: i64,
    bar_text: i64,
    grid_up: i64,
    zero_w: i64,
    min_bar: i64,
    value_gap: i64,
    tag_gap: i64,
    tag_drop: i64,
    row_pad: i64,
    note_first: i64,
    note_step: i64,
    bottom_pad: i64,
    end_pad: i64,
    reserve: i64,
    aw_min: i64,
    group_h: i64,
    sep_y: i64,
    sep_h: i64,
    group_name_y: i64,
    cap_y: i64,
    cap_room: i64,
    ratio_top: i64,
    ratio_tick_a: i64,
    ratio_tick_b: i64,
    ratio_row: i64,
    ratio_grid: i64,
    ratio_grid_h: i64,
    ratio_bar: i64,
    ratio_bar_h: i64,
    ratio_tag: i64,
    ratio_side: i64,
    ratio_tag_gap: i64,
    tick_gap: i64,
}

const FULL: Look = Look {
    half: false,
    native: false,
    wide: 1080,
    w: WIDTH,
    left: LEFT,
    margin: 32,
    title: 4,
    body: 3,
    small: 2,
    shrink: 0,
    unit_y: 80,
    unit_pitch: 38,
    sub_y: 118,
    sub_pitch: 36,
    legend_y: 154,
    legend_step: 34,
    swatch_w: 30,
    swatch_h: 21,
    swatch_gap: 42,
    item_gap: 50,
    top: 250,
    tick_up: 40,
    off: 32,
    step: BAR_STEP,
    bar_h: 26,
    bar_text: 3,
    grid_up: 4,
    zero_w: 2,
    min_bar: 2,
    value_gap: 12,
    tag_gap: 24,
    tag_drop: 2,
    row_pad: 28,
    note_first: 30,
    note_step: NOTE,
    bottom_pad: 40,
    end_pad: 30,
    reserve: 230,
    aw_min: 300,
    group_h: GROUP,
    sep_y: 6,
    sep_h: 2,
    group_name_y: 22,
    cap_y: 160,
    cap_room: 20,
    ratio_top: 270,
    ratio_tick_a: 62,
    ratio_tick_b: 42,
    ratio_row: RATIO_ROW,
    ratio_grid: 24,
    ratio_grid_h: 28,
    ratio_bar: 27,
    ratio_bar_h: 22,
    ratio_tag: 28,
    ratio_side: 90,
    ratio_tag_gap: 10,
    tick_gap: 14,
};

const HALF: Look = Look {
    half: true,
    wide: 540,
    title: 6,
    body: 4,
    unit_y: 84,
    unit_pitch: 44,
    sub_y: 128,
    legend_y: 164,
    legend_step: 42,
    swatch_h: 28,
    top: 270,
    off: 40,
    step: 36,
    bar_h: 30,
    bar_text: 2,
    note_step: 38,
    reserve: 300,
    ratio_row: 74,
    ratio_grid: 32,
    ratio_grid_h: 30,
    ratio_bar: 36,
    ratio_tag: 32,
    ..FULL
};

const SLIM: Look = Look {
    half: false,
    native: true,
    wide: 405,
    w: 405,
    left: 15,
    margin: 12,
    title: 2,
    body: 1,
    small: 1,
    shrink: 0,
    unit_y: 32,
    unit_pitch: 14,
    sub_y: 46,
    sub_pitch: 14,
    legend_y: 60,
    legend_step: 13,
    swatch_w: 11,
    swatch_h: 7,
    swatch_gap: 16,
    item_gap: 19,
    top: 96,
    tick_up: 15,
    off: 11,
    step: 12,
    bar_h: 10,
    bar_text: 2,
    grid_up: 1,
    zero_w: 1,
    min_bar: 1,
    value_gap: 5,
    tag_gap: 9,
    tag_drop: 1,
    row_pad: 10,
    note_first: 11,
    note_step: 11,
    bottom_pad: 15,
    end_pad: 11,
    reserve: 86,
    aw_min: 112,
    group_h: 21,
    sep_y: 2,
    sep_h: 1,
    group_name_y: 8,
    cap_y: 60,
    cap_room: 8,
    ratio_top: 106,
    ratio_tick_a: 26,
    ratio_tick_b: 17,
    ratio_row: 25,
    ratio_grid: 9,
    ratio_grid_h: 11,
    ratio_bar: 10,
    ratio_bar_h: 9,
    ratio_tag: 11,
    ratio_side: 34,
    ratio_tag_gap: 4,
    tick_gap: 5,
};

#[derive(Clone, Copy, Debug)]
struct Flow {
    unit_y: i64,
    sub_y: i64,
    shift: i64,
}

fn widths_text() -> String {
    let names: Vec<String> = WIDTHS.iter().map(|w| w.to_string()).collect();
    match names.split_last() {
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

impl Look {
    pub fn new(width: i64) -> Result<Self> {
        match width {
            1080 => Ok(FULL),
            540 => Ok(HALF),
            405 => Ok(SLIM),
            _ => Err(Error::new(format!(
                "the width {width} cannot be drawn well: the widths are {}",
                widths_text()
            ))),
        }
    }

    pub fn width(&self) -> i64 {
        self.wide
    }

    fn fitted(mut self, chart: &Chart) -> Self {
        if self.native
            && self.title > 1
            && Canvas::text_width(&chart.title, self.title) > self.w - 2 * self.left
        {
            self.shrink = 7 * (self.title as i64 - 1);
            self.title = 1;
        }
        self
    }

    fn flow(&self, chart: &Chart, unit: &str) -> Flow {
        let mut shift = self.shrink;
        let unit_y = self.unit_y - shift;
        if chart.bare && unit.is_empty() {
            shift += self.unit_pitch;
        }
        let sub_y = self.sub_y - shift;
        if chart.bare && chart.subtitle_line().is_empty() {
            shift += self.sub_pitch;
        }
        Flow {
            unit_y,
            sub_y,
            shift,
        }
    }

    fn row_height(&self, n: i64) -> i64 {
        self.off + self.row_pad + n * self.step
    }

    fn height(&self, chart: &Chart, bottom: i64, end: i64, notes: usize) -> i64 {
        if chart.bare && notes == 0 {
            end + self.margin
        } else {
            bottom + self.bottom_pad + notes as i64 * self.note_step + self.end_pad
        }
    }

    fn line(&self) -> usize {
        ((self.w - 2 * self.left) / (6 * self.body as i64)) as usize
    }

    fn check(&self, chart: &Chart) -> Result<()> {
        if self.wide == WIDTH {
            return Ok(());
        }
        let wide = self.wide;
        let fit = |at: &str, text: &str, scale: usize, extra: i64| -> Result<()> {
            let room = self.w - 2 * self.left - extra;
            if Canvas::text_width(text, scale) > room {
                let max = (room + scale as i64) / (6 * scale as i64);
                return Err(Error::new(format!(
                    "{at} is too long for a {wide} px wide chart: at most {max} characters fit"
                )));
            }
            Ok(())
        };
        fit("title", &chart.title, self.title, 0)?;
        fit("unit", &chart.unit_line(), self.body, 0)?;
        fit("subtitle and date", &chart.subtitle_line(), self.small, 0)?;
        for (i, p) in chart.parties.iter().enumerate() {
            fit(
                &format!("parties[{i}].name"),
                &p.name,
                self.body,
                self.swatch_gap,
            )?;
        }
        for (gi, g) in chart.groups.iter().enumerate() {
            fit(&format!("groups[{gi}].name"), &g.name, self.small, 0)?;
            for (ri, row) in g.rows.iter().enumerate() {
                fit(
                    &format!("groups[{gi}].rows[{ri}].label"),
                    &row.shown_label(),
                    self.body,
                    0,
                )?;
            }
        }
        let max = self.line();
        for (i, n) in chart.notes.iter().enumerate() {
            if let Some(w) = n.split_whitespace().find(|w| w.chars().count() > max) {
                return Err(Error::new(format!(
                    "notes[{i}] has a word longer than a line of a {wide} px wide chart ({max} characters): {w}"
                )));
            }
        }
        Ok(())
    }
}

fn wrap(text: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > max {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn note_lines(notes: &[String], l: &Look) -> Vec<String> {
    notes.iter().flat_map(|n| wrap(n, l.line())).collect()
}

pub fn number(n: f64) -> String {
    if n >= 10.0 {
        format!("{n:.0}")
    } else {
        format!("{n:.1}")
    }
}

pub fn times(r: f64, words: &Words) -> String {
    let (n, word) = if r >= 1.0 {
        (r, &words.better)
    } else {
        (1.0 / r, &words.worse)
    };
    let n = number(n);
    if n == "1.0" {
        words.same.clone()
    } else {
        format!("{n}× {word}")
    }
}

fn tag_text(chart: &Chart, ratio: f64) -> String {
    let text = times(ratio, &chart.words);
    let w = &chart.words;
    if chart.bare && [&w.better, &w.worse, &w.same].into_iter().all(|s| s == "-") {
        return text.strip_suffix(" -").unwrap_or("").to_string();
    }
    text
}

fn tone(r: f64) -> Color {
    if number(r.max(1.0 / r)) == "1.0" {
        DIM
    } else if r >= 1.0 {
        FASTER
    } else {
        SLOWER
    }
}

fn nice_step(max: f64) -> f64 {
    let raw = max / 5.0;
    let mag = 10f64.powf(raw.log10().floor());
    [1.0, 2.0, 2.5, 5.0, 10.0]
        .into_iter()
        .map(|m| m * mag)
        .find(|s| raw <= *s)
        .unwrap_or(10.0 * mag)
}

fn ticks(step: f64, peak: f64) -> Vec<f64> {
    let mut at = Vec::new();
    let mut t = 0.0;
    while t <= peak + step / 2.0 {
        at.push(t);
        t += step;
    }
    at
}

fn fixed(v: f64, places: usize) -> String {
    let s = format!("{v:.places$}");
    if places == 0 {
        s
    } else {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn tick_text(v: f64, step: f64) -> String {
    if v.fract() == 0.0 {
        return format!("{v:.0}");
    }
    let e = (step.log10() + 1e-9).floor() as i32;
    let m = step / 10f64.powi(e);
    let k = (0..3)
        .find(|&k| {
            let s = m * 10f64.powi(k);
            (s - s.round()).abs() < 1e-6 * s
        })
        .unwrap_or(3);
    let places = k - e;
    if places <= 6 {
        fixed(v, places.max(0) as usize)
    } else {
        format!("{}E{e}", fixed(v / 10f64.powi(e), k as usize))
    }
}

fn header(c: &mut Canvas, l: &Look, f: &Flow, title: &str, unit: &str, subtitle: &str) {
    c.text(l.left, l.margin, title, l.title, TEXT);
    c.text(l.left, f.unit_y, unit, l.body, TEXT);
    c.text(l.left, f.sub_y, subtitle, l.small, DIM);
}

fn group_row(c: &mut Canvas, l: &Look, y: i64, name: &str, first: bool) {
    if !first {
        c.rect(l.left, y + l.sep_y, l.w - 2 * l.left, l.sep_h, SEP);
    }
    c.text(l.left, y + l.group_name_y, name, l.small, DIM);
}

type Slots = Vec<(i64, Option<(i64, String)>)>;

fn layout(chart: &Chart, top: i64, row: i64, group: i64) -> (Slots, i64) {
    let mut at = Vec::new();
    let mut y = top;
    for g in &chart.groups {
        let mut head = if g.name.is_empty() {
            None
        } else {
            let h = Some((y, g.name.clone()));
            y += group;
            h
        };
        for _ in &g.rows {
            at.push((y, head.take()));
            y += row;
        }
    }
    (at, y)
}

fn draw_groups(c: &mut Canvas, l: &Look, at: &Slots) {
    for (i, (gy, name)) in at.iter().filter_map(|(_, h)| h.as_ref()).enumerate() {
        group_row(c, l, *gy, name, i == 0);
    }
}

fn canvas(height: i64, l: &Look) -> Result<Canvas> {
    let height = if l.half { height / 2 } else { height };
    if height > MAX_HEIGHT {
        return Err(Error::new(format!(
            "the chart would be {height} px tall, over {MAX_HEIGHT}: draw fewer rows"
        )));
    }
    Ok(if l.half {
        Canvas::halved(l.wide as usize, height as usize)
    } else {
        Canvas::new(l.wide as usize, height as usize)
    })
}

fn legend(c: Option<&mut Canvas>, l: &Look, at: i64, items: &[(&str, Color)]) -> i64 {
    let mut c = c;
    let (mut x, mut y) = (l.left, at);
    for (name, colour) in items {
        let w = l.swatch_gap + Canvas::text_width(name, l.body);
        if x > l.left && x + w > l.w - l.left {
            x = l.left;
            y += l.legend_step;
        }
        if let Some(c) = c.as_deref_mut() {
            c.rect(x, y, l.swatch_w, l.swatch_h, *colour);
            c.text(x + l.swatch_gap, y, name, l.body, TEXT);
        }
        x += w + l.item_gap;
    }
    y - at
}

fn bar_chart(chart: &Chart, l: &Look, parties: &[usize], tags: bool) -> Result<Canvas> {
    chart.check()?;
    let l = &l.fitted(chart);
    l.check(chart)?;
    if let Some((i, _)) = chart
        .rows()
        .enumerate()
        .find(|(_, r)| r.values.len() != chart.parties.len())
    {
        return Err(Error::new(format!(
            "row {} ({}) has no values to draw as bars",
            i + 1,
            chart.rows().nth(i).map_or("", |r| r.label.as_str())
        )));
    }
    if l.native && tags {
        for (i, r) in chart.rows().enumerate() {
            if let Some(ratio) = chart.ratio(r) {
                let lw = Canvas::text_width(&r.shown_label(), l.body);
                let tw = Canvas::text_width(&tag_text(chart, ratio), l.body);
                if lw + l.tag_gap + tw > l.w - 2 * l.left {
                    return Err(Error::new(format!(
                        "row {} ({}) and its tag do not fit one line of a {} px wide chart: shorten the label or the words",
                        i + 1,
                        r.label,
                        l.wide
                    )));
                }
            }
        }
    }
    let n = parties.len() as i64;
    let items: Vec<(&str, Color)> = parties
        .iter()
        .map(|&p| (chart.parties[p].name.as_str(), chart.parties[p].colour))
        .collect();
    let unit = chart.unit_line();
    let flow = l.flow(chart, &unit);
    let legend_y = l.legend_y - flow.shift;
    let extra = legend(None, l, legend_y, &items);
    let top = l.top - flow.shift + extra;
    let row_height = l.row_height(n);
    let (at, bottom) = layout(chart, top, row_height, l.group_h);
    let notes = note_lines(&chart.notes, l);
    let end = bottom - l.row_pad - l.grid_up;
    let height = l.height(chart, bottom, end, notes.len());
    let mut c = canvas(height, l)?;
    header(
        &mut c,
        l,
        &flow,
        &chart.title,
        &unit,
        &chart.subtitle_line(),
    );
    legend(Some(&mut c), l, legend_y, &items);
    let max = chart
        .rows()
        .flat_map(|r| parties.iter().map(|&p| r.values[p]))
        .fold(0.0, f64::max);
    let max = if max > 0.0 { max } else { 1.0 };
    let step = nice_step(max);
    let peak = (max / step).ceil() * step;
    if !(peak.is_finite() && step.is_finite() && step > 0.0) {
        return Err(Error::new(format!(
            "the values reach {max:e}: an axis is drawn up to {MAX_VALUE:e}"
        )));
    }
    let mut aw = l.w - 2 * l.left - l.reserve;
    for r in chart.rows() {
        for &p in parties {
            let v = r.values[p];
            if v <= 0.0 {
                continue;
            }
            let tw = Canvas::text_width(&chart.value_text(r, p), l.body);
            let room = l.w - 2 * l.left - l.value_gap - tw;
            let fit = (room as f64 * peak / v).floor() as i64;
            aw = aw.min(fit);
        }
    }
    if aw < l.aw_min {
        return Err(Error::new(
            "the value texts are too long to fit beside the bars: shorten the suffix or the texts",
        ));
    }
    let x = |v: f64| l.left + (v / peak * aw as f64).round() as i64;
    let mut grid = Vec::new();
    let mut last = i64::MIN;
    for t in ticks(step, peak) {
        let gx = x(t);
        grid.push((gx, t == 0.0));
        let label = tick_text(t, step);
        let w = Canvas::text_width(&label, l.small);
        let lx = gx - if t == 0.0 { 0 } else { w / 2 };
        if l.native && lx < last + l.tick_gap {
            return Err(Error::new(format!(
                "the axis labels overlap on a {} px wide chart: use smaller values",
                l.wide
            )));
        }
        last = lx + w;
        c.text(lx, top - l.tick_up, &label, l.small, DIM);
    }
    draw_groups(&mut c, l, &at);
    for (r, (y, _)) in chart.rows().zip(&at) {
        let y = *y;
        for &(gx, zero) in &grid {
            c.rect(
                gx,
                y + l.off - l.grid_up,
                if zero { l.zero_w } else { 1 },
                l.step * n,
                if zero { DIM } else { GRID },
            );
        }
        let label = r.shown_label();
        c.text(l.left, y, &label, l.body, TEXT);
        if tags && let Some(ratio) = chart.ratio(r) {
            let tag = tag_text(chart, ratio);
            if !tag.is_empty() {
                let tw = Canvas::text_width(&tag, l.body);
                let lw = Canvas::text_width(&label, l.body);
                let ty = if l.left + lw + l.tag_gap + tw > l.w - l.left {
                    y + l.off + l.tag_drop
                } else {
                    y
                };
                c.text(l.w - l.left - tw, ty, &tag, l.body, tone(ratio));
            }
        }
        for (i, &p) in parties.iter().enumerate() {
            let by = y + l.off + i as i64 * l.step;
            let len = (x(r.values[p]) - l.left).max(l.min_bar);
            c.rect(l.left, by, len, l.bar_h, chart.parties[p].colour);
            c.text(
                l.left + len + l.value_gap,
                by + l.bar_text,
                &chart.value_text(r, p),
                l.body,
                TEXT,
            );
        }
    }
    for (i, line) in notes.iter().enumerate() {
        c.text(
            l.left,
            bottom + l.note_first + i as i64 * l.note_step,
            line,
            l.body,
            DIM,
        );
    }
    Ok(c)
}

pub fn paired(chart: &Chart) -> Result<Canvas> {
    paired_at(chart, WIDTH)
}

pub fn paired_at(chart: &Chart, width: i64) -> Result<Canvas> {
    let l = Look::new(width)?;
    if chart.parties.len() < 2 {
        return Err(Error::new(
            "the paired chart needs two parties or more; a single party draws as bars",
        ));
    }
    let all: Vec<usize> = (0..chart.parties.len()).collect();
    bar_chart(chart, &l, &all, true)
}

pub fn bars(chart: &Chart, party: usize) -> Result<Canvas> {
    bars_at(chart, party, WIDTH)
}

pub fn bars_at(chart: &Chart, party: usize, width: i64) -> Result<Canvas> {
    let l = Look::new(width)?;
    if party >= chart.parties.len() {
        return Err(Error::new(format!(
            "there is no party {party}; the chart has {}",
            chart.parties.len()
        )));
    }
    bar_chart(chart, &l, &[party], false)
}

pub fn ratios(chart: &Chart) -> Result<Canvas> {
    ratios_at(chart, WIDTH)
}

pub fn ratios_at(chart: &Chart, width: i64) -> Result<Canvas> {
    let l = Look::new(width)?;
    chart.check()?;
    let l = &l.fitted(chart);
    l.check(chart)?;
    if chart.parties.len() < 2 && chart.rows().any(|r| r.ratio.is_none()) {
        return Err(Error::new(
            "the ratio chart needs two parties, or a ratio on every row",
        ));
    }
    let mut speedups = Vec::new();
    for (i, r) in chart.rows().enumerate() {
        match chart.ratio(r) {
            Some(s) => speedups.push(s),
            None => {
                return Err(Error::new(format!(
                    "row {} ({}) has no ratio: the compared values must be above zero",
                    i + 1,
                    r.label
                )));
            }
        }
    }
    let versus = chart
        .parties
        .get(chart.versus)
        .map_or("THE OTHER", |p| p.name.as_str());
    let words = &chart.words;
    let worse_caption = format!("< {} THAN {versus}", words.worse);
    let better_caption = format!("{} THAN {versus} >", words.better);
    let half_room = (l.w - 2 * l.left) / 2 - l.cap_room;
    if Canvas::text_width(&worse_caption, l.small) > half_room
        || Canvas::text_width(&better_caption, l.small) > half_room
    {
        return Err(Error::new(format!(
            "the ratio chart's captions are too long: \"{better_caption}\" must fit half the width"
        )));
    }
    let unit = format!("TIMES {} OR {}, LOG SCALE", words.better, words.worse);
    if Canvas::text_width(&unit, l.body) > l.w - 2 * l.left {
        return Err(Error::new(format!(
            "the ratio chart's unit line is too long: {unit}"
        )));
    }
    let flow = l.flow(chart, &unit);
    let top = l.ratio_top - flow.shift;
    let (at, bottom) = layout(chart, top, l.ratio_row, l.group_h);
    let notes = note_lines(&chart.notes, l);
    let end =
        bottom - l.ratio_row + (l.ratio_grid + l.ratio_grid_h).max(l.ratio_bar + l.ratio_bar_h);
    let height = l.height(chart, bottom, end, notes.len());
    let mut c = canvas(height, l)?;
    header(
        &mut c,
        l,
        &flow,
        &chart.title,
        &unit,
        &chart.subtitle_line(),
    );
    let worst = speedups.iter().map(|s| s.max(1.0 / s)).fold(1.0, f64::max);
    let decades = [2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0];
    let bound = decades.into_iter().find(|d| *d >= worst).unwrap_or(1000.0);
    let cx = l.w / 2;
    let half = cx - l.left - l.ratio_side;
    let x =
        |r: f64| cx + (r.clamp(1.0 / bound, bound).ln() / bound.ln() * half as f64).round() as i64;
    let cap_y = l.cap_y - flow.shift;
    c.text(l.left, cap_y, &worse_caption, l.small, DIM);
    let fw = Canvas::text_width(&better_caption, l.small);
    c.text(l.w - l.left - fw, cap_y, &better_caption, l.small, DIM);
    let mut ticks = vec![1.0];
    ticks.extend(decades.into_iter().filter(|d| *d <= bound));
    let mut grid = Vec::new();
    let mut centre: Option<(i64, i64)> = None;
    for side in [1.0f64, -1.0] {
        let mut edge = centre.map(|(l, r)| if side > 0.0 { r } else { l });
        for &t in &ticks {
            if t == 1.0 && side < 0.0 {
                continue;
            }
            let r = t.powf(side);
            let gx = x(r);
            grid.push((gx, t == 1.0));
            if !(t <= 2.0 || t == bound || t.log10().fract() == 0.0) {
                continue;
            }
            let (a, b) = if t == 1.0 {
                ("1×".to_string(), "SAME")
            } else {
                (
                    format!("{}×", tick_text(t, 1.0)),
                    if side > 0.0 {
                        words.better.as_str()
                    } else {
                        words.worse.as_str()
                    },
                )
            };
            let w = Canvas::text_width(&a, l.small).max(Canvas::text_width(b, l.small));
            let (lo, hi) = (gx - w / 2, gx + w / 2);
            let fits = match edge {
                None => true,
                Some(e) if side > 0.0 => lo > e + l.tick_gap,
                Some(e) => hi < e - l.tick_gap,
            };
            if !fits || lo < l.left || hi > l.w - l.left {
                continue;
            }
            if t == 1.0 {
                centre = Some((lo, hi));
            }
            edge = Some(if side > 0.0 { hi } else { lo });
            c.text(
                gx - Canvas::text_width(&a, l.small) / 2,
                top - l.ratio_tick_a,
                &a,
                l.small,
                DIM,
            );
            c.text(
                gx - Canvas::text_width(b, l.small) / 2,
                top - l.ratio_tick_b,
                b,
                l.small,
                DIM,
            );
        }
    }
    draw_groups(&mut c, l, &at);
    for (i, ((r, speedup), (y, _))) in chart.rows().zip(&speedups).zip(&at).enumerate() {
        let (y, speedup) = (*y, *speedup);
        for &(gx, one) in &grid {
            c.rect(
                gx,
                y + l.ratio_grid,
                if one { l.zero_w } else { 1 },
                l.ratio_grid_h,
                if one { DIM } else { GRID },
            );
        }
        c.text(l.left, y, &r.shown_label(), l.body, TEXT);
        let (x0, x1) = (x(1.0), x(speedup));
        let colour = if speedup >= 1.0 { FASTER } else { SLOWER };
        c.rect(
            x0.min(x1),
            y + l.ratio_bar,
            (x1 - x0).abs().max(l.min_bar),
            l.ratio_bar_h,
            colour,
        );
        let tag = format!("{}×", number(speedup.max(1.0 / speedup)));
        let tw = Canvas::text_width(&tag, l.body);
        let tx = if speedup >= 1.0 {
            x1.max(x0) + l.ratio_tag_gap
        } else {
            x1.min(x0) - l.ratio_tag_gap - tw
        };
        if l.native && (tx < l.left || tx + tw > l.w - l.left) {
            return Err(Error::new(format!(
                "row {} ({}) has a tag too wide for a {} px wide chart: {tag}",
                i + 1,
                r.label,
                l.wide
            )));
        }
        c.text(tx, y + l.ratio_tag, &tag, l.body, TEXT);
    }
    for (i, line) in notes.iter().enumerate() {
        c.text(
            l.left,
            bottom + l.note_first + i as i64 * l.note_step,
            line,
            l.body,
            DIM,
        );
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::{nice_step, tick_text, ticks};

    #[test]
    fn ticks_from_a_step_of_a_millionth_up_keep_their_decimals() {
        assert_eq!(tick_text(0.0, 0.0001), "0");
        assert_eq!(tick_text(0.0004, 0.0001), "0.0004");
        assert_eq!(tick_text(0.30000000000000004, 0.1), "0.3");
        assert_eq!(tick_text(0.000003, 0.000001), "0.000003");
        assert_eq!(tick_text(25.0, 5.0), "25");
        assert_eq!(tick_text(2.5, 0.5), "2.5");
        assert_eq!(tick_text(0.75, 0.25), "0.75");
        assert_eq!(tick_text(0.000025, 0.000025), "0.000025");
    }

    #[test]
    fn ticks_from_a_step_below_a_millionth_are_never_zero() {
        assert_eq!(tick_text(0.0, 5e-7), "0");
        assert_eq!(tick_text(5e-7, 5e-7), "5E-7");
        assert_eq!(tick_text(5e-7 + 5e-7 + 5e-7, 5e-7), "15E-7");
        assert_eq!(tick_text(2.5e-8, 5e-9), "25E-9");
        assert_eq!(tick_text(2.5e-6, 2.5e-6), "2.5E-6");
        assert_eq!(tick_text(7.5e-6, 2.5e-6), "7.5E-6");
        assert_eq!(tick_text(5e-6, 2.5e-6), "5E-6");
        let mut t = 0.0;
        let step = 2e-301;
        for _ in 0..5 {
            t += step;
            let s = tick_text(t, step);
            assert_ne!(s, "0", "{t:e}");
            assert!(s.ends_with("E-301"), "{s}");
        }
    }

    #[test]
    fn a_step_a_hair_under_a_power_of_ten_keeps_its_exponent() {
        let step = 1e-7f64.next_down();
        assert_eq!(tick_text(step, step), "1E-7");
        assert_eq!(tick_text(2.0 * step, step), "2E-7");
    }

    #[test]
    fn every_axis_from_a_trillionth_to_the_largest_value_labels_its_ticks_truly() {
        let mut max = 1e-12;
        let mut axes = 0;
        while max <= 1e15 {
            let step = nice_step(max);
            let peak = (max / step).ceil() * step;
            let at = ticks(step, peak);
            assert!(at.len() >= 2, "{max:e}");
            for t in at {
                let label = tick_text(t, step);
                let back: f64 = label
                    .parse()
                    .unwrap_or_else(|_| panic!("{label} for {t:e} on a {max:e} axis"));
                assert!(
                    (back - t).abs() <= step * 1e-6,
                    "{label} for {t:e} on a {max:e} axis, step {step:e}"
                );
            }
            max *= 1.07;
            axes += 1;
        }
        assert!(axes > 600, "{axes}");
    }
}
