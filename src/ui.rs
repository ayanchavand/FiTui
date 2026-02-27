use ratatui::{
    prelude::*,
    widgets::{Block, Table, Row, Cell, TableState, Padding, Paragraph},
};
use chrono::Datelike as _;

use crate::{
    app::{App, Mode},
    models::{Transaction, TransactionType, RecurringInterval},
    stats,
    stats::StatsSnapshot,
    theme::Theme,
};

// list of tab titles; order must align with `App::current_tab` mapping
const TAB_TITLES: [&str; 3] = ["Transactions", "Stats", "Recurring"];
mod form;
use form::draw_transaction_form;

mod header;
use header::draw_header;

mod modal;
use modal::draw_popup;

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 30;

fn draw_tabs(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let titles: Vec<Line> = TAB_TITLES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let label = format!("  {}  ", t);
            if i == app.current_tab() {
                Line::from(Span::styled(
                    label,
                    Style::default()
                        .fg(theme.background)
                        .bg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    label,
                    Style::default().fg(theme.foreground).bg(theme.surface),
                ))
            }
        })
        .collect();

    let tabs = ratatui::widgets::Tabs::new(titles)
        .select(app.current_tab())
        .style(Style::default().bg(theme.surface).fg(theme.foreground))
        .highlight_style(
            Style::default()
                .bg(theme.accent)
                .fg(theme.background)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled("│", Style::default().fg(theme.subtle)));

    f.render_widget(tabs, area);
}

pub fn draw_ui(f: &mut Frame, app: &App, snapshot: &StatsSnapshot) {
    let theme = Theme::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(f.size());

    draw_tabs(f, chunks[0], app, &theme);
    let content_area = chunks[1];

    match app.mode {
        Mode::Stats => {
            stats::draw_stats_view(f, content_area, snapshot, &theme, &app.currency);
        }

        Mode::Adding => {
            draw_main_view(
                f,
                content_area,
                &app.transactions,
                snapshot.earned,
                snapshot.spent,
                snapshot.balance,
                app,
                &theme,
            );
            draw_transaction_form(f, app, &theme);
        }

        Mode::Popup => {
            draw_main_view(
                f,
                content_area,
                &app.transactions,
                snapshot.earned,
                snapshot.spent,
                snapshot.balance,
                app,
                &theme,
            );
            draw_popup(f, app, &theme);
        }

        Mode::RecurringManagement => {
            draw_recurring_management(f, content_area, app, &theme);
        }

        _ => {
            draw_main_view(
                f,
                content_area,
                &app.transactions,
                snapshot.earned,
                snapshot.spent,
                snapshot.balance,
                app,
                &theme,
            );
        }
    }
}

fn draw_main_view(
    f: &mut Frame,
    area: Rect,
    transactions: &[Transaction],
    earned: f64,
    spent: f64,
    balance: f64,
    app: &App,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Length(7), Constraint::Min(1)])
        .split(area);

    draw_header(f, chunks[0], earned, spent, balance, theme, &app.currency);
    draw_transactions_list(f, chunks[1], transactions, app, theme);
}

