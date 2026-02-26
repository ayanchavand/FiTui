use ratatui::{
    prelude::*,
    widgets::{Block, Table, Row, Cell, TableState, Padding, Paragraph},
};

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
        .map(|t| Line::from(Span::styled(
            format!(" {} ", t),
            Style::default().fg(theme.foreground),
        )))
        .collect();

    let tabs = ratatui::widgets::Tabs::new(titles)
        .select(app.current_tab())
        // no block = no borders
        .style(
            Style::default()
                .bg(theme.surface)   // solid bar color
                .fg(theme.foreground),
        )
        .highlight_style(
            Style::default()
                .bg(theme.accent)        // invert background
                .fg(theme.background)    // invert foreground
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(" ")); // remove pipe dividers

    f.render_widget(tabs, area);
}

pub fn draw_ui(f: &mut Frame, app: &App, snapshot: &StatsSnapshot) {
    let theme = Theme::default();

    // allocate a small strip at top for tabs, remainder for view-specific content
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1),   // ← was 3
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
                "'a'",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to add one",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
            ),
        ]));
        f.render_widget(empty, layout[0]);
    } else {
        // header with vertical separators
        let header = Row::new(vec![
            Cell::from(Span::styled(
                "Date",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            sep_cell(theme),
            Cell::from(Span::styled(
                "Source",
                Style::default().fg(theme.subtle).add_modifier(Modifier::BOLD),
            )),
            sep_cell(theme),
            Cell::from(Span::styled(
                "Amount",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            sep_cell(theme),
            Cell::from(Span::styled(
                "Type",
                Style::default().fg(theme.subtle).add_modifier(Modifier::BOLD),
            )),
            sep_cell(theme),
            Cell::from(Span::styled(
                "Rec",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            sep_cell(theme),
            Cell::from(Span::styled(
                "Tag",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
        ])
        .style(Style::default().bg(theme.accent_soft));

        // build rows (horizontal separators removed)
        let mut rows_with_sep: Vec<Row> = Vec::new();
        for tx in transactions {
            rows_with_sep.push(transaction_row(tx, app, theme, &app.currency));
        }

        let mut state = create_table_state(app.selected);

        // columns now include Rec indicator plus separator columns
        let table = Table::new(rows_with_sep, &[
                Constraint::Ratio(1, 12), // date
                Constraint::Length(1),     // sep
                Constraint::Ratio(1, 12), // source
                Constraint::Length(1),
                Constraint::Ratio(1, 12), // amount
                Constraint::Length(1),
                Constraint::Ratio(1, 12), // type
                Constraint::Length(1),
                Constraint::Ratio(1, 12), // rec
                Constraint::Length(1),
                Constraint::Ratio(1, 12), // tag
            ])
            .header(header)
            .block(theme.block("Transactions "))
            .column_spacing(1)
            .style(Style::default().bg(theme.background))
            .highlight_style(theme.highlight_style())
            .highlight_symbol("▶ ");

        f.render_stateful_widget(table, layout[0], &mut state);
    }

    // Enhanced footer with better visual grouping
    let footer_block = Block::default()
        .borders(ratatui::widgets::Borders::TOP)
        .border_style(Style::default().fg(theme.subtle))
        .style(Style::default().bg(theme.background))
        .padding(Padding::new(1, 1, 0, 0));

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  [", theme.muted_text()),
        Span::styled("↑↓", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("] Navigate  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("Tab/←→", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("] Switch view  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("d", Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)),
        Span::styled("] Delete  ", theme.muted_text()),

        Span::styled("[", theme.muted_text()),
        Span::styled("e", Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)),
        Span::styled("] Edit  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("q", Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)),
        Span::styled("] Quit", theme.muted_text()),
    ]))
    .block(footer_block);

    f.render_widget(footer, layout[1]);
}

// Helpers for the new table-based UI

fn transaction_row(tx: &Transaction, app: &App, theme: &Theme, currency: &str) -> Row<'static> {
    let color = theme.transaction_color(tx.kind);
    let (icon, _kind_label) = match tx.kind {
        TransactionType::Credit => ("↑", "Credit"),
        TransactionType::Debit => ("↓", "Debit"),
    };

    let recurring_indicator = app
        .get_recurring_for_transaction(tx)
        .map(|r| {
            let interval_icon = match r.interval {
                RecurringInterval::Daily => "📅",
                RecurringInterval::Weekly => "📆",
                RecurringInterval::Monthly => "📅",
            };
            interval_icon.to_string()
        })
        .unwrap_or_default();

    Row::new(vec![
        Cell::from(Span::styled(
            format!("{:<11}", tx.date),
            Style::default().fg(theme.muted),
        )),
        sep_cell(theme),
        Cell::from(Span::styled(
            format!("{:<15}", truncate_string(&tx.source, 15)),
            Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
        )),
        sep_cell(theme),
        Cell::from(Span::styled(
            format!("{}{:>9.2}", currency, tx.amount),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        sep_cell(theme),
        Cell::from(Span::styled(
            icon,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        sep_cell(theme),
        // new recurring column
        Cell::from(Span::styled(
            format!("{}", recurring_indicator),
            Style::default().fg(theme.accent),
        )),
        sep_cell(theme),
        Cell::from(Span::styled(
            format!("#{}", tx.tag.as_str()),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )),
    ])
}

fn recurring_row(entry: &crate::models::RecurringEntry, theme: &Theme) -> Row<'static> {
    let status = if entry.active { "✓" } else { "✗" };
    let status_style = if entry.active {
        theme.success()
    } else {
        Style::default().fg(theme.muted)
    };

    Row::new(vec![
        Cell::from(Span::styled(status, status_style)),
        sep_cell(theme),
        Cell::from(Span::styled(
            format!("{:<20}", truncate_string(&entry.source, 20)),
            Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
        )),
        sep_cell(theme),
        Cell::from(Span::styled(
            format!("{:>10}", entry.amount),
            Style::default().fg(theme.accent),
        )),
        sep_cell(theme),
        Cell::from(Span::styled(
            format!("{:<8}", entry.interval.display()),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC),
        )),
    ])
}

fn sep_cell(theme: &Theme) -> Cell<'static> {
    // vertical separator cell used between columns
    Cell::from(Span::styled(
        "│",
        Style::default().fg(theme.subtle),
    ))
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn create_table_state(selected: usize) -> TableState {
    let mut state = TableState::default();
    state.select(Some(selected));
    state
}

fn draw_recurring_management(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Recurring Entries ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )
    ]))
    .block(theme.block(" "))
    .alignment(Alignment::Left);

    f.render_widget(header, layout[0]);

    if app.recurring_entries.is_empty() {
        let empty = Paragraph::new("No recurring entries yet.")
            .style(Style::default().fg(theme.muted));
        f.render_widget(empty, layout[1]);
    } else {
        let header = Row::new(vec![
            Cell::from(""),
            sep_cell(theme),
            Cell::from(Span::styled(
                "Source",
                Style::default().fg(theme.subtle).add_modifier(Modifier::BOLD),
            )),
            sep_cell(theme),
            Cell::from(Span::styled(
                "Amount",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            sep_cell(theme),
            Cell::from(Span::styled(
                "Interval",
                Style::default().fg(theme.accent_soft).add_modifier(Modifier::BOLD),
            )),
        ])
        .style(Style::default().bg(theme.accent_soft));

        // build rows for recurring entries without horizontal separators
        let mut rows_with_sep: Vec<Row> = Vec::new();
        for entry in &app.recurring_entries {
            rows_with_sep.push(recurring_row(entry, theme));
        }

        let mut state = create_table_state(app.selected_recurring);

        // evenly distribute columns + separators
        let table = Table::new(rows_with_sep, &[
                Constraint::Ratio(1, 8), // status
                Constraint::Length(1),
                Constraint::Ratio(1, 8), // source
                Constraint::Length(1),
                Constraint::Ratio(1, 8), // amount
                Constraint::Length(1),
                Constraint::Ratio(1, 8), // interval
            ])
            .header(header)
            .block(theme.block(" 🔄 List "))
            .column_spacing(1)
            .style(Style::default().bg(theme.background))
            .highlight_style(theme.highlight_style())
            .highlight_symbol("▶ ");

        f.render_stateful_widget(table, layout[1], &mut state);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  [", theme.muted_text()),
        Span::styled("↑↓", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("] Navigate  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("Space", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("] Toggle  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("d", Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)),
        Span::styled("] Delete  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("Esc", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("] Back  ", theme.muted_text()),
        Span::styled("[", theme.muted_text()),
        Span::styled("Tab/←→", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("] Switch view", theme.muted_text()),
    ]))
    .block(
        Block::default()
            .borders(ratatui::widgets::Borders::TOP)
            .border_style(Style::default().fg(theme.subtle))
            .style(Style::default().bg(theme.background))
    )
    .alignment(Alignment::Left);

    f.render_widget(footer, layout[2]);
}

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
        assert_eq!(truncate_string("abcdef", 5), "abcd…");
    }

    #[test]
    fn table_state_selection() {
        let state = create_table_state(3);
        assert_eq!(state.selected(), Some(3));
    }

    #[test]
    fn transaction_row_format() {
        let theme = Theme::default();
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

        let tx = Transaction {
            id: 1,
            source: "Test".into(),
            amount: 12.34,
            kind: TransactionType::Credit,
            tag: Tag("tag".into()),
            date: "2026-02-25".into(),
        };

        let row = transaction_row(&tx, &app, &theme, &app.currency);
        // Basic content checks (string conversion may include styling)
        let debug = format!("{:?}", row);
        assert!(debug.contains("Test"));
        assert!(debug.contains("12.34"));
        // should include vertical separators
        assert!(debug.contains('│'));
    }

    #[test]
    fn tabs_constant_and_selection() {
        // ensure we kept three titles and selection reflects app state
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
        assert!(debug.contains('│')); // vertical separator should appear
    }
}

