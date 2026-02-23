use std::collections::{ BTreeMap, HashMap };
use ratatui::{ prelude::*, widgets::{ BarChart, Block, Paragraph } };
use crossterm::event::KeyCode;

use crate::{ app::App, models::{ Tag, Transaction, TransactionType }, theme::Theme };
pub struct StatsSnapshot {
    pub earned: f64,
    pub spent: f64,
    pub balance: f64,

    pub per_tag: HashMap<Tag, f64>,
    pub monthly_history: Vec<(String, f64, f64)>,

    pub tx_count: usize,
    pub largest: Option<Transaction>,
    pub smallest: Option<Transaction>,
    pub top_tags: Vec<(Tag, f64)>,
}

impl StatsSnapshot {
    pub fn new(transactions: &[Transaction]) -> Self {
        let earned = calculate_earned(transactions);
        let spent = calculate_spent(transactions);
        let balance = earned - spent;

        let per_tag = calculate_spent_per_tag(transactions);
        let monthly_history = calculate_monthly_history(transactions);

        let tx_count = transactions.len();
        let largest = get_largest_transaction(transactions);
        let smallest = get_smallest_transaction(transactions);

        let top_tags = get_top_tags(&per_tag);

        Self {
            earned,
            spent,
            balance,
            per_tag,
            monthly_history,
            tx_count,
            largest,
            smallest,
            top_tags,
        }
    }
}

// ============================================================================
// Stats calculation functions
// ============================================================================

/// Calculate total earned transactions from app transactions
pub fn calculate_earned(transactions: &[Transaction]) -> f64 {
    transactions
        .iter()
        .filter(|tx| tx.kind == TransactionType::Credit)
        .map(|tx| tx.amount)
        .sum()
}

/// Calculate total spent transactions from app transactions
pub fn calculate_spent(transactions: &[Transaction]) -> f64 {
    transactions
        .iter()
        .filter(|tx| tx.kind == TransactionType::Debit)
        .map(|tx| tx.amount)
        .sum()
}

/// Build a map of spending per tag from all debit transactions
pub fn calculate_spent_per_tag(transactions: &[Transaction]) -> HashMap<Tag, f64> {
    let mut map = HashMap::new();
    for tx in transactions.iter().filter(|tx| tx.kind == TransactionType::Debit) {
        *map.entry(tx.tag.clone()).or_insert(0.0) += tx.amount;
    }
    map
}

/// Get the largest transaction by amount
pub fn get_largest_transaction(transactions: &[Transaction]) -> Option<Transaction> {
    transactions
        .iter()
        .max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal))
        .cloned()
}

/// Get the smallest transaction by amount
pub fn get_smallest_transaction(transactions: &[Transaction]) -> Option<Transaction> {
    transactions
        .iter()
        .min_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap_or(std::cmp::Ordering::Equal))
        .cloned()
}

/// Get top tags sorted by spending amount
pub fn get_top_tags(per_tag: &HashMap<Tag, f64>) -> Vec<(Tag, f64)> {
    let mut top_tags: Vec<(Tag, f64)> = per_tag
        .iter()
        .map(|(t, v)| (t.clone(), *v))
        .collect();
    top_tags.sort_by(|a, b| { b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal) });
    top_tags
}

/// Calculate monthly history grouped by month (YYYY-MM)
/// Returns up to 6 most recent months with (month, earned, spent) tuples
pub fn calculate_monthly_history(transactions: &[Transaction]) -> Vec<(String, f64, f64)> {
    let mut monthly_map: BTreeMap<String, (f64, f64)> = BTreeMap::new();

    for tx in transactions {
        let month = if tx.date.len() >= 7 { tx.date[..7].to_string() } else { tx.date.clone() };

        let entry = monthly_map.entry(month).or_insert((0.0, 0.0));

        match tx.kind {
            TransactionType::Credit => {
                entry.0 += tx.amount;
            }
            TransactionType::Debit => {
                entry.1 += tx.amount;
            }
        }
    }

    monthly_map
        .into_iter()
        .rev()
        .take(6)
        .map(|(m, (e, s))| (m, e, s))
        .collect()
}

