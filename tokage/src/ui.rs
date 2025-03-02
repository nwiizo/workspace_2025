use crate::app::App;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line as TextLine, Span},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs, Wrap,
        canvas::{Canvas, Line, Rectangle},
    },
    Frame,
};
use similar::ChangeTag;

pub fn render_ui<B: Backend>(frame: &mut Frame, app: &App) {
    let size = frame.area();
    
    if app.show_help {
        // Show help overlay
        render_help::<B>(frame, size);
    } else {
        // Main UI
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),  // Title
                    Constraint::Length(3),  // Tabs
                    Constraint::Min(0),     // Content
                    Constraint::Length(1),  // Status bar
                ]
                .as_ref(),
            )
            .split(size);
        
        // Title
        render_title::<B>(frame, main_chunks[0]);
        
        // Tabs
        render_tabs::<B>(frame, main_chunks[1], app);
        
        // Content based on selected tab
        match app.tab_index {
            0 => render_results_tab::<B>(frame, main_chunks[2], app),
            1 => render_stats_tab::<B>(frame, main_chunks[2], app),
            2 => render_diff_tab::<B>(frame, main_chunks[2], app),
            _ => {}
        }
        
        // Status bar
        render_status_bar::<B>(frame, main_chunks[3]);
    }
}

fn render_title<B: Backend>(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![
        TextLine::from(vec![
            Span::styled("TO", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("KA", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled("GE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - ", Style::default().fg(Color::White)),
            Span::styled("Test Observer & Runner Interface", Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)),
        ]),
        TextLine::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("?", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" for help", Style::default().fg(Color::Gray)),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
    );
    
    frame.render_widget(title, area);
}

fn render_tabs<B: Backend>(frame: &mut Frame, area: Rect, app: &App) {
    let titles = vec!["Test Results", "Statistics", "Diff View"];
    let tabs = Tabs::new(titles.iter().map(|t| TextLine::from(*t)).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(app.tab_index);
    
    frame.render_widget(tabs, area);
}

fn render_results_tab<B: Backend>(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(30), // Test list
                Constraint::Percentage(70), // Test details
            ]
            .as_ref(),
        )
        .split(area);
    
    // Test list with fancy styling
    let tests: Vec<ListItem> = app
        .test_results
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let status_symbol = if t.success { "✓" } else { "✗" };
            let status_color = if t.success { Color::Green } else { Color::Red };
            
            let content = TextLine::from(vec![
                Span::styled(
                    format!(" {} ", status_symbol),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[Test {:02}] ", i + 1),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(t.name.clone()),
            ]);
            
            if i == app.selected_test {
                ListItem::new(content).style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(content)
            }
        })
        .collect();
    
    let tests_list = List::new(tests)
        .block(
            Block::default()
                .title(" Tests ")
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    
    frame.render_widget(tests_list, chunks[0]);
    
    // Test details area
    if let Some(test_result) = app.test_results.get(app.selected_test) {
        let details_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
            )
            .split(chunks[1]);
        
        // Expected output with fancy styling
        let expected_title = format!(" Expected Output {} ", 
            if test_result.success { "✓" } else { "≠" });
        
        let expected = Paragraph::new(
            if let Some(diff) = &test_result.diff {
                let expected_lines: Vec<TextLine> = diff
                    .iter()
                    .filter(|line| line.tag != ChangeTag::Insert)
                    .map(|line| {
                        let style = match line.tag {
                            ChangeTag::Delete => Style::default().fg(Color::Red),
                            _ => Style::default(),
                        };
                        TextLine::from(vec![Span::styled(&line.content, style)])
                    })
                    .collect();
                expected_lines
            } else {
                vec![TextLine::from(vec![Span::raw(&test_result.actual_output)])]
            },
        )
        .block(
            Block::default()
                .title(expected_title)
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::default().fg(
                        if test_result.success { Color::Green } else { Color::Red }
                    )
                )
        )
        .wrap(Wrap { trim: false });
        
        // Actual output with fancy styling
        let actual_title = format!(" Actual Output {} ", 
            if test_result.success { "✓" } else { "≠" });
        
        let actual = Paragraph::new(
            if let Some(diff) = &test_result.diff {
                let actual_lines: Vec<TextLine> = diff
                    .iter()
                    .filter(|line| line.tag != ChangeTag::Delete)
                    .map(|line| {
                        let style = match line.tag {
                            ChangeTag::Insert => Style::default().fg(Color::Green),
                            _ => Style::default(),
                        };
                        TextLine::from(vec![Span::styled(&line.content, style)])
                    })
                    .collect();
                actual_lines
            } else {
                vec![TextLine::from(vec![Span::raw(&test_result.actual_output)])]
            },
        )
        .block(
            Block::default()
                .title(actual_title)
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(
                    Style::default().fg(
                        if test_result.success { Color::Green } else { Color::Yellow }
                    )
                )
        )
        .wrap(Wrap { trim: false });
        
        frame.render_widget(expected, details_layout[0]);
        frame.render_widget(actual, details_layout[1]);
    } else {
        // No test selected or no tests available
        let no_tests = Paragraph::new("No test results available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Details ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            );
        
        frame.render_widget(no_tests, chunks[1]);
    }
}

