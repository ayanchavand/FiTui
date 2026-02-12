use ratatui::{
    prelude::*,
    widgets::{Clear, List, ListItem, ListState, Paragraph, Block, Padding, BarChart},
};
use std::collections::HashMap;

use crate::{
    app::{App, Mode},
    form::Field,
    models::{Tag, Transaction, TransactionType},
    theme::Theme,
};

pub fn draw_ui(
    f: &mut Frame,
    transactions: &[Transaction],
    earned: f64,
    spent: f64,
    balance: f64,
    per_tag: &HashMap<Tag, f64>,
    monthly_history: &[(String, f64, f64)],
    tx_count: usize,
    largest: Option<Transaction>,
    smallest: Option<Transaction>,
    top_tags: &[(Tag, f64)],
    app: &App,
) {
    let theme = Theme::default();
    
    match app.mode {
        Mode::Stats => draw_stats_view(f, earned, spent, balance, per_tag, monthly_history, tx_count, largest, smallest, top_tags, &theme, &app.currency),
        Mode::Adding => {
            draw_main_view(f, transactions, earned, spent, balance, app, &theme);
            draw_transaction_form(f, app, &theme);
        }
        _ => draw_main_view(f, transactions, earned, spent, balance, app, &theme),
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
        .margin(1)
        .constraints([Constraint::Length(7), Constraint::Min(1)])
        .split(f.size());

    draw_header(f, chunks[0], earned, spent, balance, theme, &app.currency);
    draw_transactions_list(f, chunks[1], transactions, app, theme);
}

fn draw_header(
    f: &mut Frame,
    area: Rect,
    earned: f64,
    spent: f64,
    balance: f64,
    theme: &Theme,
    currency: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    // Earned Card
    let earned_content = vec![
        Line::from(vec![
            Span::styled("↑ ", Style::default().fg(theme.credit)),
            Span::styled("EARNED", theme.muted_text()),
        ]),
        Line::raw(""),
        Line::styled(
            format!("{}{:.2}", currency, earned),
            Style::default()
                .fg(theme.credit)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    let earned_card = Paragraph::new(earned_content)
        .block(theme.panel())
        .alignment(Alignment::Center);
    f.render_widget(earned_card, chunks[0]);

    // Balance Card (highlighted)
    let balance_color = if balance >= 0.0 {
        theme.credit
    } else {
        theme.debit
    };
    let balance_content = vec![
        Line::styled("BALANCE", theme.title()),
        Line::raw(""),
        Line::styled(
            format!("{}{:.2}", currency, balance),
            Style::default()
                .fg(balance_color)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    ];
    let balance_card = Paragraph::new(balance_content)
        .block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(theme.surface))
        )
        .alignment(Alignment::Center);
    f.render_widget(balance_card, chunks[1]);

    // Spent Card
    let spent_content = vec![
        Line::from(vec![
            Span::styled("↓ ", Style::default().fg(theme.debit)),
            Span::styled("SPENT", theme.muted_text()),
        ]),
        Line::raw(""),
        Line::styled(
            format!("{}{:.2}", currency, spent),
            Style::default()
                .fg(theme.debit)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    let spent_card = Paragraph::new(spent_content)
        .block(theme.panel())
        .alignment(Alignment::Center);
    f.render_widget(spent_card, chunks[2]);
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

    let items = build_transaction_items(transactions, theme, &app.currency);
    let mut state = create_list_state(app.selected);
    
    let list = List::new(items)
        .block(theme.block("📊 Transactions"))
        .highlight_style(theme.highlight_style())
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, layout[0], &mut state);

    // Enhanced footer with better visual hierarchy
    let footer_block = Block::default()
        .borders(ratatui::widgets::Borders::TOP)
        .border_style(Style::default().fg(theme.subtle))
        .style(Style::default().bg(theme.background))
        .padding(Padding::new(1, 1, 0, 0));

    let footer_content = vec![
        Line::from(vec![
            Span::styled("  [", theme.muted_text()),
            Span::styled("↑↓", Style::default().fg(theme.accent)),
            Span::styled("] Navigate  ", theme.muted_text()),
            Span::styled("[", theme.muted_text()),
            Span::styled("a", Style::default().fg(theme.credit)),
            Span::styled("] Add  ", theme.muted_text()),
            Span::styled("[", theme.muted_text()),
            Span::styled("e", Style::default().fg(theme.accent)),
            Span::styled("] Edit  ", theme.muted_text()),
            Span::styled("[", theme.muted_text()),
            Span::styled("d", Style::default().fg(theme.debit)),
            Span::styled("] Delete  ", theme.muted_text()),
            Span::styled("[", theme.muted_text()),
            Span::styled("s", Style::default().fg(theme.accent)),
            Span::styled("] Stats  ", theme.muted_text()),
            Span::styled("[", theme.muted_text()),
            Span::styled("q", Style::default().fg(theme.subtle)),
            Span::styled("] Quit", theme.muted_text()),
        ]),
    ];

    let footer = Paragraph::new(footer_content)
        .block(footer_block);
    f.render_widget(footer, layout[1]);
}

fn build_transaction_items(transactions: &[Transaction], theme: &Theme, currency: &str) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();
    
    items.push(create_table_header(theme));
    items.push(create_divider(theme));
    
    if transactions.is_empty() {
        items.push(ListItem::new(Line::styled(
            "  No transactions yet. Press 'a' to add one!",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        )));
    } else {
        for tx in transactions {
            items.push(create_transaction_row(tx, theme, currency));
        }
    }
    
    items
}

fn create_table_header(theme: &Theme) -> ListItem<'static> {
    ListItem::new(Line::from(vec![
        Span::styled("  Date       ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
        Span::styled("Source          ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
        Span::styled("Amount      ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
        Span::styled("Type      ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
        Span::styled("Tag", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
    ]))
}

fn create_divider(theme: &Theme) -> ListItem<'static> {
    ListItem::new(Line::styled(
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(theme.subtle),
    ))
}

fn create_transaction_row(tx: &Transaction, theme: &Theme, currency: &str) -> ListItem<'static> {
    let color = theme.transaction_color(tx.kind);
    let (icon, kind_label) = match tx.kind {
        TransactionType::Credit => ("↑", "Credit"),
        TransactionType::Debit => ("↓", "Debit"),
    };

    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{:<11}", tx.date),
            Style::default().fg(theme.muted)
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:<15}", truncate_string(&tx.source, 15)),
            Style::default().fg(theme.foreground)
        ),
        Span::raw(" "),
        Span::styled(
            format!("{}{:>9.2}", currency, tx.amount),
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD)
        ),
        Span::raw("  "),
        Span::styled(
            icon,
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:<7}", kind_label),
            Style::default().fg(color)
        ),
        Span::raw(" "),
        Span::styled(
            format!("#{}", tx.tag.as_str()),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC)
        ),
    ]);

    ListItem::new(line)
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn create_list_state(selected: usize) -> ListState {
    let mut state = ListState::default();
    state.select(Some(selected + 2));
    state
}

