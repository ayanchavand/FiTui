use ratatui::{
        prelude::*,
        widgets::{Block, Borders},
    };

    use crate::models::TransactionType;

    #[derive(Clone, Copy)]
    pub struct Theme {
        pub accent: Color,
        pub accent_soft: Color,

        pub credit: Color,
        pub debit: Color,

        pub muted: Color,
        pub subtle: Color,

        pub background: Color,
        pub surface: Color,
        pub row_alt: Color,

        pub foreground: Color,
    }

    impl Theme {
        pub fn default() -> Self {
            Self {
                accent: Color::Rgb(100, 181, 246),
                accent_soft: Color::Rgb(80, 140, 200),

                credit: Color::Rgb(102, 187, 106),
                debit: Color::Rgb(239, 83, 80),

                muted: Color::Rgb(160, 160, 170),
                subtle: Color::Rgb(90, 90, 110),

                background: Color::Rgb(24, 24, 36),
                surface: Color::Rgb(34, 34, 52),
                row_alt: Color::Rgb(29, 29, 44), // midpoint between background and surface

                foreground: Color::Rgb(220, 225, 245),
            }
        }

        pub fn transaction_color(&self, tx_type: TransactionType) -> Color {
            match tx_type {
                TransactionType::Credit => self.credit,
                TransactionType::Debit => self.debit,
            }
        }

        pub fn danger(&self) -> Style {
            Style::default()
                .fg(self.debit)
                .add_modifier(Modifier::BOLD)
        }

        pub fn success(&self) -> Style {
            Style::default()
                .fg(self.credit)
                .add_modifier(Modifier::BOLD)
        }

        pub fn muted_text(&self) -> Style {
            Style::default().fg(self.muted)
        }

        pub fn title(&self) -> Style {
            Style::default()
                .fg(self.accent)
                .add_modifier(Modifier::BOLD)
        }

        pub fn highlight_style(&self) -> Style {
            Style::default()
                .bg(self.surface)
                .fg(self.foreground)
                .add_modifier(Modifier::BOLD)
        }

        pub fn cursor_style(&self) -> Style {
            Style::default()
                .fg(self.accent)
        }

        pub fn block<'a>(&self, title: &'a str) -> Block<'a> {
            Block::default()
                .title(Span::styled(title, self.title()))
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(self.accent_soft))
        }

        pub fn panel<'a>(&self) -> Block<'a> {
            Block::default()
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(self.subtle))
                .style(Style::default().bg(self.background))
        }

        pub fn popup<'a>(&self, title: &'a str) -> Block<'a> {
            Block::default()
                .title(Span::styled(title, self.title()))
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(Style::default().fg(self.accent))
                .style(Style::default().bg(self.surface))
        }

        pub fn separator_span(&self) -> Span<'static> {
            Span::styled(" │ ", Style::default().fg(self.subtle))
        }

        pub fn cursor_indicator(&self) -> Span<'static> {
            Span::styled("►", Style::default().fg(self.accent).add_modifier(Modifier::BOLD))
        }

        pub fn bracket_open(&self) -> Span<'static> {
            Span::styled("[", self.muted_text())
        }

        pub fn bracket_close(&self) -> Span<'static> {
            Span::styled("] ", self.muted_text())
        }

        pub fn dimmed_span<'a>(&self, text: &'a str) -> Span<'a> {
            Span::styled(text, Style::default().fg(self.muted))
        }

        pub fn highlight_span<'a>(&self, text: &'a str) -> Span<'a> {
            Span::styled(text, Style::default().fg(self.accent).add_modifier(Modifier::BOLD))
        }
    }