// ============================================================================
// Stats UI rendering functions
// ============================================================================

pub fn draw_stats_view(f: &mut Frame, snapshot: &StatsSnapshot, theme: &Theme, currency: &str) {
    let earned = snapshot.earned;
    let spent = snapshot.spent;
    let balance = snapshot.balance;

    let per_tag = &snapshot.per_tag;
    let monthly_history = &snapshot.monthly_history;

    let tx_count = snapshot.tx_count;

    // Clone only the Options (for now)
    let largest = snapshot.largest.clone();
    let smallest = snapshot.smallest.clone();

    let top_tags = &snapshot.top_tags;

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
    let month_label_refs: Vec<&str> = month_labels
        .iter()
        .map(|s| s.as_str())
        .collect();

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
    let tag_label_refs: Vec<&str> = tag_labels
        .iter()
        .map(|s| s.as_str())
        .collect();
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
    let breakdown_lines = build_stats_content(
        earned,
        spent,
        balance,
        per_tag,
        monthly_history,
        tx_count,
        largest,
        smallest,
        top_tags,
        theme,
        currency
    );
    let breakdown = Paragraph::new(breakdown_lines)
        .block(theme.block("Details"))
        .alignment(Alignment::Left);

    f.render_widget(breakdown, breakdown_area);

    // Enhanced footer
    let footer_block = Block::default()
        .borders(ratatui::widgets::Borders::TOP)
        .border_style(Style::default().fg(theme.subtle))
        .style(Style::default().bg(theme.background))
        .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));

    let footer = Paragraph::new(
        Line::styled("  [Esc] Back to Main View", Style::default().fg(theme.muted))
    )
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
    currency: &str
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::raw(""));
    lines.extend(create_overview_section(earned, spent, balance, theme, currency));
    lines.push(Line::raw(""));
    lines.push(
        Line::styled(
            "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(theme.subtle)
        )
    );
    lines.push(Line::raw(""));

    // Quick stats summary
    lines.push(
        Line::styled(
            format!(
                "  Transactions: {}  |  Total Earned: {}{:.2}  |  Total Spent: {}{:.2}",
                tx_count,
                currency,
                earned,
                currency,
                spent
            ),
            Style::default().fg(theme.muted)
        )
    );
    lines.push(Line::raw(""));

    // Monthly history mini-table
    lines.push(
        Line::styled(
            "  Last Months (YYYY-MM)  Earned      Spent",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        )
    );
    lines.push(Line::raw(""));
    if monthly_history.is_empty() {
        lines.push(
            Line::styled(
                "     No monthly data available.",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC)
            )
        );
    } else {
        for (m, e, s) in monthly_history {
            lines.push(
                Line::from(
                    vec![
                        Span::raw("     "),
                        Span::styled(format!("{:<7}", m), Style::default().fg(theme.foreground)),
                        Span::raw("  "),
                        Span::styled(
                            format!("{}{:>9.2}", currency, e),
                            Style::default().fg(theme.credit)
                        ),
                        Span::raw("  "),
                        Span::styled(
                            format!("{}{:>9.2}", currency, s),
                            Style::default().fg(theme.debit)
                        )
                    ]
                )
            );
        }
    }

    lines.push(Line::raw(""));

    // Top tags
    lines.push(
        Line::styled(
            "  Top Spending Categories",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        )
    );
    lines.push(Line::raw(""));
    if top_tags.is_empty() {
        lines.push(
            Line::styled(
                "     No category data.",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC)
            )
        );
    } else {
        for (i, (tag, amt)) in top_tags.iter().take(5).enumerate() {
            lines.push(
                Line::from(
                    vec![
                        Span::raw("     "),
                        Span::styled(
                            format!("{}. #{:<12}", i + 1, tag.as_str()),
                            Style::default().fg(theme.foreground)
                        ),
                        Span::raw("  "),
                        Span::styled(
                            format!("{}{:>9.2}", currency, amt),
                            Style::default().fg(theme.debit)
                        )
                    ]
                )
            );
        }
    }

    lines.push(Line::raw(""));

    // Largest / Smallest transactions
    lines.push(
        Line::styled(
            "  Notable Transactions",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        )
    );
    lines.push(Line::raw(""));
    if let Some(tx) = largest {
        lines.push(
            Line::from(
                vec![
                    Span::raw("     Largest: "),
                    Span::styled(
                        format!(
                            "{} | {}{:.2} | #{}",
                            tx.source,
                            currency,
                            tx.amount,
                            tx.tag.as_str()
                        ),
                        Style::default().fg(theme.foreground)
                    )
                ]
            )
        );
    }
    if let Some(tx) = smallest {
        lines.push(
            Line::from(
                vec![
                    Span::raw("     Smallest: "),
                    Span::styled(
                        format!(
                            "{} | {}{:.2} | #{}",
                            tx.source,
                            currency,
                            tx.amount,
                            tx.tag.as_str()
                        ),
                        Style::default().fg(theme.foreground)
                    )
                ]
            )
        );
    }

    lines.push(Line::raw(""));
    lines.push(
        Line::styled(
            "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(theme.subtle)
        )
    );

    lines.push(Line::raw(""));
    lines.push(
        Line::styled(
            "  📊 Spending Breakdown by Category",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        )
    );
    lines.push(Line::raw(""));

    if per_tag.is_empty() {
        lines.push(
            Line::styled(
                "     No spending data available yet.",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC)
            )
        );
    } else {
        lines.extend(create_tag_breakdown_section(per_tag, theme, currency));
    }

    lines.push(Line::raw(""));
    lines
}