fn draw_transactions_list(
    f: &mut Frame,
    area: Rect,
    transactions: &[Transaction],
    app: &App,
    theme: &Theme,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    if transactions.is_empty() {
        let empty = Paragraph::new(Line::from(vec![
            Span::raw("   "),
            Span::styled(
                "No transactions yet. Press ",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
            ),
            Span::styled(
                "a",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to add one.",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
            ),
        ]));
        f.render_widget(empty, layout[0]);
    } else {
        // Column header row — TYPE removed, BALANCE added
        let header = Row::new(vec![
            centered_header_cell("DATE",    theme.accent, theme),
            sep_cell(theme),
            centered_header_cell("SOURCE",  theme.subtle, theme),
            sep_cell(theme),
            centered_header_cell("AMOUNT",  theme.accent, theme),
            sep_cell(theme),
            centered_header_cell("BALANCE", theme.subtle, theme),
            sep_cell(theme),
            centered_header_cell("RECUR",   theme.accent, theme),
            sep_cell(theme),
            centered_header_cell("TAG",     theme.accent, theme),
        ])
        .style(Style::default().bg(theme.accent_soft))
        .height(1);

        // Pre-compute running balance for each transaction (oldest → newest).
        // Transactions are assumed to be stored newest-first, so we reverse,
        // accumulate, then reverse back so the index matches the display order.
        let mut running: Vec<f64> = Vec::with_capacity(transactions.len());
        {
            let mut bal = 0f64;
            for tx in transactions.iter().rev() {
                match tx.kind {
                    TransactionType::Credit => bal += tx.amount,
                    TransactionType::Debit  => bal -= tx.amount,
                }
                running.push(bal);
            }
            running.reverse();
        }

        // Build rows, inserting a date-group divider whenever the date changes.
        // We track the "previous date label" and inject a separator row before
        // the first transaction of each new group.
        let today     = chrono::Local::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);

        let date_label = |date_str: &str| -> String {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                if d == today     { return "Today".to_string(); }
                if d == yesterday { return "Yesterday".to_string(); }
                // Same year → omit the year for brevity
                if d.year() == today.year() {
                    return d.format("%b %-d").to_string(); // e.g. "Feb 24"
                }
                return d.format("%b %-d, %Y").to_string();
            }
            date_str.to_string()
        };

        // We need to know the total column count to span the divider row.
        // Columns: DATE │ SOURCE │ AMOUNT │ BALANCE │ RECUR │ TAG = 11 cells.
        const COL_COUNT: usize = 11;

        let mut rows: Vec<Row> = Vec::new();
        let mut prev_date: Option<String> = None;
        let mut table_index: usize = 0; // tracks real tx rows for alternating shade

        for (i, tx) in transactions.iter().enumerate() {
            let needs_divider = prev_date.as_deref() != Some(&tx.date);

            if needs_divider {
                let label = date_label(&tx.date);
                // Build a single-cell divider that spans all columns.
                // We fake the span by making the first cell wide and leaving
                // the rest empty — ratatui tables don't support true colspan,
                // so we put the label in column 0 and blank the rest.
                let divider_cells: Vec<Cell> = (0..COL_COUNT)
                    .map(|col| {
                        if col == 0 {
                            Cell::from(
                                Text::from(format!("  ── {} ", label))
                                    .style(Style::default()
                                        .fg(theme.accent)
                                        .add_modifier(Modifier::BOLD | Modifier::ITALIC)),
                            )
                        } else {
                            Cell::from(
                                Text::from(if col % 2 == 1 { "─" } else { "" })
                                    .style(Style::default().fg(theme.subtle)),
                            )
                        }
                    })
                    .collect();

                rows.push(
                    Row::new(divider_cells)
                        .style(Style::default().bg(theme.surface))
                        .height(1),
                );
                prev_date = Some(tx.date.clone());
            }

            // Alternating row background: even/odd based on real tx index.
            let row_bg = if table_index % 2 == 0 {
                theme.background
            } else {
                theme.surface // slightly lighter stripe
            };
            table_index += 1;

            rows.push(transaction_row(tx, running[i], app, theme, &app.currency, row_bg));
        }

        let mut state = create_table_state(app.selected);

        // Columns: DATE │ SOURCE │ AMOUNT │ BALANCE │ RECUR │ TAG
        // TYPE removed (redundant with color+symbol on AMOUNT).
        // BALANCE added in its place at ~same width.
        //   DATE    12% — "YYYY-MM-DD"
        //   SOURCE  28% — widest free-text
        //   AMOUNT  14% — colored amount with symbol
        //   BALANCE 14% — running balance
        //   RECUR   10% — "Monthly" max
        //   TAG     22% — second free-text
        let table = Table::new(rows, &[
                Constraint::Percentage(12), // DATE
                Constraint::Length(1),      // │
                Constraint::Percentage(28), // SOURCE
                Constraint::Length(1),      // │
                Constraint::Percentage(14), // AMOUNT
                Constraint::Length(1),      // │
                Constraint::Percentage(14), // BALANCE
                Constraint::Length(1),      // │
                Constraint::Percentage(10), // RECUR
                Constraint::Length(1),      // │
                Constraint::Percentage(22), // TAG
            ])
            .header(header)
            .block(theme.block("Transactions"))
            .column_spacing(0)
            .style(Style::default().bg(theme.background))
            .highlight_style(theme.highlight_style())
            .highlight_symbol("▶ ");

        f.render_stateful_widget(table, layout[0], &mut state);
    }

    // Footer hint bar
    let footer_block = Block::default()
        .borders(ratatui::widgets::Borders::TOP)
        .border_style(Style::default().fg(theme.subtle))
        .style(Style::default().bg(theme.background))
        .padding(Padding::new(1, 1, 0, 0));

    let key   = |k: &'static str| Span::styled(k, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
    let label = |l: &'static str| Span::styled(l, theme.muted_text());
    let sep   = || Span::styled("  ", theme.muted_text());

    let footer = Paragraph::new(Line::from(vec![
        key("↑↓"), label(" Navigate"),  sep(),
        key("Tab"), label("/"), key("←→"), label(" Switch view"), sep(),
        key("a"), label(" Add"),  sep(),
        key("e"), label(" Edit"),  sep(),
        key("d"), label(" Delete"), sep(),
        key("q"), label(" Quit"),
    ]))
    .block(footer_block);

    f.render_widget(footer, layout[1]);
}

