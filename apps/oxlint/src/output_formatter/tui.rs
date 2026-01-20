use std::io;

use crate::output_formatter::{FormattedRule, InternalFormatter, get_formatted_rules};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use oxc_diagnostics::{
    Error, GraphicalReportHandler,
    reporter::{DiagnosticReporter, DiagnosticResult},
};
use ratatui::{prelude::*, widgets::*};
use rustc_hash::FxHashMap;

#[derive(Debug, Default)]
pub struct TuiOutputFormatter;

impl InternalFormatter for TuiOutputFormatter {
    fn all_rules(&self) -> Option<String> {
        let rules = get_formatted_rules();

        let _ = run_rule_explorer_tui(rules);
        None
    }

    fn lint_command_info(&self, _lint_command_info: &super::LintCommandInfo) -> Option<String> {
        None
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(TuiReporterWrapper::default())
    }
}

enum ActivePane {
    Categories,
    Rules,
}

struct TuiAppState {
    categories: Vec<String>,
    rules_by_category: FxHashMap<String, Vec<FormattedRule>>,
    active_pane: ActivePane,
    category_list_state: ListState,
    rule_list_state: ListState,
}

impl TuiAppState {
    fn new(rules: Vec<FormattedRule>) -> Self {
        let mut rules_by_category: FxHashMap<String, Vec<FormattedRule>> = FxHashMap::default();

        for rule in rules {
            rules_by_category.entry(rule.category.to_string()).or_default().push(rule);
        }

        let mut categories: Vec<String> = rules_by_category.keys().cloned().collect();
        categories.sort();

        for list in rules_by_category.values_mut() {
            list.sort_by(|a, b| a.name.cmp(b.name));
        }

        let mut cat_state = ListState::default();
        cat_state.select(Some(0));
        let mut rule_state = ListState::default();
        rule_state.select(Some(0));

        Self {
            categories,
            rules_by_category,
            active_pane: ActivePane::Categories,
            category_list_state: cat_state,
            rule_list_state: rule_state,
        }
    }

    fn current_category(&self) -> &str {
        if self.categories.is_empty() {
            return "";
        }
        &self.categories[self.category_list_state.selected().unwrap_or(0)]
    }

    fn current_rules(&self) -> &[FormattedRule] {
        self.rules_by_category.get(self.current_category()).map_or(&[], |v| v.as_slice())
    }

    fn current_rule(&self) -> Option<&FormattedRule> {
        self.current_rules().get(self.rule_list_state.selected().unwrap_or(0))
    }
}

fn run_rule_explorer_tui(rules: Vec<FormattedRule>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = TuiAppState::new(rules);

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(30),
                    Constraint::Percentage(50),
                ])
                .split(area);

            // LEFT PANE (CATEGORIES)
            let cat_items: Vec<ListItem> =
                app.categories.iter().map(|c| ListItem::new(Line::from(c.as_str()))).collect();

            let cat_style = if matches!(app.active_pane, ActivePane::Categories) {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let cat_list = List::new(cat_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Categories ")
                        .border_style(cat_style),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");

            f.render_stateful_widget(cat_list, chunks[0], &mut app.category_list_state);

            // MIDDLE PANE (RULES)
            let rule_items: Vec<ListItem> = {
                let current_rules = app.current_rules();
                current_rules.iter().map(|r| ListItem::new(Line::from(r.name))).collect()
            };

            let rule_style = if matches!(app.active_pane, ActivePane::Rules) {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let rules_list = List::new(rule_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Rules ")
                        .border_style(rule_style),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("> ");

            f.render_stateful_widget(rules_list, chunks[1], &mut app.rule_list_state);

            // RIGHT PANE (DETAILS)
            let detail_block = Block::default().borders(Borders::ALL).title(" Details ");

            if let Some(rule) = app.current_rule() {
                let text = vec![
                    Line::from(vec![
                        Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(rule.name),
                    ]),
                    Line::from(vec![
                        Span::styled("Scope: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(rule.scope),
                    ]),
                    Line::from(vec![
                        Span::styled("Category: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(rule.category.to_string()),
                    ]),
                    Line::from(vec![
                        Span::styled("Default On: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(if rule.is_default { "Yes" } else { "No" }),
                    ]),
                    Line::from(vec![
                        Span::styled("Auto-Fix: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(&rule.fix), // Borrow String
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Info: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(rule.description()),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Docs: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            &rule.docs_url,
                            Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Use ←/→ or h/l to switch panes. Use ↓/↑ or j/k to select. 'q' to quit.",
                        Style::default().fg(Color::DarkGray),
                    )),
                ];

                let p = Paragraph::new(text).block(detail_block).wrap(Wrap { trim: true });
                f.render_widget(p, chunks[2]);
            } else {
                f.render_widget(Paragraph::new("No rule selected").block(detail_block), chunks[2]);
            }
        })?;

        // INPUT HANDLERS
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,

                // Switch Pane
                KeyCode::Left | KeyCode::Char('h') => {
                    app.active_pane = ActivePane::Categories;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    app.active_pane = ActivePane::Rules;
                }

                // Navigate List
                KeyCode::Down | KeyCode::Char('j') => match app.active_pane {
                    ActivePane::Categories => {
                        let i = app.category_list_state.selected().unwrap_or(0);
                        let next = if i >= app.categories.len() - 1 { 0 } else { i + 1 };
                        app.category_list_state.select(Some(next));
                        app.rule_list_state.select(Some(0));
                    }
                    ActivePane::Rules => {
                        let len = app.current_rules().len();
                        if len > 0 {
                            let i = app.rule_list_state.selected().unwrap_or(0);
                            let next = if i >= len - 1 { 0 } else { i + 1 };
                            app.rule_list_state.select(Some(next));
                        }
                    }
                },

                KeyCode::Up | KeyCode::Char('k') => match app.active_pane {
                    ActivePane::Categories => {
                        let i = app.category_list_state.selected().unwrap_or(0);
                        let next = if i == 0 { app.categories.len() - 1 } else { i - 1 };
                        app.category_list_state.select(Some(next));
                        app.rule_list_state.select(Some(0));
                    }
                    ActivePane::Rules => {
                        let len = app.current_rules().len();
                        if len > 0 {
                            let i = app.rule_list_state.selected().unwrap_or(0);
                            let next = if i == 0 { len - 1 } else { i - 1 };
                            app.rule_list_state.select(Some(next));
                        }
                    }
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

#[derive(Default)]
struct TuiReporterWrapper {
    handler: GraphicalReportHandler,
}

impl DiagnosticReporter for TuiReporterWrapper {
    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        None
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let mut output = String::new();
        self.handler.render_report(&mut output, error.as_ref()).unwrap();
        Some(output)
    }
}