fn render_stats_tab<B: Backend>(frame: &mut Frame, area: Rect, app: &App) {
    let (passed, total, pass_rate) = app.get_stats();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ]
            .as_ref(),
        )
        .split(area);
    
    // Summary stats in a fancy table
    let header_cells = ["Metric", "Value"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells)
        .style(Style::default().fg(Color::Yellow))
        .height(1)
        .bottom_margin(1);
    
    let rows = vec![
        Row::new(vec![
            Cell::from("Total Tests"),
            Cell::from(total.to_string()),
        ]),
        Row::new(vec![
            Cell::from("Passed Tests"),
            Cell::from(passed.to_string()).style(Style::default().fg(Color::Green)),
        ]),
        Row::new(vec![
            Cell::from("Failed Tests"),
            Cell::from((total - passed).to_string()).style(Style::default().fg(Color::Red)),
        ]),
        Row::new(vec![
            Cell::from("Pass Rate"),
            Cell::from(format!("{:.1}%", pass_rate)).style(
                if pass_rate > 90.0 {
                    Style::default().fg(Color::Green)
                } else if pass_rate > 70.0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Red)
                }
            ),
        ]),
    ];
    
    let table = Table::new(rows, &[Constraint::Percentage(50), Constraint::Percentage(50)])
        .header(header)
        .block(
            Block::default()
                .title(" Test Statistics ")
                .title_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        );
    
    frame.render_widget(table, chunks[0]);
    
    // Visual chart of pass/fail ratio
    let pass_percentage = if total > 0 { passed as f64 / total as f64 } else { 0.0 };
    
    // Show a bar chart of pass/fail
    let canvas = Canvas::default()
        .block(
            Block::default()
                .title(" Pass Rate ")
                .title_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        )
        .paint(|ctx| {
            // background
            ctx.draw(&Rectangle {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 5.0,
                color: Color::DarkGray,
            });
            
            // Pass bar (green)
            ctx.draw(&Rectangle {
                x: 0.0,
                y: 0.0,
                width: 100.0 * pass_percentage,
                height: 5.0,
                color: Color::Green,
            });
            
            // Add a line at 100%
            ctx.draw(&Line {
                x1: 100.0,
                y1: 0.0,
                x2: 100.0,
                y2: 5.0,
                color: Color::White,
            });
            
            // Markers at 25%, 50%, 75%
            for x in [25.0, 50.0, 75.0] {
                ctx.draw(&Line {
                    x1: x,
                    y1: 0.0,
                    x2: x,
                    y2: 5.0,
                    color: Color::Gray,
                });
            }
        })
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 10.0]);
    
    frame.render_widget(canvas, chunks[1]);
}

fn render_diff_tab<B: Backend>(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(test_result) = app.test_results.get(app.selected_test) {
        if let Some(diff) = &test_result.diff {
            // Create a unified diff view
            let mut diff_spans = Vec::new();
            
            // Header
            diff_spans.push(TextLine::from(vec![
                Span::styled("Diff for test: ", Style::default().fg(Color::White)),
                Span::styled(&test_result.name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            
            diff_spans.push(TextLine::from(vec![Span::raw("───────────────────────────────────────")]));
            
            // Add each diff line with appropriate styling
            for line in diff {
                let (prefix, style) = match line.tag {
                    ChangeTag::Delete => ("-", Style::default().fg(Color::Red)),
                    ChangeTag::Insert => ("+", Style::default().fg(Color::Green)),
                    ChangeTag::Equal => (" ", Style::default()),
                };
                
                diff_spans.push(TextLine::from(vec![
                    Span::styled(
                        format!("{} {}", prefix, line.content),
                        style,
                    ),
                ]));
            }
            
            let diff_view = Paragraph::new(diff_spans)
                .block(
                    Block::default()
                        .title(" Unified Diff View ")
                        .title_style(Style::default().fg(Color::Magenta))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Blue))
                )
                .wrap(Wrap { trim: false });
            
            frame.render_widget(diff_view, area);
        } else {
            // No diff available (test passed)
            let message = if test_result.success {
                "✓ Test passed - no differences to display"
            } else {
                "No diff information available"
            };
            
            let no_diff = Paragraph::new(message)
                .style(
                    if test_result.success {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    }
                )
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(" Diff View ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                );
            
            frame.render_widget(no_diff, area);
        }
    } else {
        // No test selected
        let no_test = Paragraph::new("No test selected")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Diff View ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            );
        
        frame.render_widget(no_test, area);
    }
}

fn render_status_bar<B: Backend>(frame: &mut Frame, area: Rect) {
    let status_text = vec![
        Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": quit | "),
        Span::styled("↑/k", Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled("↓/j", Style::default().fg(Color::Yellow)),
        Span::raw(": navigate | "),
        Span::styled("←/h", Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled("→/l", Style::default().fg(Color::Yellow)),
        Span::raw(": tabs | "),
        Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": help"),
    ];
    
    let status_bar = Paragraph::new(TextLine::from(status_text))
        .style(Style::default().bg(Color::DarkGray))
        .alignment(Alignment::Center);
    
    frame.render_widget(status_bar, area);
}

fn render_help<B: Backend>(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        TextLine::from(vec![
            Span::styled(
                "Tokage Test Runner - Help",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
        TextLine::from(vec![Span::raw("")]),
        TextLine::from(vec![
            Span::styled("Navigation", Style::default().add_modifier(Modifier::UNDERLINED)),
        ]),
        TextLine::from(vec![
            Span::styled("    j", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Move to next test"),
        ]),
        TextLine::from(vec![
            Span::styled("    k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("↑", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(": Move to previous test"),
        ]),
        // 他のヘルプテキストを追加
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    
    // 中央に表示するためのレイアウト
    let area = centered_rect(60, 60, area);
    frame.render_widget(help_paragraph, area);
}

// ヘルプウィンドウを中央に表示するためのヘルパー関数
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}