fn draw_stats_view(
    f: &mut Frame,
    earned: f64,
    spent: f64,
    balance: f64,
    per_tag: &HashMap<Tag, f64>,
    monthly_history: &[(String, f64, f64)],
    tx_count: usize,
    largest: Option<Transaction>,
    smallest: Option<Transaction>,
    top_tags: &[(Tag, f64)],
    theme: &Theme,
    currency: &str,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.size());

    // Split main stats area into top charts and bottom breakdown
    let top_bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(1)])
        .split(layout[0]);

    let charts_area = top_bottom[0];
    let breakdown_area = top_bottom[1];

    // Charts area: left = monthly history, right = top tags
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(charts_area);

    // Prepare monthly bars and sparkline
    let mut month_labels: Vec<String> = Vec::new();
    let mut earned_vals: Vec<u64> = Vec::new();
    let mut spent_vals: Vec<u64> = Vec::new();
    for (m, e, s) in monthly_history.iter().rev() {
        month_labels.push(m.clone());
        earned_vals.push((*e).round().abs() as u64);
        spent_vals.push((*s).round().abs() as u64);
    }

    // Labels as &str for BarChart
    let month_label_refs: Vec<&str> = month_labels.iter().map(|s| s.as_str()).collect();

    // Monthly earned bar chart
    let monthly_earned: Vec<(&str, u64)> = month_label_refs
        .iter()
        .zip(earned_vals.iter())
        .map(|(l, v)| (*l, *v))
        .collect();

    let max_month = earned_vals.iter().chain(spent_vals.iter()).copied().max().unwrap_or(0);

    let earned_chart = BarChart::default()
        .data(&monthly_earned)
        .block(Block::default().title("Monthly Earned").borders(ratatui::widgets::Borders::ALL))
        .max(max_month.max(1))
        .bar_width(7)
        .bar_gap(1)
        .bar_style(Style::default().fg(theme.credit));

    f.render_widget(earned_chart, cols[0]);

    // Right column: top tags bar chart
    let mut tag_labels: Vec<String> = Vec::new();
    let mut tag_vals: Vec<u64> = Vec::new();
    for (t, v) in top_tags.iter().take(6) {
        tag_labels.push(t.as_str().to_string());
        tag_vals.push((*v).round().abs() as u64);
    }
    let tag_label_refs: Vec<&str> = tag_labels.iter().map(|s| s.as_str()).collect();
    let tag_bars: Vec<(&str, u64)> = tag_label_refs
        .iter()
        .zip(tag_vals.iter())
        .map(|(l, v)| (*l, *v))
        .collect();

    let max_tag = tag_vals.iter().copied().max().unwrap_or(0);

    let tags_chart = BarChart::default()
        .data(&tag_bars)
        .block(Block::default().title("Top Tags").borders(ratatui::widgets::Borders::ALL))
        .max(max_tag.max(1))
        .bar_width(6)
        .bar_gap(1)
        .bar_style(Style::default().fg(theme.debit));

    f.render_widget(tags_chart, cols[1]);

    // Below charts: breakdown paragraph (reuse existing content builder for details)
    let breakdown_lines = build_stats_content(earned, spent, balance, per_tag, monthly_history, tx_count, largest, smallest, top_tags, theme, currency);
    let breakdown = Paragraph::new(breakdown_lines)
        .block(theme.block("Details"))
        .alignment(Alignment::Left);

    f.render_widget(breakdown, breakdown_area);

    // Enhanced footer
    let footer_block = Block::default()
        .borders(ratatui::widgets::Borders::TOP)
        .border_style(Style::default().fg(theme.subtle))
        .style(Style::default().bg(theme.background))
        .padding(Padding::new(1, 1, 0, 0));

    let footer = Paragraph::new(Line::styled(
        "  [Esc] Back to Main View",
        Style::default().fg(theme.muted),
    ))
    .block(footer_block)
    .alignment(Alignment::Left);
    
    f.render_widget(footer, layout[1]);
}