fn create_overview_section(
    earned: f64,
    spent: f64,
    balance: f64,
    theme: &Theme,
    currency: &str
) -> Vec<Line<'static>> {
    let balance_color = if balance >= 0.0 { theme.credit } else { theme.debit };
    let savings_rate = if earned > 0.0 {
        (((earned - spent) / earned) * 100.0).max(0.0)
    } else {
        0.0
    };

    vec![
        Line::styled(
            "  💰 Financial Overview",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        ),
        Line::raw(""),
        Line::from(
            vec![
                Span::raw("     Total Earned  : "),
                Span::styled(
                    format!("{}{:>10.2}", currency, earned),
                    Style::default().fg(theme.credit).add_modifier(Modifier::BOLD)
                )
            ]
        ),
        Line::from(
            vec![
                Span::raw("     Total Spent   : "),
                Span::styled(
                    format!("{}{:>10.2}", currency, spent),
                    Style::default().fg(theme.debit).add_modifier(Modifier::BOLD)
                )
            ]
        ),
        Line::from(
            vec![
                Span::raw("     Balance       : "),
                Span::styled(
                    format!("{}{:>10.2}", currency, balance),
                    Style::default()
                        .fg(balance_color)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                )
            ]
        ),
        Line::from(
            vec![
                Span::raw("     Savings Rate  : "),
                Span::styled(
                    format!("{:>9.1}%", savings_rate),
                    Style::default()
                        .fg(if savings_rate > 20.0 { theme.credit } else { theme.accent })
                        .add_modifier(Modifier::BOLD)
                )
            ]
        )
    ]
}

fn create_tag_breakdown_section(
    per_tag: &HashMap<Tag, f64>,
    theme: &Theme,
    currency: &str
) -> Vec<Line<'static>> {
    let mut tag_vec: Vec<_> = per_tag.iter().collect();
    tag_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    let max_spent = tag_vec
        .first()
        .map(|(_, v)| **v)
        .unwrap_or(0.0);
    let total_spent: f64 = per_tag.values().sum();

    let mut lines = Vec::new();

    for (tag, &amount) in tag_vec {
        let percentage = if total_spent > 0.0 { (amount / total_spent) * 100.0 } else { 0.0 };

        lines.push(create_tag_bar(tag.as_str(), amount, percentage, max_spent, theme, currency));
    }

    lines
}

