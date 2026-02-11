use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::{
    app::{App, Mode},
    models::{Transaction, TransactionType},
};

pub fn draw_ui(
    f: &mut Frame,
    transactions: &[Transaction],
    earned: f64,
    spent: f64,
    balance: f64,
    per_tag: &Vec<(String, f64)>, // ✅ NEW
    app: &App,
) {
    // ✅ If Stats mode, draw stats page only
    if app.mode == Mode::Stats {
        draw_stats_page(f, earned, spent, balance, per_tag);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(1),
        ])
        .split(f.size());

    draw_header(f, chunks[0], earned, spent, balance);
    draw_transactions(f, chunks[1], transactions, app);

    if app.mode == Mode::Adding {
        draw_popup(f, app);
    }
}

/* ---------------- THEME ---------------- */

fn theme() -> (Color, Color, Color, Color) {
    let accent = Color::Cyan;
    let credit = Color::LightGreen;
    let debit = Color::LightRed;
    let muted = Color::Gray;

    (accent, credit, debit, muted)
}

/* ---------------- HEADER ---------------- */

fn draw_header(f: &mut Frame, area: Rect, earned: f64, spent: f64, balance: f64) {
    let (accent, _, _, muted) = theme();

    let text = vec![
        Line::styled(
            "Personal Finance Dashboard",
            Style::default()
                .fg(accent)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
        Line::styled(
            format!(
                "Earned: ₹{:.2}   Spent: ₹{:.2}   Balance: ₹{:.2}",
                earned, spent, balance
            ),
            Style::default().fg(muted),
        ),
    ];

    let header = Paragraph::new(text)
        .block(
            Block::default()
                .title("Overview")
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(accent)),
        )
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

/* ---------------- TRANSACTIONS ---------------- */
fn draw_transactions(f: &mut Frame, area: Rect, transactions: &[Transaction], app: &App) {
    let (accent, credit, debit, muted) = theme();

    let mut items: Vec<ListItem> = Vec::new();

    // ✅ Column Header Row
    items.push(ListItem::new(Line::from(vec![
        Span::styled("Date       ", Style::default().fg(muted)),
        Span::styled("Source         ", Style::default().fg(muted)),
        Span::styled("Amount     ", Style::default().fg(muted)),
        Span::styled("Type      ", Style::default().fg(muted)),
        Span::styled("Tag", Style::default().fg(muted)),
    ])));

    // Divider line
    items.push(ListItem::new(Line::styled(
        "──────────────────────────────────────────────",
        Style::default().fg(muted),
    )));

    // ✅ Transaction Rows
    for tx in transactions {
        let color = match tx.kind {
            TransactionType::Credit => credit,
            TransactionType::Debit => debit,
        };

        // Kind label with selectable look
        let kind_label = match tx.kind {
            TransactionType::Credit => "<CREDIT>",
            TransactionType::Debit => "<DEBIT>",
        };

        // Tag label with selectable look
        let tag_label = format!("<{}>", tx.tag.as_str());

        let line = Line::from(vec![
            // Date
            Span::styled(
                format!("{:<10}", tx.date),
                Style::default().fg(muted),
            ),
            Span::raw("  "),

            // Source
            Span::styled(
                format!("{:<14}", tx.source),
                Style::default().fg(Color::White),
            ),
            Span::raw("  "),

            // Amount
            Span::styled(
                format!("₹{:>8.2}", tx.amount),
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),

            // Kind
            Span::styled(
                format!("{:<8}", kind_label),
                Style::default().fg(color),
            ),
            Span::raw("  "),

            // Tag
            Span::styled(
                tag_label,
                Style::default()
                    .fg(accent)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);

        items.push(ListItem::new(line));
    }

    // ✅ Hint Footer
    items.push(ListItem::new(Line::raw("")));
    items.push(ListItem::new(Line::styled(
        "[↑↓] Navigate   [a] Add   [d] Delete   [s] Stats   [q] Quit",
        Style::default().fg(muted),
    )));

    // Selection State
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected + 2)); 
    // +2 because header + divider take first slots

    let list = List::new(items)
        .block(
            Block::default()
                .title("💳 Transactions")
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(accent)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("  ❯ ");

    f.render_stateful_widget(list, area, &mut state);
}


/* ---------------- 📊 STATS PAGE ---------------- */

fn draw_stats_page(
    f: &mut Frame,
    earned: f64,
    spent: f64,
    balance: f64,
    per_tag: &Vec<(String, f64)>,
) {
    let (accent, credit, debit, muted) = theme();

    let area = f.size();

    // Find max spending for bar scaling
    let max_spent = per_tag
        .iter()
        .map(|(_, v)| *v)
        .fold(0.0, f64::max);

    let mut lines = vec![
        // Title
        Line::styled(
            "📊 Finance Stats Dashboard",
            Style::default()
                .fg(accent)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),

        // Summary Section
        Line::styled(
            "Overview",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Line::raw(format!(
            "  💰 Earned   : ₹{:.2}",
            earned
        )),
        Line::raw(format!(
            "  💸 Spent    : ₹{:.2}",
            spent
        )),
        Line::raw(format!(
            "  📌 Balance  : ₹{:.2}",
            balance
        )),
        Line::raw(""),
        Line::raw("────────────────────────────────────────"),
        Line::raw(""),

        // Breakdown Header
        Line::styled(
            "Spending Breakdown by Tag",
            Style::default()
                .fg(accent)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),
    ];

    // Per-tag breakdown with mini bars
    for (tag, total) in per_tag {
        let bar_width = if max_spent > 0.0 {
            ((total / max_spent) * 12.0).round() as usize
        } else {
            0
        };

        let bar = "█".repeat(bar_width);

        lines.push(Line::from(vec![
            // Tag styled like selectable
            Span::styled(
                format!("<{:<10}>", tag),
                Style::default()
                    .fg(accent)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::raw("  "),
            // Bar indicator
            Span::styled(bar, Style::default().fg(debit)),
            Span::raw(" "),
            // Amount
            Span::styled(
                format!("₹{:.2}", total),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Footer hint
    lines.push(Line::raw(""));
    lines.push(Line::raw("────────────────────────────────────────"));
    lines.push(Line::styled(
        "[Esc] Back   |   Press ↑↓ in Transactions   |   Stats Mode",
        Style::default().fg(muted),
    ));

    let block = Paragraph::new(lines)
        .block(
            Block::default()
                .title("📈 Statistics")
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(accent)),
        )
        .alignment(Alignment::Left);

    f.render_widget(block, area);
}


/* ---------------- POPUP FORM ---------------- */

fn draw_popup(f: &mut Frame, app: &App) {
    let (accent, credit, debit, muted) = theme();
    let area = centered_rect(70, 55, f.size());

    let form = &app.form;

    // Helper: highlight active field
    let highlight = |field: crate::form::Field| {
        if form.active == field {
            Style::default()
                .fg(accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        }
    };

    // Kind shown as selectable
    let kind_style = match form.kind {
        crate::models::TransactionType::Credit => Style::default().fg(credit),
        crate::models::TransactionType::Debit => Style::default().fg(debit),
    };

    let kind_display = format!("<{:?}>", form.kind);
    let tag_display = format!("<{:?}>", form.tag);

    let lines = vec![
        // Title
        Line::styled(
            "➕ Add Transaction",
            Style::default()
                .fg(accent)
                .add_modifier(Modifier::BOLD),
        ),
        Line::raw(""),

        // Source
        Line::from(vec![
            Span::styled("Source : ", Style::default().fg(muted)),
            Span::styled(&form.source, highlight(crate::form::Field::Source)),
        ]),

        // Amount
        Line::from(vec![
            Span::styled("Amount : ", Style::default().fg(muted)),
            Span::styled(&form.amount, highlight(crate::form::Field::Amount)),
        ]),

        Line::raw(""),

        // Kind (Selectable)
        Line::from(vec![
            Span::styled("Type   : ", Style::default().fg(muted)),
            Span::styled(
                kind_display,
                kind_style.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "   ← →",
                Style::default().fg(muted),
            ),
        ]),

        // Tag (Selectable)
        Line::from(vec![
            Span::styled("Tag    : ", Style::default().fg(muted)),
            Span::styled(
                tag_display,
                Style::default()
                    .fg(accent)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::styled(
                "   ← →",
                Style::default().fg(muted),
            ),
        ]),

        Line::raw(""),

        // Date
        Line::from(vec![
            Span::styled("Date   : ", Style::default().fg(muted)),
            Span::styled(&form.date, highlight(crate::form::Field::Date)),
        ]),

        Line::raw(""),
        Line::raw("────────────────────────────────────"),

        // Footer hints
        Line::styled(
            "[Tab] Next Field   [←→] Change Type/Tag   [Enter] Save   [Esc] Cancel",
            Style::default().fg(muted),
        ),
    ];

    let popup = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Transaction Form")
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(accent)),
        )
        .alignment(Alignment::Left);

    // Render popup
    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}


/* ---------------- CENTER RECT ---------------- */

fn centered_rect(px: u16, py: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - py) / 2),
            Constraint::Percentage(py),
            Constraint::Percentage((100 - py) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - px) / 2),
            Constraint::Percentage(px),
            Constraint::Percentage((100 - px) / 2),
        ])
        .split(vert[1])[1]
}