fn build_stats_content(
    earned: f64,
    spent: f64,
    balance: f64,
    per_tag: &HashMap<Tag, f64>,
    monthly_history: &[(String, f64, f64)],
    tx_count: usize,
    largest: Option<Transaction>,
    smallest: Option<Transaction>,
    top_tags: &[(Tag, f64)],
    theme: &Theme,
    currency: &str,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::raw(""));
    lines.extend(create_overview_section(earned, spent, balance, theme, currency));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(theme.subtle),
    ));
    lines.push(Line::raw(""));

    // Quick stats summary
    lines.push(Line::styled(
        format!("  Transactions: {}  |  Total Earned: {}{:.2}  |  Total Spent: {}{:.2}", tx_count, currency, earned, currency, spent),
        Style::default().fg(theme.muted),
    ));
    lines.push(Line::raw(""));

    // Monthly history mini-table
    lines.push(Line::styled(
        "  Last Months (YYYY-MM)  Earned      Spent",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::raw(""));
    if monthly_history.is_empty() {
        lines.push(Line::styled(
            "     No monthly data available.",
            Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
        ));
    } else {
        for (m, e, s) in monthly_history {
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(format!("{:<7}", m), Style::default().fg(theme.foreground)),
                Span::raw("  "),
                Span::styled(format!("{}{:>9.2}", currency, e), Style::default().fg(theme.credit)),
                Span::raw("  "),
                Span::styled(format!("{}{:>9.2}", currency, s), Style::default().fg(theme.debit)),
            ]));
        }
    }

    lines.push(Line::raw(""));

    // Top tags
    lines.push(Line::styled(
        "  Top Spending Categories",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::raw(""));
    if top_tags.is_empty() {
        lines.push(Line::styled(
            "     No category data.",
            Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
        ));
    } else {
        for (i, (tag, amt)) in top_tags.iter().take(5).enumerate() {
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(format!("{}. #{:<12}", i + 1, tag.as_str()), Style::default().fg(theme.foreground)),
                Span::raw("  "),
                Span::styled(format!("{}{:>9.2}", currency, amt), Style::default().fg(theme.debit)),
            ]));
        }
    }

    lines.push(Line::raw(""));

    // Largest / Smallest transactions
    lines.push(Line::styled(
        "  Notable Transactions",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::raw(""));
    if let Some(tx) = largest {
        lines.push(Line::from(vec![
            Span::raw("     Largest: "),
            Span::styled(format!("{} | {}{:.2} | #{}", tx.source, currency, tx.amount, tx.tag.as_str()), Style::default().fg(theme.foreground)),
        ]));
    }
    if let Some(tx) = smallest {
        lines.push(Line::from(vec![
            Span::raw("     Smallest: "),
            Span::styled(format!("{} | {}{:.2} | #{}", tx.source, currency, tx.amount, tx.tag.as_str()), Style::default().fg(theme.foreground)),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(theme.subtle),
    ));

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  📊 Spending Breakdown by Category",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::raw(""));
    
    if per_tag.is_empty() {
        lines.push(Line::styled(
            "     No spending data available yet.",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        ));
    } else {
        lines.extend(create_tag_breakdown_section(per_tag, theme, currency));
    }

    lines.push(Line::raw(""));
    lines
}

fn create_overview_section(earned: f64, spent: f64, balance: f64, theme: &Theme, currency: &str) -> Vec<Line<'static>> {
    let balance_color = if balance >= 0.0 { theme.credit } else { theme.debit };
    let savings_rate = if earned > 0.0 {
        ((earned - spent) / earned * 100.0).max(0.0)
    } else {
        0.0
    };

    vec![
        Line::styled(
            "  💰 Financial Overview",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        ),
        Line::raw(""),
        Line::from(vec![
            Span::raw("     Total Earned  : "),
            Span::styled(
                format!("{}{:>10.2}", currency, earned),
                Style::default()
                    .fg(theme.credit)
                    .add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::raw("     Total Spent   : "),
            Span::styled(
                format!("{}{:>10.2}", currency, spent),
                Style::default()
                    .fg(theme.debit)
                    .add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::raw("     Balance       : "),
            Span::styled(
                format!("{}{:>10.2}", currency, balance),
                Style::default()
                    .fg(balance_color)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            ),
        ]),
        Line::from(vec![
            Span::raw("     Savings Rate  : "),
            Span::styled(
                format!("{:>9.1}%", savings_rate),
                Style::default()
                    .fg(if savings_rate > 20.0 { theme.credit } else { theme.accent })
                    .add_modifier(Modifier::BOLD)
            ),
        ]),
    ]
}

fn create_tag_breakdown_section(
    per_tag: &HashMap<Tag, f64>,
    theme: &Theme,
    currency: &str,
) -> Vec<Line<'static>> {
    let mut tag_vec: Vec<_> = per_tag.iter().collect();
    tag_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    
    let max_spent = tag_vec.first().map(|(_, v)| **v).unwrap_or(0.0);
    let total_spent: f64 = per_tag.values().sum();

    let mut lines = Vec::new();
    
    for (tag, &amount) in tag_vec {
        let percentage = if total_spent > 0.0 {
            amount / total_spent * 100.0
        } else {
            0.0
        };
        
        lines.push(create_tag_bar(
            tag.as_str(),
            amount,
            percentage,
            max_spent,
            theme,
            currency,
        ));
    }
    
    lines
}

fn create_tag_bar(
    tag: &str,
    amount: f64,
    percentage: f64,
    max_amount: f64,
    theme: &Theme,
    currency: &str,
) -> Line<'static> {
    let bar_width = calculate_bar_width(amount, max_amount);
    let bar = "█".repeat(bar_width);
    let empty_bar = "░".repeat(20usize.saturating_sub(bar_width));

    Line::from(vec![
        Span::raw("     "),
        Span::styled(
            format!("#{:<12}", tag),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC)
        ),
        Span::raw(" "),
        Span::styled(bar, Style::default().fg(theme.debit)),
        Span::styled(empty_bar, Style::default().fg(theme.subtle)),
        Span::raw("  "),
        Span::styled(
            format!("{}{:>9.2}", currency, amount),
            Style::default()
                .fg(theme.foreground)
                .add_modifier(Modifier::BOLD)
        ),
        Span::raw(" "),
        Span::styled(
            format!("({:>5.1}%)", percentage),
            Style::default().fg(theme.muted)
        ),
    ])
}

