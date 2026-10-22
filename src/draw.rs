use crate::canvas::{Canvas, Color, DIM, FASTER, GRID, SEP, SLOWER, TEXT};
use crate::chart::{Chart, Words};
use crate::{Error, Result};

pub const WIDTH: i64 = 1080;
pub const LEFT: i64 = 40;
pub const ROW: i64 = 124;
pub const RATIO_ROW: i64 = 66;
pub const GROUP: i64 = 50;
pub const NOTE: i64 = 30;
pub const BAR_STEP: i64 = 32;
pub const MAX_HEIGHT: i64 = 32_000;
pub const MAX_PARTIES: usize = 12;
pub const MAX_VALUE: f64 = 1e15;

fn wrap(text: &str, scale: usize) -> Vec<String> {
    let max = ((WIDTH - 2 * LEFT) / (6 * scale as i64)) as usize;
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

fn note_lines(notes: &[String]) -> Vec<String> {
    notes.iter().flat_map(|n| wrap(n, 3)).collect()
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

fn tick_text(v: f64, step: f64) -> String {
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else if step < 1e-6 {
        let e = step.log10().floor() as i32;
        format!("{}E{e}", tick_text(v / 10f64.powi(e), 1.0))
    } else {
        let s = format!("{v:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn header(c: &mut Canvas, title: &str, unit: &str, subtitle: &str) {
    c.text(LEFT, 32, title, 4, TEXT);
    c.text(LEFT, 80, unit, 3, TEXT);
    c.text(LEFT, 118, subtitle, 2, DIM);
}

fn group_row(c: &mut Canvas, y: i64, name: &str, first: bool) {
    if !first {
        c.rect(LEFT, y + 6, WIDTH - 2 * LEFT, 2, SEP);
    }
    c.text(LEFT, y + 22, name, 2, DIM);
}

type Slots = Vec<(i64, Option<(i64, String)>)>;

fn layout(chart: &Chart, top: i64, row: i64) -> (Slots, i64) {
    let mut at = Vec::new();
    let mut y = top;
    for g in &chart.groups {
        let mut head = if g.name.is_empty() {
            None
        } else {
            let h = Some((y, g.name.clone()));
            y += GROUP;
            h
        };
        for _ in &g.rows {
            at.push((y, head.take()));
            y += row;
        }
    }
    (at, y)
}

fn draw_groups(c: &mut Canvas, at: &Slots) {
    for (i, (gy, name)) in at.iter().filter_map(|(_, h)| h.as_ref()).enumerate() {
        group_row(c, *gy, name, i == 0);
    }
}

fn canvas(height: i64) -> Result<Canvas> {
    if height > MAX_HEIGHT {
        return Err(Error::new(format!(
            "the chart would be {height} px tall, over {MAX_HEIGHT}: draw fewer rows"
        )));
    }
    Ok(Canvas::new(WIDTH as usize, height as usize))
}

fn legend(c: Option<&mut Canvas>, items: &[(&str, Color)]) -> i64 {
    let mut c = c;
    let (mut x, mut y) = (LEFT, 154);
    for (name, colour) in items {
        let w = 42 + Canvas::text_width(name, 3);
        if x > LEFT && x + w > WIDTH - LEFT {
            x = LEFT;
            y += 34;
        }
        if let Some(c) = c.as_deref_mut() {
            c.rect(x, y, 30, 21, *colour);
            c.text(x + 42, y, name, 3, TEXT);
        }
        x += w + 50;
    }
    y - 154
}

fn bar_chart(chart: &Chart, parties: &[usize], tags: bool) -> Result<Canvas> {
    chart.check()?;
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
    let n = parties.len() as i64;
    let items: Vec<(&str, Color)> = parties
        .iter()
        .map(|&p| (chart.parties[p].name.as_str(), chart.parties[p].colour))
        .collect();
    let extra = legend(None, &items);
    let top = 250 + extra;
    let row_height = ROW - 2 * BAR_STEP + n * BAR_STEP;
    let (at, bottom) = layout(chart, top, row_height);
    let notes = note_lines(&chart.notes);
    let height = bottom + 40 + notes.len() as i64 * NOTE + 30;
    let mut c = canvas(height)?;
    header(
        &mut c,
        &chart.title,
        &chart.unit_line(),
        &chart.subtitle_line(),
    );
    legend(Some(&mut c), &items);
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
    let mut aw = WIDTH - 2 * LEFT - 230;
    for r in chart.rows() {
        for &p in parties {
            let v = r.values[p];
            if v <= 0.0 {
                continue;
            }
            let tw = Canvas::text_width(&chart.value_text(r, p), 3);
            let room = WIDTH - 2 * LEFT - 12 - tw;
            let fit = (room as f64 * peak / v).floor() as i64;
            aw = aw.min(fit);
        }
    }
    if aw < 300 {
        return Err(Error::new(
            "the value texts are too long to fit beside the bars: shorten the suffix or the texts",
        ));
    }
    let x = |v: f64| LEFT + (v / peak * aw as f64).round() as i64;
    let mut grid = Vec::new();
    let mut t = 0.0;
    while t <= peak + step / 2.0 {
        let gx = x(t);
        grid.push((gx, t == 0.0));
        let label = tick_text(t, step);
        let w = Canvas::text_width(&label, 2);
        c.text(
            gx - if t == 0.0 { 0 } else { w / 2 },
            top - 40,
            &label,
            2,
            DIM,
        );
        t += step;
    }
    draw_groups(&mut c, &at);
    for (r, (y, _)) in chart.rows().zip(&at) {
        let y = *y;
        for &(gx, zero) in &grid {
            c.rect(
                gx,
                y + 28,
                if zero { 2 } else { 1 },
                BAR_STEP * n,
                if zero { DIM } else { GRID },
            );
        }
        let label = r.shown_label();
        c.text(LEFT, y, &label, 3, TEXT);
        if tags && let Some(ratio) = chart.ratio(r) {
            let tag = times(ratio, &chart.words);
            let tw = Canvas::text_width(&tag, 3);
            let lw = Canvas::text_width(&label, 3);
            let ty = if LEFT + lw + 24 + tw > WIDTH - LEFT {
                y + 34
            } else {
                y
            };
            c.text(WIDTH - LEFT - tw, ty, &tag, 3, tone(ratio));
        }
        for (i, &p) in parties.iter().enumerate() {
            let by = y + 32 + i as i64 * BAR_STEP;
            let len = (x(r.values[p]) - LEFT).max(2);
            c.rect(LEFT, by, len, 26, chart.parties[p].colour);
            c.text(LEFT + len + 12, by + 3, &chart.value_text(r, p), 3, TEXT);
        }
    }
    for (i, line) in notes.iter().enumerate() {
        c.text(LEFT, bottom + 30 + i as i64 * NOTE, line, 3, DIM);
    }
    Ok(c)
}

pub fn paired(chart: &Chart) -> Result<Canvas> {
    if chart.parties.len() < 2 {
        return Err(Error::new(
            "the paired chart needs two parties or more; a single party draws as bars",
        ));
    }
    let all: Vec<usize> = (0..chart.parties.len()).collect();
    bar_chart(chart, &all, true)
}

pub fn bars(chart: &Chart, party: usize) -> Result<Canvas> {
    if party >= chart.parties.len() {
        return Err(Error::new(format!(
            "there is no party {party}; the chart has {}",
            chart.parties.len()
        )));
    }
    bar_chart(chart, &[party], false)
}

pub fn ratios(chart: &Chart) -> Result<Canvas> {
    chart.check()?;
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
    let half_room = (WIDTH - 2 * LEFT) / 2 - 20;
    if Canvas::text_width(&worse_caption, 2) > half_room
        || Canvas::text_width(&better_caption, 2) > half_room
    {
        return Err(Error::new(format!(
            "the ratio chart's captions are too long: \"{better_caption}\" must fit half the width"
        )));
    }
    let unit = format!("TIMES {} OR {}, LOG SCALE", words.better, words.worse);
    if Canvas::text_width(&unit, 3) > WIDTH - 2 * LEFT {
        return Err(Error::new(format!(
            "the ratio chart's unit line is too long: {unit}"
        )));
    }
    let top = 270i64;
    let (at, bottom) = layout(chart, top, RATIO_ROW);
    let notes = note_lines(&chart.notes);
    let height = bottom + 40 + notes.len() as i64 * NOTE + 30;
    let mut c = canvas(height)?;
    header(&mut c, &chart.title, &unit, &chart.subtitle_line());
    let worst = speedups.iter().map(|s| s.max(1.0 / s)).fold(1.0, f64::max);
    let decades = [2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0];
    let bound = decades.into_iter().find(|d| *d >= worst).unwrap_or(1000.0);
    let cx = WIDTH / 2;
    let half = cx - LEFT - 90;
    let x =
        |r: f64| cx + (r.clamp(1.0 / bound, bound).ln() / bound.ln() * half as f64).round() as i64;
    c.text(LEFT, 160, &worse_caption, 2, DIM);
    let fw = Canvas::text_width(&better_caption, 2);
    c.text(WIDTH - LEFT - fw, 160, &better_caption, 2, DIM);
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
            let w = Canvas::text_width(&a, 2).max(Canvas::text_width(b, 2));
            let (l, r_) = (gx - w / 2, gx + w / 2);
            let fits = match edge {
                None => true,
                Some(e) if side > 0.0 => l > e + 14,
                Some(e) => r_ < e - 14,
            };
            if !fits || l < LEFT || r_ > WIDTH - LEFT {
                continue;
            }
            if t == 1.0 {
                centre = Some((l, r_));
            }
            edge = Some(if side > 0.0 { r_ } else { l });
            c.text(gx - Canvas::text_width(&a, 2) / 2, top - 62, &a, 2, DIM);
            c.text(gx - Canvas::text_width(b, 2) / 2, top - 42, b, 2, DIM);
        }
    }
    draw_groups(&mut c, &at);
    for ((r, speedup), (y, _)) in chart.rows().zip(&speedups).zip(&at) {
        let (y, speedup) = (*y, *speedup);
        for &(gx, one) in &grid {
            c.rect(
                gx,
                y + 24,
                if one { 2 } else { 1 },
                28,
                if one { DIM } else { GRID },
            );
        }
        c.text(LEFT, y, &r.shown_label(), 3, TEXT);
        let (x0, x1) = (x(1.0), x(speedup));
        let colour = if speedup >= 1.0 { FASTER } else { SLOWER };
        c.rect(x0.min(x1), y + 27, (x1 - x0).abs().max(2), 22, colour);
        let tag = format!("{}×", number(speedup.max(1.0 / speedup)));
        let tw = Canvas::text_width(&tag, 3);
        let tx = if speedup >= 1.0 {
            x1.max(x0) + 10
        } else {
            x1.min(x0) - 10 - tw
        };
        c.text(tx, y + 28, &tag, 3, TEXT);
    }
    for (i, line) in notes.iter().enumerate() {
        c.text(LEFT, bottom + 30 + i as i64 * NOTE, line, 3, DIM);
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::tick_text;

    #[test]
    fn ticks_from_a_step_of_a_millionth_up_keep_their_decimals() {
        assert_eq!(tick_text(0.0, 0.0001), "0");
        assert_eq!(tick_text(0.0004, 0.0001), "0.0004");
        assert_eq!(tick_text(0.30000000000000004, 0.1), "0.3");
        assert_eq!(tick_text(0.000003, 0.000001), "0.000003");
        assert_eq!(tick_text(25.0, 5.0), "25");
        assert_eq!(tick_text(2.5, 0.5), "2.5");
    }

    #[test]
    fn ticks_from_a_step_below_a_millionth_are_never_zero() {
        assert_eq!(tick_text(0.0, 5e-7), "0");
        assert_eq!(tick_text(5e-7, 5e-7), "5E-7");
        assert_eq!(tick_text(5e-7 + 5e-7 + 5e-7, 5e-7), "15E-7");
        assert_eq!(tick_text(2.5e-8, 5e-9), "25E-9");
        let mut t = 0.0;
        let step = 2e-301;
        for _ in 0..5 {
            t += step;
            let s = tick_text(t, step);
            assert_ne!(s, "0", "{t:e}");
            assert!(s.ends_with("E-301"), "{s}");
        }
    }
}
