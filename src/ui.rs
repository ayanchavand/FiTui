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

mod form;
use form::draw_transaction_form;

mod header;
use header::draw_header;

mod modal;
use modal::draw_popup;

// Layout constants
const HEADER_MARGIN_LEFT: u16 = 10;
const HEADER_MARGIN_RIGHT: u16 = 10;
const HEADER_CONTENT_WIDTH: u16 = 80;
const HEADER_PANEL_WIDTH: u16 = 33;
const HEADER_PANEL_WIDTH_CENTER: u16 = 34;

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 30;

pub fn draw_ui(f: &mut Frame, app: &App, snapshot: &StatsSnapshot) {
    let theme = Theme::default();

    match app.mode {
        Mode::Stats => {
            stats::draw_stats_view(f, snapshot, &theme, &app.currency);
        }

        Mode::Adding => {
            draw_main_view(
                f,
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
            draw_recurring_management(f, app, &theme);
        }

        _ => {
            draw_main_view(
                f,
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
        .split(f.size());

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
        let header = Row::new(vec![
            Cell::from(Span::styled(
                "Date",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Source",
                Style::default().fg(theme.subtle).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Amount",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Type",
                Style::default().fg(theme.subtle).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Tag",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
        ]);

        let rows: Vec<Row> = transactions
            .iter()
            .map(|tx| transaction_row(tx, app, theme, &app.currency))
            .collect();

        let mut state = create_table_state(app.selected);

        let table = Table::new(rows, &[
                Constraint::Length(12),
                Constraint::Length(16),
                Constraint::Length(11),
                Constraint::Length(8),
                Constraint::Min(4),
            ])
            .header(header)
            .block(theme.block("Transactions "))
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
        Span::styled("a", Style::default().fg(theme.credit).add_modifier(Modifier::BOLD)),
        Span::styled("] Add  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("e", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("] Edit  ", theme.muted_text()),

        Span::styled("[", theme.muted_text()),
        Span::styled("v", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("] Manage Recurring  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("d", Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)),
        Span::styled("] Delete  ", theme.muted_text()),
        
        Span::styled("[", theme.muted_text()),
        Span::styled("s", Style::default().fg(theme.accent_soft).add_modifier(Modifier::BOLD)),
        Span::styled("] Stats  ", theme.muted_text()),
        
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
            format!(" {}", interval_icon)
        })
        .unwrap_or_default();

    Row::new(vec![
        Cell::from(Span::styled(
            format!("{:<11}", tx.date),
            Style::default().fg(theme.muted),
        )),
        Cell::from(Span::styled(
            format!("{:<15}", truncate_string(&tx.source, 15)),
            Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            format!("{}{:>9.2}", currency, tx.amount),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            icon,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            format!("#{}{}", tx.tag.as_str(), recurring_indicator),
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
        Cell::from(Span::styled(
            format!("{:<20}", truncate_string(&entry.source, 20)),
            Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            format!("{:>10}", entry.amount),
            Style::default().fg(theme.accent),
        )),
        Cell::from(Span::styled(
            format!("{:<8}", entry.interval.display()),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC),
        )),
    ])
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

fn draw_recurring_management(f: &mut Frame, app: &App, theme: &Theme) {
    let area = f.size();

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
            Cell::from(Span::styled(
                "Source",
                Style::default().fg(theme.subtle).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Amount",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Interval",
                Style::default().fg(theme.accent_soft).add_modifier(Modifier::BOLD),
            )),
        ]);

        let rows: Vec<Row> = app
            .recurring_entries
            .iter()
            .map(|entry| recurring_row(entry, theme))
            .collect();

        let mut state = create_table_state(app.selected_recurring);

        let table = Table::new(rows, &[
                Constraint::Length(2),
                Constraint::Length(22),
                Constraint::Length(11),
                Constraint::Length(8),
            ])
            .header(header)
            .block(theme.block(" 🔄 List "))
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
        Span::styled("] Back", theme.muted_text()),
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
        assert!(format!("{:?}", row).contains("Test"));
        assert!(format!("{:?}", row).contains("12.34"));
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
        assert!(format!("{:?}", row).contains("Foo"));
    }
}

