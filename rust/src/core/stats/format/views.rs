//! Secondary gain views: `--graph` sparkline, `--daily` table, `--json`.

use super::util::{active_theme, day_total_saved, format_big, format_num, usd_estimate};
use crate::core::theme;

use super::super::model::{CostModel, DayStats};

/// Renders a 30-day token savings bar chart with sparkline.
pub fn format_gain_graph() -> String {
    let theme = active_theme();
    let store = crate::core::stats::load();
    let rst = theme::rst();
    let bold = theme::bold();
    let dim = theme::dim();

    if store.daily.is_empty() {
        return format!(
            "{dim}No daily data yet.{rst} Use lean-ctx for a few days to see the graph."
        );
    }

    let cm = CostModel::default();
    let days: Vec<_> = store
        .daily
        .iter()
        .rev()
        .take(30)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let savings: Vec<u64> = days.iter().map(|day| day_total_saved(day, &cm)).collect();

    let max_saved = *savings.iter().max().unwrap_or(&1);
    let max_saved = max_saved.max(1);

    let bar_width = 36;
    let mut out = Vec::new();

    out.push(String::new());
    out.push(format!(
        "  {icon} {title}  {dim}Token Savings Graph (last 30 days){rst}",
        icon = theme.header_icon(),
        title = theme.brand_title(),
    ));
    out.push(format!("  {ln}", ln = theme.border_line(58)));
    out.push(format!(
        "  {dim}{:>58}{rst}",
        format!("peak: {}", format_big(max_saved))
    ));
    out.push(String::new());

    for (i, day) in days.iter().enumerate() {
        let saved = savings[i];
        let ratio = saved as f64 / max_saved as f64;
        let bar = theme::pad_right(&theme.gradient_bar(ratio, bar_width), bar_width);

        let input_saved = day.input_tokens.saturating_sub(day.output_tokens);
        let pct = if day.input_tokens > 0 {
            input_saved as f64 / day.input_tokens as f64 * 100.0
        } else {
            0.0
        };
        let date_short = day.date.get(5..).unwrap_or(&day.date);

        out.push(format!(
            "  {m}{date_short}{rst} {brd}│{rst} {bar} {bold}{:>6}{rst} {dim}{pct:.0}%{rst}",
            format_big(saved),
            m = theme.muted.fg(),
            brd = theme.border.fg(),
        ));
    }

    let total_saved: u64 = savings.iter().sum();
    let total_cmds: u64 = days.iter().map(|day| day.commands).sum();
    let spark = theme.gradient_sparkline(&savings);

    out.push(String::new());
    out.push(format!("  {ln}", ln = theme.border_line(58)));
    out.push(format!(
        "  {spark}  {bold}{txt}{}{rst} saved across {bold}{}{rst} commands",
        format_big(total_saved),
        format_num(total_cmds),
        txt = theme.text.fg(),
    ));
    out.push(String::new());

    out.join("\n")
}

/// Renders a daily breakdown table of token savings with totals.
pub fn format_gain_daily() -> String {
    format_gain_daily_impl(None)
}

/// Renders a single day's savings as a compact box, or a "no data" message
/// when `date` (already resolved to `YYYY-MM-DD`) has no recorded stats.
/// Reuses the `--daily` table renderer with a one-row filter (#gain-deep-date) —
/// the task/cost/agent/heatmap sections in `--deep` stay all-time since those
/// stores don't carry per-day breakdowns.
pub fn format_gain_day(date: &str) -> String {
    format_gain_daily_impl(Some(date))
}

