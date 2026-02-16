use ratatui::{
    prelude::*,
    widgets::{Block, Clear, List, ListItem, ListState, Padding, Paragraph},
};

use crate::{
    app::{App, Mode, PopupKind},
    form::Field,
    models::{Transaction, TransactionType},
    stats,
    stats::StatsSnapshot,
    theme::Theme,
};

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

fn draw_popup(f: &mut Frame, app: &App, theme: &Theme) {
    if let Some(popup) = &app.popup {
        let area = centered_rect(60, 30, f.size());

        // Clear behind popup with slight shadow effect
        f.render_widget(Clear, area);

        let (title, lines, is_confirm) = match popup {
            PopupKind::Confirm { title, message, .. } => {
                (title.as_str(), message.clone(), true)
            }
            PopupKind::Info { title, message } => {
                (title.as_str(), message.clone(), false)
            }
        };

        // Enhanced styled button row with better visual separation
        let buttons = if is_confirm {
            Line::from(vec![
                Span::raw("   "),
                Span::styled("[", theme.muted_text()),
                Span::styled("y", Style::default().fg(theme.credit).add_modifier(Modifier::BOLD)),
                Span::styled("] ", theme.muted_text()),
                Span::styled("Yes", theme.success()),
                Span::raw("      "),
                Span::styled("[", theme.muted_text()),
                Span::styled("n", Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)),
                Span::styled("] ", theme.muted_text()),
                Span::styled("No", theme.danger()),
            ])
        } else {
            Line::from(vec![
                Span::styled("[", theme.muted_text()),
                Span::styled("Esc", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("] ", theme.muted_text()),
                Span::styled("Close", theme.muted_text()),
            ])
        };

        // Popup content with enhanced spacing and hierarchy
        let content = vec![
            Line::raw(""),
            Line::raw(""),
            Line::styled(
                lines,
                Style::default()
                    .fg(theme.foreground)
                    .add_modifier(Modifier::BOLD)
            ),
            Line::raw(""),
            Line::raw(""),
            Line::styled(
                "─────────────────────────────────────────────────",
                Style::default().fg(theme.subtle),
            ),
            Line::raw(""),
            buttons,
            Line::raw(""),
        ];

        let widget = Paragraph::new(content)
            .block(theme.popup(title))
            .alignment(Alignment::Center);

        f.render_widget(widget, area);
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

fn draw_header(
    f: &mut Frame,
    area: Rect,
    earned: f64,
    spent: f64,
    balance: f64,
    theme: &Theme,
    currency: &str,
) {
    // Add margin for centering
    let margin_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(area);

    let horizontal_margin = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(margin_layout[1]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(horizontal_margin[1]);

    // Enhanced EARNED panel with better visual hierarchy
    let earned_content = vec![
        Line::from(vec![
            Span::styled("↑ ", Style::default().fg(theme.credit).add_modifier(Modifier::BOLD)),
            Span::styled("EARNED", theme.title()),
        ]),
        Line::raw(""),
        Line::styled(
            format!("{}{:.2}", currency, earned),
            Style::default()
                .fg(theme.credit)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    f.render_widget(
        Paragraph::new(earned_content)
            .block(theme.panel())
            .alignment(Alignment::Center),
        chunks[0],
    );

    // Enhanced BALANCE panel with dynamic styling
    let balance_color = if balance >= 0.0 {
        theme.credit
    } else {
        theme.debit
    };
    let balance_symbol = if balance >= 0.0 { "✓" } else { "⚠" };
    
    let balance_content = vec![
        Line::from(vec![
            Span::styled(balance_symbol, Style::default().fg(balance_color).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled("BALANCE", theme.title()),
        ]),
        Line::raw(""),
        Line::styled(
            format!("{}{:.2}", currency, balance),
            Style::default()
                .fg(balance_color)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    ];
    f.render_widget(
        Paragraph::new(balance_content)
            .block(
                Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_set(ratatui::symbols::border::ROUNDED)
                    .border_style(Style::default().fg(theme.accent))
                    .style(Style::default().bg(theme.surface))
            )
            .alignment(Alignment::Center),
        chunks[1],
    );

    // Enhanced SPENT panel with better visual hierarchy
    let spent_content = vec![
        Line::from(vec![
            Span::styled("↓ ", Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)),
            Span::styled("SPENT", theme.title()),
        ]),
        Line::raw(""),
        Line::styled(
            format!("{}{:.2}", currency, spent),
            Style::default()
                .fg(theme.debit)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    f.render_widget(
        Paragraph::new(spent_content)
            .block(theme.panel())
            .alignment(Alignment::Center),
        chunks[2],
    );
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
        .block(theme.block(" 💰 Transactions "))
        .highlight_style(theme.highlight_style())
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, layout[0], &mut state);

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

fn draw_transaction_form(f: &mut Frame, app: &App, theme: &Theme) {
    let area = centered_rect(65, 65, f.size());
    let form_content = build_form_content(app, theme);

    let title = if app.editing.is_some() {
        " ✏️  Edit Transaction "
    } else {
        " ➕ Add New Transaction "
    };

    let popup = Paragraph::new(form_content)
        .block(theme.popup(title))
        .alignment(Alignment::Left);

    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

/* KEEP ALL YOUR EXISTING HELPERS BELOW UNCHANGED:
   - build_transaction_items
   - create_table_header
   - create_divider
   - create_transaction_row
   - truncate_string
   - create_list_state
   - build_form_content
   - create_form_field
   - create_type_selector
   - create_tag_selector
   - create_recurring_selector
*/
fn build_transaction_items(
    transactions: &[Transaction],
    theme: &Theme,
    currency: &str,
) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();
    items.push(create_table_header(theme));
    items.push(create_divider(theme));
    if transactions.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("   "),
            Span::styled("📋 ", Style::default().fg(theme.accent)),
            Span::styled(
                "No transactions yet. Press ",
                Style::default()
                    .fg(theme.muted)
                    .add_modifier(Modifier::ITALIC)
            ),
            Span::styled(
                "'a'",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            ),
            Span::styled(
                " to add one!",
                Style::default()
                    .fg(theme.muted)
                    .add_modifier(Modifier::ITALIC)
            ),
        ])));
    } else {
        for tx in transactions {
            items.push(create_transaction_row(tx, theme, currency));
        }
    }
    items
}

fn create_table_header(theme: &Theme) -> ListItem<'static> {
    ListItem::new(Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "📅 Date ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "│ Source ",
            Style::default()
                .fg(theme.subtle)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "│ Amount ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "│ Type ",
            Style::default()
                .fg(theme.subtle)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "│ Tag",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
}

fn create_divider(theme: &Theme) -> ListItem<'static> {
    ListItem::new(Line::styled(
        " ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
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
        Span::raw(" "),
        Span::styled(
            format!("{:<11}", tx.date),
            Style::default().fg(theme.muted)
        ),
        Span::styled(" │ ", Style::default().fg(theme.subtle)),
        Span::styled(
            format!("{:<15}", truncate_string(&tx.source, 15)),
            Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme.subtle)),
        Span::styled(
            format!("{}{:>9.2}", currency, tx.amount),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme.subtle)),
        Span::styled(
            icon,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:<7}", kind_label),
            Style::default().fg(color)
        ),
        Span::styled(" │ ", Style::default().fg(theme.subtle)),
        Span::styled(
            format!("#{}", tx.tag.as_str()),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
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

fn build_form_content(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    let form = &app.form;
    vec![
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("📝 ", Style::default().fg(theme.accent)),
            Span::styled(
                "Fill in the transaction details below",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC)
            ),
        ]),
        Line::raw(""),
        Line::styled(
            "  ───────────────────────────────────────────────────",
            Style::default().fg(theme.subtle),
        ),
        Line::raw(""),
        create_form_field(
            "Source",
            &form.source,
            form.active,
            Field::Source,
            "e.g., Salary, Groceries, Rent",
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
        create_form_field(
            "Date",
            &form.date,
            form.active,
            Field::Date,
            "YYYY-MM-DD",
            theme,
        ),
        Line::raw(""),
        Line::styled(
            "  ───────────────────────────────────────────────────",
            Style::default().fg(theme.subtle),
        ),
        Line::raw(""),
        create_type_selector(&form.kind, form.active == Field::Kind, theme),
        Line::raw(""),
        create_tag_selector(&app.tags, form.tag_index, form.active == Field::Tag, theme),
        Line::raw(""),
        create_recurring_selector(form.recurring, form.active == Field::Recurring, theme),
        Line::raw(""),
        Line::styled(
            "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(theme.accent_soft),
        ),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[", theme.muted_text()),
            Span::styled("Tab", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("] Next  ", theme.muted_text()),
            
            Span::styled("[", theme.muted_text()),
            Span::styled("←→", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("] Toggle  ", theme.muted_text()),
            
            Span::styled("[", theme.muted_text()),
            Span::styled("Enter", theme.success()),
            Span::styled("] Save  ", theme.muted_text()),
            
            Span::styled("[", theme.muted_text()),
            Span::styled("Esc", theme.danger()),
            Span::styled("] Cancel", theme.muted_text()),
        ]),
        Line::raw(""),
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
        placeholder.to_string()
    } else {
        value.to_string()
    };
    
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
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
    
    let cursor = if is_active { 
        Span::styled("│", theme.cursor_style())
    } else { 
        Span::raw("")
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    Line::from(vec![
        indicator,
        Span::styled(format!("{:<9}", label), label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(display_value, value_style),
        cursor,
    ])
}

fn create_type_selector(
    kind: &crate::models::TransactionType,
    is_active: bool,
    theme: &Theme,
) -> Line<'static> {
    let (kind_icon, kind_label, kind_style) = match kind {
        crate::models::TransactionType::Credit => ("↑", "Credit (Income)", theme.success()),
        crate::models::TransactionType::Debit => ("↓", "Debit (Expense)", theme.danger()),
    };
    
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.muted_text()
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    Line::from(vec![
        indicator,
        Span::styled("Type     ", label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(kind_icon, kind_style.add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(kind_label, kind_style),
        Span::raw("  "),
        Span::styled(
            "← →",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        ),
    ])
}

fn create_tag_selector(
    tags: &[crate::models::Tag],
    index: usize,
    is_active: bool,
    theme: &Theme,
) -> Line<'static> {
    let tag = tags.get(index).map(|t| t.as_str()).unwrap_or("other");
    
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.muted_text()
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    Line::from(vec![
        indicator,
        Span::styled("Tag      ", label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(
            format!("#{}", tag),
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "← →",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        ),
    ])
}

fn create_recurring_selector(recurring: bool, is_active: bool, theme: &Theme) -> Line<'static> {
    let (status_icon, status_text, status_style) = if recurring {
        ("🔄", "Yes", theme.success())
    } else {
        ("🚫", "No", Style::default().fg(theme.muted))
    };
    
    let label_style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.muted_text()
    };
    
    let indicator = if is_active {
        Span::styled("▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("  ")
    };
    
    Line::from(vec![
        indicator,
        Span::styled("Recurring", label_style),
        Span::styled("│ ", Style::default().fg(theme.subtle)),
        Span::styled(status_icon, status_style),
        Span::raw(" "),
        Span::styled(status_text, status_style),
        Span::raw("  "),
        Span::styled(
            "← →",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        ),
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
