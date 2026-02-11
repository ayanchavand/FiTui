use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::{
    models::{Transaction, TransactionType},
    App, Mode,
};

pub fn draw_ui(
    f: &mut Frame,
    transactions: &Vec<Transaction>,
    earned: f64,
    spent: f64,
    balance: f64,
    app: &App,
) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(1),
        ])
        .split(size);

    // ============================
    // Header Stats
    // ============================

    let balance_style = if balance >= 0.0 {
        Style::default().fg(Color::Green).bold()
    } else {
        Style::default().fg(Color::Red).bold()
    };

    let header_text = Line::from(vec![
        Span::styled(" Earned: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("₹{:.2}", earned),
            Style::default().fg(Color::Green).bold(),
        ),
        Span::raw("   "),
        Span::styled("Spent: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("₹{:.2}", spent),
            Style::default().fg(Color::Red).bold(),
        ),
        Span::raw("   "),
        Span::styled("Balance: ", Style::default().fg(Color::Gray)),
        Span::styled(format!("₹{:.2}", balance), balance_style),
    ]);

    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .title(" 📊 Overview ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .alignment(Alignment::Center);

    f.render_widget(header, chunks[0]);

    // ============================
    // Transaction List
    // ============================

    let items: Vec<ListItem> = transactions
        .iter()
        .map(|tx| {
            let amount_style = match tx.kind {
                TransactionType::Credit => Style::default().fg(Color::Green),
                TransactionType::Debit => Style::default().fg(Color::Red),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<12}", tx.date),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:<15}", tx.source),
                    Style::default().fg(Color::White),
                ),
                Span::raw(" "),
                Span::styled(format!("{:>8.2}", tx.amount), amount_style.bold()),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", tx.tag.as_str()),
                    Style::default().fg(Color::Yellow),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" 💰 Transactions (a = add, q = quit) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );

    f.render_widget(list, chunks[1]);

    // ============================
    // Popup Add Form
    // ============================

    if app.mode == Mode::Adding {
        draw_add_popup(f, app);
    }
}

// ============================
// Add Transaction Popup
// ============================

fn draw_add_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 45, f.size());

    let fields = vec![
        ("Source", &app.source),
        ("Amount", &app.amount),
        ("Type", &app.kind),
        ("Tag", &app.tag),
        ("Date", &app.date),
    ];

    let mut lines: Vec<Line> = Vec::new();

    for (i, (label, value)) in fields.iter().enumerate() {
        let style = if i == app.field_index {
            Style::default().fg(Color::Cyan).bold()
        } else {
            Style::default()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{:<8}: ", label), style),
            Span::raw(value.to_string()),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "[Tab=Next]  [Enter=Save]  [Esc=Cancel]",
        Style::default().fg(Color::Gray),
    ));

    let popup = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" ➕ Add Transaction ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .alignment(Alignment::Left);

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

// Helper: Centered popup rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