#[allow(clippy::many_single_char_names)]
fn format_gain_daily_impl(date_filter: Option<&str>) -> String {
    let theme = active_theme();
    let store = crate::core::stats::load();
    let rst = theme::rst();
    let bold = theme::bold();
    let dim = theme::dim();

    let days: Vec<DayStats> = if let Some(d) = date_filter {
        match store.daily.iter().find(|day| day.date == d) {
            Some(day) => vec![day.clone()],
            None => return format!("{dim}No data recorded for {d}.{rst}"),
        }
    } else if store.daily.is_empty() {
        return format!("{dim}No daily data yet.{rst}");
    } else {
        store
            .daily
            .iter()
            .rev()
            .take(30)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .cloned()
            .collect()
    };

    let mut out = Vec::new();
    let w = 76;

    let side = theme.box_side();
    let daily_box = |content: &str| -> String {
        let padded = theme::pad_right(content, w);
        format!("  {side}{padded}{side}")
    };

    out.push(String::new());
    let title = match date_filter {
        Some(d) => format!("Day: {d}"),
        None => "Daily Breakdown".to_string(),
    };
    out.push(format!(
        "  {icon} {title2}  {dim}{title}{rst}",
        icon = theme.header_icon(),
        title2 = theme.brand_title(),
    ));
    out.push(format!("  {}", theme.box_top(w)));
    let hdr = format!(
        " {bold}{txt}{:<12} {:>6}  {:>10}  {:>10}  {:>7}  {:>8}  {:>8}{rst}",
        "Date",
        "Cmds",
        "Input",
        "Saved",
        "Rate",
        "USD",
        "Ver",
        txt = theme.text.fg(),
    );
    out.push(daily_box(&hdr));
    out.push(format!("  {}", theme.box_mid(w)));

    let cm = CostModel::default();
    for day in &days {
        let saved = day_total_saved(day, &cm);
        let input_saved = day.input_tokens.saturating_sub(day.output_tokens);
        let pct = if day.input_tokens > 0 {
            input_saved as f64 / day.input_tokens as f64 * 100.0
        } else {
            0.0
        };
        let pc = theme.pct_color(pct);
        let usd = usd_estimate(saved);
        let ver = if day.version.is_empty() {
            "—".to_string()
        } else {
            format!("v{}", day.version)
        };
        let row = format!(
            " {m}{:<12}{rst} {:>6}  {:>10}  {pc}{bold}{:>10}{rst}  {pc}{:>6.1}%{rst}  {dim}{:>8}{rst}  {dim}{:>8}{rst}",
            day.date,
            day.commands,
            format_big(day.input_tokens),
            format_big(saved),
            pct,
            usd,
            ver,
            m = theme.muted.fg(),
        );
        out.push(daily_box(&row));
    }

    // A single-day view (`--deep --date=...`) is redundant against its own TOTAL
    // row and a one-point trend line — skip both, just close the box.
    if date_filter.is_some() {
        out.push(format!("  {}", theme.box_bottom(w)));
        out.push(String::new());
        return out.join("\n");
    }

    let total_input: u64 = store.daily.iter().map(|day| day.input_tokens).sum();
    let total_saved: u64 = store
        .daily
        .iter()
        .map(|day| day_total_saved(day, &cm))
        .sum();
    let total_pct = if total_input > 0 {
        let input_saved: u64 = store
            .daily
            .iter()
            .map(|day| day.input_tokens.saturating_sub(day.output_tokens))
            .sum();
        input_saved as f64 / total_input as f64 * 100.0
    } else {
        0.0
    };
    let total_usd = usd_estimate(total_saved);
    let sc = theme.success.fg();

    out.push(format!("  {}", theme.box_mid(w)));
    let total_row = format!(
        " {bold}{txt}{:<12}{rst} {:>6}  {:>10}  {sc}{bold}{:>10}{rst}  {sc}{bold}{:>6.1}%{rst}  {bold}{:>8}{rst}  {bold}{:>8}{rst}",
        "TOTAL",
        format_num(store.total_commands),
        format_big(total_input),
        format_big(total_saved),
        total_pct,
        total_usd,
        "",
        txt = theme.text.fg(),
    );
    out.push(daily_box(&total_row));
    out.push(format!("  {}", theme.box_bottom(w)));

    let daily_savings: Vec<u64> = days.iter().map(|day| day_total_saved(day, &cm)).collect();
    let spark = theme.gradient_sparkline(&daily_savings);
    out.push(format!("  {dim}Trend:{rst} {spark}"));
    out.push(String::new());

    out.join("\n")
}

/// Returns the full stats store as pretty-printed JSON.
pub fn format_gain_json() -> String {
    let store = crate::core::stats::load();
    serde_json::to_string_pretty(&store).unwrap_or_else(|_| "{}".to_string())
}