// ---------------------------------------------------------------------------
// Row builders
// ---------------------------------------------------------------------------

fn transaction_row(
    tx: &Transaction,
    running_balance: f64,
    app: &App,
    theme: &Theme,
    currency: &str,
    row_bg: ratatui::style::Color,
) -> Row<'static> {
    let color = theme.transaction_color(tx.kind);

    let direction_symbol = match tx.kind {
        TransactionType::Credit => "▲",
        TransactionType::Debit  => "▼",
    };

    let recur_label = app
        .get_recurring_for_transaction(tx)
        .map(|r| match r.interval {
            RecurringInterval::Daily   => "Daily",
            RecurringInterval::Weekly  => "Weekly",
            RecurringInterval::Monthly => "Monthly",
        })
        .unwrap_or("-");

    let amount_str  = format!("{} {}{:.2}", direction_symbol, currency, tx.amount);
    let balance_str = format!("{}{:.2}", currency, running_balance);

    // Balance color: green if positive, red if negative, muted if zero
    let balance_color = if running_balance > 0.0 {
        theme.credit
    } else if running_balance < 0.0 {
        theme.debit
    } else {
        theme.muted
    };

    Row::new(vec![
        // DATE
        Cell::from(
            Text::from(tx.date.clone())
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.muted).bg(row_bg)),
        ),
        sep_cell(theme),
        // SOURCE
        Cell::from(
            Text::from(truncate_string(&tx.source, 40))
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.foreground).bg(row_bg).add_modifier(Modifier::BOLD)),
        ),
        sep_cell(theme),
        // AMOUNT — colored with direction symbol
        Cell::from(
            Text::from(amount_str)
                .alignment(Alignment::Center)
                .style(Style::default().fg(color).bg(row_bg).add_modifier(Modifier::BOLD)),
        ),
        sep_cell(theme),
        // BALANCE — running total, color reflects sign
        Cell::from(
            Text::from(balance_str)
                .alignment(Alignment::Center)
                .style(Style::default().fg(balance_color).bg(row_bg)),
        ),
        sep_cell(theme),
        // RECUR
        Cell::from(
            Text::from(recur_label)
                .alignment(Alignment::Center)
                .style(Style::default().fg(
                    if recur_label == "-" { theme.muted } else { theme.accent },
                ).bg(row_bg)),
        ),
        sep_cell(theme),
        // TAG
        Cell::from(
            Text::from(tx.tag.as_str().to_owned())
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.accent_soft).bg(row_bg).add_modifier(Modifier::ITALIC)),
        ),
    ])
    .style(Style::default().bg(row_bg))
}