fn calculate_bar_width(amount: f64, max_amount: f64) -> usize {
    if max_amount > 0.0 {
        ((amount / max_amount) * 20.0).round() as usize
    } else {
        0
    }
}

fn draw_transaction_form(f: &mut Frame, app: &App, theme: &Theme) {
    let area = centered_rect(60, 60, f.size());
    let form_content = build_form_content(app, theme);

    let title = if app.editing.is_some() {
        "✏️ Edit Transaction"
    } else {
        "➕ Add New Transaction"
    };

    let popup = Paragraph::new(form_content)
        .block(theme.popup(title))
        .alignment(Alignment::Left);

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn build_form_content(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    let form = &app.form;
    
    vec![
        Line::raw(""),
        create_form_field(
            "Source",
            &form.source,
            form.active,
            Field::Source,
            "e.g., Salary, Groceries, etc.",
            theme,
        ),
        Line::raw(""),
        create_form_field(
            "Amount",
            &form.amount,
            form.active,
            Field::Amount,
            "e.g., 1000.50",
            theme,
        ),
        Line::raw(""),
        Line::raw(""),
        create_type_selector(&form.kind, theme),
        Line::raw(""),
        create_tag_selector(&app.tags, form.tag_index, theme),
        Line::raw(""),
        Line::raw(""),
        create_form_field(
            "Date",
            &form.date,
            form.active,
            Field::Date,
            "YYYY-MM-DD",
            theme,
        ),
        Line::raw(""),
        Line::raw(""),
        Line::styled(
            "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(theme.subtle),
        ),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  [", theme.muted_text()),
            Span::styled("Tab", Style::default().fg(theme.accent)),
            Span::styled("] Next Field  ", theme.muted_text()),
            Span::styled("[", theme.muted_text()),
            Span::styled("←→", Style::default().fg(theme.accent)),
            Span::styled("] Change Type/Tag  ", theme.muted_text()),
            Span::styled("[", theme.muted_text()),
            Span::styled("Enter", Style::default().fg(theme.credit)),
            Span::styled("] Save  ", theme.muted_text()),
            Span::styled("[", theme.muted_text()),
            Span::styled("Esc", Style::default().fg(theme.debit)),
            Span::styled("] Cancel", theme.muted_text()),
        ]),
    ]
}