fn create_tag_bar(
    tag: &str,
    amount: f64,
    percentage: f64,
    max_amount: f64,
    theme: &Theme,
    currency: &str
) -> Line<'static> {
    let bar_width = calculate_bar_width(amount, max_amount);
    let bar = "█".repeat(bar_width);
    let empty_bar = "░".repeat((20usize).saturating_sub(bar_width));

    Line::from(
        vec![
            Span::raw("     "),
            Span::styled(
                format!("#{:<12}", tag),
                Style::default().fg(theme.accent_soft).add_modifier(Modifier::ITALIC)
            ),
            Span::raw(" "),
            Span::styled(bar, Style::default().fg(theme.debit)),
            Span::styled(empty_bar, Style::default().fg(theme.subtle)),
            Span::raw("  "),
            Span::styled(
                format!("{}{:>9.2}", currency, amount),
                Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)
            ),
            Span::raw(" "),
            Span::styled(format!("({:>5.1}%)", percentage), Style::default().fg(theme.muted))
        ]
    )
}

fn calculate_bar_width(amount: f64, max_amount: f64) -> usize {
    if max_amount > 0.0 { ((amount / max_amount) * 20.0).round() as usize } else { 0 }
}

// ============================================================================
// Stats input handler
// ============================================================================

pub fn handle_stats(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Esc => {
            app.mode = crate::app::Mode::Normal;
        }
        _ => {}
    }

    false
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Transaction, TransactionType, Tag};

    fn tx(id: i32, source: &str, amount: f64, kind: TransactionType, tag: &str, date: &str) -> Transaction {
        Transaction {
            id,
            source: source.to_string(),
            amount,
            kind,
            tag: Tag::from_str(tag),
            date: date.to_string(),
        }
    }

    #[test]
    fn calculate_earned_and_spent() {
        let transactions = vec![
            tx(1, "a", 200.0, TransactionType::Credit, "salary", "2026-02-01"),
            tx(2, "b", 50.0, TransactionType::Debit, "food", "2026-02-02"),
            tx(3, "c", 25.0, TransactionType::Debit, "misc", "2026-02-03"),
        ];

        let earned = calculate_earned(&transactions);
        let spent = calculate_spent(&transactions);

        assert_eq!(earned, 200.0);
        assert_eq!(spent, 75.0);
    }

    #[test]
    fn monthly_history_groups_and_orders() {
        let transactions = vec![
            tx(1, "a", 100.0, TransactionType::Credit, "x", "2026-01-05"),
            tx(2, "b", 30.0, TransactionType::Debit, "y", "2026-02-10"),
            tx(3, "c", 50.0, TransactionType::Credit, "x", "2026-02-15"),
            tx(4, "d", 20.0, TransactionType::Debit, "z", "2025-12-31"),
        ];

        let months = calculate_monthly_history(&transactions);

        // Expect descending months (latest first)
        assert!(months.len() >= 3);
        assert_eq!(months[0].0, "2026-02");
        // Check earned/spent values for 2026-02
        let feb = &months[0];
        assert_eq!(feb.1, 50.0); // earned
        assert_eq!(feb.2, 30.0); // spent
    }

    #[test]
    fn top_tags_sort_descending() {
        let mut map = std::collections::HashMap::new();
        map.insert(Tag::from_str("a"), 10.0);
        map.insert(Tag::from_str("b"), 50.0);
        map.insert(Tag::from_str("c"), 30.0);

        let top = get_top_tags(&map);
        assert_eq!(top[0].0, Tag::from_str("b"));
        assert_eq!(top[1].0, Tag::from_str("c"));
        assert_eq!(top[2].0, Tag::from_str("a"));
    }

    #[test]
    fn calculate_bar_width_handles_zero_max() {
        assert_eq!(calculate_bar_width(0.0, 0.0), 0);
        assert!(calculate_bar_width(5.0, 10.0) > 0);
    }
}