fn recurring_row(entry: &crate::models::RecurringEntry, theme: &Theme) -> Row<'static> {
    let (status_symbol, status_style) = if entry.active {
        ("● Active",   theme.success())
    } else {
        ("○ Paused",   Style::default().fg(theme.muted))
    };

    let interval_str = entry.interval.display().to_owned();

    Row::new(vec![
        Cell::from(
            Text::from(status_symbol)
                .alignment(Alignment::Center)
                .style(status_style),
        ),
        sep_cell(theme),
        Cell::from(
            Text::from(truncate_string(&entry.source, 30))
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        ),
        sep_cell(theme),
        Cell::from(
            Text::from(format!("{:.2}", entry.amount))
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.accent)),
        ),
        sep_cell(theme),
        Cell::from(
            Text::from(interval_str)
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.accent_soft).add_modifier(Modifier::ITALIC)),
        ),
    ])
}

// ---------------------------------------------------------------------------
// Recurring management view
// ---------------------------------------------------------------------------

fn draw_recurring_management(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    let header_para = Paragraph::new(Line::from(vec![
        Span::styled(
            " Recurring Entries",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(theme.block(""))
    .alignment(Alignment::Left);

    f.render_widget(header_para, layout[0]);

    if app.recurring_entries.is_empty() {
        let empty = Paragraph::new(
            "No recurring entries yet. Add a transaction with a recurring interval to get started.",
        )
        .style(Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC));
        f.render_widget(empty, layout[1]);
    } else {
        let table_header = Row::new(vec![
            centered_header_cell("STATUS",   theme.subtle,      theme),
            sep_cell(theme),
            centered_header_cell("SOURCE",   theme.subtle,      theme),
            sep_cell(theme),
            centered_header_cell("AMOUNT",   theme.accent,      theme),
            sep_cell(theme),
            centered_header_cell("INTERVAL", theme.accent_soft, theme),
        ])
        .style(Style::default().bg(theme.accent_soft))
        .height(1);

        let rows: Vec<Row> = app
            .recurring_entries
            .iter()
            .map(|e| recurring_row(e, theme))
            .collect();

        let mut state = create_table_state(app.selected_recurring);

        // Same spacing philosophy: sep_cell handles gaps, column_spacing(0) avoids
        // double-spacing. Percentage splits the available width evenly:
        //   STATUS   15% — "● Active" / "○ Paused"
        //   SOURCE   45% — free text, deserves most space
        //   AMOUNT   20% — numbers
        //   INTERVAL 20% — "Monthly" etc.
        let table = Table::new(rows, &[
                Constraint::Percentage(15), // STATUS
                Constraint::Length(1),      // │
                Constraint::Percentage(45), // SOURCE
                Constraint::Length(1),      // │
                Constraint::Percentage(20), // AMOUNT
                Constraint::Length(1),      // │
                Constraint::Percentage(20), // INTERVAL
            ])
            .header(table_header)
            .block(theme.block(" 🔄 Scheduled"))
            .column_spacing(0)
            .style(Style::default().bg(theme.background))
            .highlight_style(theme.highlight_style())
            .highlight_symbol("▶ ");

        f.render_stateful_widget(table, layout[1], &mut state);
    }

    let key = |k: &'static str| Span::styled(k, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
    let label = |l: &'static str| Span::styled(l, theme.muted_text());
    let sep = || Span::styled("  ", theme.muted_text());

    let footer = Paragraph::new(Line::from(vec![
        key("↑↓"), label(" Navigate"), sep(),
        key("Space"), label(" Toggle active"), sep(),
        key("d"), label(" Delete"), sep(),
        key("Esc"), label(" Back"), sep(),
        key("Tab"), label("/"), key("←→"), label(" Switch view"),
    ]))
    .block(
        Block::default()
            .borders(ratatui::widgets::Borders::TOP)
            .border_style(Style::default().fg(theme.subtle))
            .style(Style::default().bg(theme.background))
            .padding(Padding::new(1, 1, 0, 0)),
    )
    .alignment(Alignment::Left);

    f.render_widget(footer, layout[2]);
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn sep_cell(theme: &Theme) -> Cell<'static> {
    Cell::from(Span::styled(
        "│",
        Style::default().fg(theme.subtle),
    ))
}

/// Build a bold, center-aligned header cell with the given foreground color.
fn centered_header_cell(label: &'static str, fg: ratatui::style::Color, _theme: &Theme) -> Cell<'static> {
    Cell::from(
        Text::from(label)
            .alignment(Alignment::Center)
            .style(Style::default().fg(fg).add_modifier(Modifier::BOLD)),
    )
}

/// Truncate a string to `max_len` chars, appending an ellipsis if cut.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    }
}

fn create_table_state(selected: usize) -> TableState {
    let mut state = TableState::default();
    state.select(Some(selected));
    state
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Transaction, TransactionType, Tag, RecurringEntry, RecurringInterval};

    #[test]
    fn truncate_string_short() {
        assert_eq!(truncate_string("abc", 5), "abc");
    }

    #[test]
    fn truncate_string_long() {
        // 5 chars → keep 4 + ellipsis
        assert_eq!(truncate_string("abcdef", 5), "abcd…");
    }

    #[test]
    fn truncate_string_unicode() {
        // Should truncate on char boundary, not byte boundary
        let s = "héllo world";
        let t = truncate_string(s, 6);
        assert_eq!(t.chars().count(), 6); // 5 chars + ellipsis
    }

    #[test]
    fn table_state_selection() {
        let state = create_table_state(3);
        assert_eq!(state.selected(), Some(3));
    }

    #[test]
    fn transaction_row_format() {
        let theme = Theme::default();
        let app = App {
            mode: Mode::Normal,
            form: crate::form::TransactionForm::new(),
            editing: None,
            tags: vec![],
            transactions: vec![],
            recurring_entries: vec![],
            selected: 0,
            selected_recurring: 0,
            currency: "$".into(),
            popup: None,
        };

        let tx = Transaction {
            id: 1,
            source: "Test".into(),
            amount: 12.34,
            kind: TransactionType::Credit,
            tag: Tag("tag".into()),
            date: "2026-02-25".into(),
        };

        let row = transaction_row(&tx, 12.34, &app, &theme, &app.currency, theme.background);
        let debug = format!("{:?}", row);
        assert!(debug.contains("Test"));
        assert!(debug.contains("12.34"));
        assert!(debug.contains('│'));
        // No leading '#' on tag
        assert!(!debug.contains("#tag"));
        // Should show interval placeholder
        assert!(debug.contains('-'));
    }

    #[test]
    fn tabs_constant_and_selection() {
        assert_eq!(TAB_TITLES.len(), 3);
        let mut app = App {
            mode: Mode::Normal,
            form: crate::form::TransactionForm::new(),
            editing: None,
            tags: vec![],
            transactions: vec![],
            recurring_entries: vec![],
            selected: 0,
            selected_recurring: 0,
            currency: "$".into(),
            popup: None,
        };
        assert_eq!(app.current_tab(), 0);
        app.mode = Mode::Stats;
        assert_eq!(app.current_tab(), 1);
        app.mode = Mode::RecurringManagement;
        assert_eq!(app.current_tab(), 2);
    }

    #[test]
    fn recurring_row_format() {
        let theme = Theme::default();
        let entry = RecurringEntry {
            id: 1,
            source: "Foo".into(),
            amount: 99.0,
            kind: TransactionType::Debit,
            tag: Tag("t".into()),
            interval: RecurringInterval::Weekly,
            original_date: "2026-02-01".into(),
            last_inserted_date: "".into(),
            active: true,
        };

        let row = recurring_row(&entry, &theme);
        let debug = format!("{:?}", row);
        assert!(debug.contains("Foo"));
        assert!(debug.contains("99"));
        assert!(debug.contains('│'));
        assert!(debug.contains("Active"));
    }
}