fn create_form_field(
    label: &str,
    value: &str,
    active_field: Field,
    field: Field,
    placeholder: &str,
    theme: &Theme,
) -> Line<'static> {
    let is_active = active_field == field;
    
    let display_value = if value.is_empty() && !is_active {
        placeholder
    } else {
        value
    };

    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        theme.muted_text()
    };

    let value_style = if is_active {
        Style::default()
            .fg(theme.foreground)
            .bg(theme.surface)
            .add_modifier(Modifier::BOLD)
    } else if value.is_empty() {
        Style::default()
            .fg(theme.subtle)
            .add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(theme.foreground)
    };

    let cursor = if is_active { "│" } else { "" };

    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{:<8}", label), label_style),
        Span::raw(": "),
        Span::styled(format!("{}{}", display_value, cursor), value_style),
    ])
}

fn create_type_selector(kind: &TransactionType, theme: &Theme) -> Line<'static> {
    let (kind_icon, kind_label, kind_style) = match kind {
        TransactionType::Credit => ("↑", "Credit (Income)", theme.success()),
        TransactionType::Debit => ("↓", "Debit (Expense)", theme.danger()),
    };

    Line::from(vec![
        Span::raw("  "),
        Span::styled("Type    ", theme.muted_text()),
        Span::raw(": "),
        Span::styled(kind_icon, kind_style),
        Span::raw(" "),
        Span::styled(kind_label, kind_style),
        Span::raw("  "),
        Span::styled("← →", Style::default().fg(theme.accent)),
    ])
}

fn create_tag_selector(tags: &[Tag], index: usize, theme: &Theme) -> Line<'static> {
    let tag = tags.get(index).map(|t| t.as_str()).unwrap_or("other");

    Line::from(vec![
        Span::raw("  "),
        Span::styled("Tag     ", theme.muted_text()),
        Span::raw(": "),
        Span::styled(
            format!("#{}", tag),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD)
        ),
        Span::raw("  "),
        Span::styled("← →", Style::default().fg(theme.accent)),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical_layout[1])[1]
}