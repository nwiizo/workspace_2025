use crate::app::{App, PopupType};
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
use chrono::{DateTime, Utc, TimeZone};

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
            3 => render_command_tab::<B>(frame, main_chunks[2], app),
            4 => render_history_tab::<B>(frame, main_chunks[2], app),
            _ => {}
        }
        
        // Status bar
        render_status_bar::<B>(frame, main_chunks[3]);
        
        // ポップアップがあれば表示
        if app.show_popup {
            render_popup::<B>(frame, size, app);
        }
        
        // 結果ポップアップがあれば表示
        if app.result_popup_visible {
            render_result_popup::<B>(frame, size, app);
        }
    }
}

fn render_title<B: Backend>(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(vec![
        TextLine::from(vec![
            Span::styled("YA", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("MO", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::styled("RI", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" - ", Style::default().fg(Color::White)),
            Span::styled("YAML Test Observer & Runner Interface", Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)),
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
    let titles = vec!["Test Results", "Statistics", "Diff View", "Commands", "History"];
    
    // タブのスタイルを改善
    let tab_titles: Vec<TextLine> = titles.iter().enumerate().map(|(i, t)| {
        if i == app.tab_index {
            // 選択中のタブは背景色と太字で強調
            TextLine::from(vec![
                Span::styled(
                    format!(" {} ", t),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                )
            ])
        } else {
            // 非選択タブは通常表示
            TextLine::from(vec![
                Span::styled(
                    format!(" {} ", t),
                    Style::default().fg(Color::White)
                )
            ])
        }
    }).collect();
    
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .title(" Tabs ")
                .title_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue))
        )
        .divider("|")  // タブ間の区切り文字
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
            let status_bg = if t.success { Color::Green } else { Color::Red };
            
            // テスト結果をより視覚的に分かりやすく
            let content = if i == app.selected_test {
                // 選択中のテストは背景色を変えて強調
                TextLine::from(vec![
                    Span::styled(
                        format!(" {} ", status_symbol),
                        Style::default()
                            .fg(Color::White)
                            .bg(status_bg)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" Test {:02} ", i + 1),
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {} ", t.name),
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                // 非選択のテスト
                TextLine::from(vec![
                    Span::styled(
                        format!(" {} ", status_symbol),
                        Style::default()
                            .fg(Color::White)
                            .bg(status_bg),
                    ),
                    Span::styled(
                        format!(" Test {:02} ", i + 1),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::raw(format!(" {} ", t.name)),
                ])
            };
            
            ListItem::new(content)
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

fn render_command_tab<B: Backend>(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(30), // Test list
                Constraint::Percentage(70), // Command details
            ]
            .as_ref(),
        )
        .split(area);
    
    // Test list (same as in results tab)
    let tests: Vec<ListItem> = app
        .test_results
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let status_symbol = if t.success { "✓" } else { "✗" };
            
            let content = TextLine::from(vec![
                Span::styled(
                    format!(" {} ", status_symbol),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
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
    
    // Command details
    if let Some(command_details) = app.get_command_details() {
        let (command, args, input, execution_time, is_release, build_commands) = command_details;
        
        // Format the command and arguments
        let full_command = format!("{} {}", command, args.join(" "));
        
        // Create a table for command details
        let mut rows = vec![
            Row::new(vec![
                Cell::from("Command:"),
                Cell::from(full_command).style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Execution Time:"),
                Cell::from(format!("{} ms", execution_time)).style(Style::default().fg(Color::Yellow)),
            ]),
            Row::new(vec![
                Cell::from("Build Mode:"),
                Cell::from(if is_release { "Release" } else { "Debug" })
                    .style(Style::default().fg(if is_release { Color::Magenta } else { Color::Blue })),
            ]),
        ];
        
        // Add build commands if available
        if let Some(commands) = build_commands {
            if !commands.is_empty() {
                rows.push(Row::new(vec![
                    Cell::from("Build Commands:"),
                    Cell::from(commands.join("; ")).style(Style::default().fg(Color::Green)),
                ]));
            }
        }
        
        let command_table = Table::new(rows, &[Constraint::Percentage(30), Constraint::Percentage(70)])
            .block(
                Block::default()
                    .title(" Command Details ")
                    .title_style(Style::default().fg(Color::Yellow))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Blue))
            )
            .column_spacing(1);
        
        let details_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(6), // Command table (increased height for build info)
                    Constraint::Min(0),    // Input (if any)
                ]
                .as_ref(),
            )
            .split(chunks[1]);
        
        frame.render_widget(command_table, details_chunks[0]);
        
        // Show input if available
        if let Some(input_text) = input {
            let input_paragraph = Paragraph::new(input_text.as_str())
                .block(
                    Block::default()
                        .title(" Input ")
                        .title_style(Style::default().fg(Color::Yellow))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Blue))
                )
                .wrap(Wrap { trim: false });
            
            frame.render_widget(input_paragraph, details_chunks[1]);
        }
    } else {
        // No test selected or no tests available
        let no_tests = Paragraph::new("No command details available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Command Details ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            );
        
        frame.render_widget(no_tests, chunks[1]);
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
        Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": run tests | "),
        Span::styled("R", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": run release | "),
        Span::styled("b", Style::default().fg(Color::Yellow)),
        Span::raw(": toggle build mode | "),
        Span::styled("H", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": history | "),
        Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": help"),
    ];
    
    let status_bar = Paragraph::new(TextLine::from(status_text))
        .style(Style::default().bg(Color::DarkGray))
        .alignment(Alignment::Center);
    
    frame.render_widget(status_bar, area);
}

fn render_help<B: Backend>(frame: &mut Frame, area: Rect) {
    let help_area = centered_rect(60, 60, area);
    
    let help_text = vec![
        TextLine::from(vec![
            Span::styled("YAMORI Help", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        TextLine::from(""),
        TextLine::from(vec![
            Span::styled("Navigation", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
        ]),
        TextLine::from(vec![
            Span::styled("j/↓", Style::default().fg(Color::Yellow)),
            Span::raw(": Move down"),
        ]),
        TextLine::from(vec![
            Span::styled("k/↑", Style::default().fg(Color::Yellow)),
            Span::raw(": Move up"),
        ]),
        TextLine::from(vec![
            Span::styled("h/←", Style::default().fg(Color::Yellow)),
            Span::raw(": Previous tab"),
        ]),
        TextLine::from(vec![
            Span::styled("l/→", Style::default().fg(Color::Yellow)),
            Span::raw(": Next tab"),
        ]),
        TextLine::from(""),
        TextLine::from(vec![
            Span::styled("Actions", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
        ]),
        TextLine::from(vec![
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::raw(": Re-run tests"),
        ]),
        TextLine::from(vec![
            Span::styled("b", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle release mode"),
        ]),
        TextLine::from(vec![
            Span::styled("R", Style::default().fg(Color::Yellow)),
            Span::raw(": Run tests in release mode"),
        ]),
        TextLine::from(""),
        TextLine::from(vec![
            Span::styled("History", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
        ]),
        TextLine::from(vec![
            Span::styled("H", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle history view"),
        ]),
        TextLine::from(vec![
            Span::styled("n", Style::default().fg(Color::Yellow)),
            Span::raw(": Next history entry (in history view)"),
        ]),
        TextLine::from(vec![
            Span::styled("p", Style::default().fg(Color::Yellow)),
            Span::raw(": Previous history entry (in history view)"),
        ]),
        TextLine::from(""),
        TextLine::from(vec![
            Span::styled("General", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
        ]),
        TextLine::from(vec![
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
        TextLine::from(vec![
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle help"),
        ]),
        TextLine::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Close help/history view"),
        ]),
    ];
    
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
        );
    
    frame.render_widget(help, help_area);
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

// 履歴表示モードのレンダリング
fn render_history_tab<B: Backend>(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),  // 説明
                Constraint::Min(0),     // 履歴リスト
            ]
            .as_ref(),
        )
        .split(area);
    
    // 説明
    let help_text = Paragraph::new(vec![
        TextLine::from(vec![
            Span::styled("Use ", Style::default().fg(Color::Gray)),
            Span::styled("↑/k", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" and ", Style::default().fg(Color::Gray)),
            Span::styled("↓/j", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" to navigate history. Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" to load selected history.", Style::default().fg(Color::Gray)),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .title(" History Navigation ")
            .title_style(Style::default().fg(Color::Yellow))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
    );
    
    frame.render_widget(help_text, chunks[0]);
    
    // 履歴リスト
    let history_stats = app.get_history_stats();
    
    let rows: Vec<Row> = history_stats.iter().enumerate().map(|(i, (timestamp, passed, total, is_release))| {
        // Unix タイムスタンプを DateTime に変換
        let dt: DateTime<Utc> = Utc.timestamp_opt(*timestamp as i64, 0).unwrap();
        let formatted_time = dt.format("%Y-%m-%d %H:%M:%S").to_string();
        
        // 合格率を計算
        let pass_rate = if *total > 0 {
            (*passed as f64 / *total as f64) * 100.0
        } else {
            0.0
        };
        
        // 合格率に応じた色を設定
        let pass_rate_style = if pass_rate > 90.0 {
            Style::default().fg(Color::Green)
        } else if pass_rate > 70.0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Red)
        };
        
        // 行のスタイルを設定
        let row_style = if i == app.selected_history {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        // 行を作成
        Row::new(vec![
            Cell::from(format!("{}", i + 1)).style(row_style),
            Cell::from(formatted_time).style(row_style),
            Cell::from(format!("{}/{}", passed, total)).style(
                if i == app.selected_history {
                    row_style
                } else if *passed == *total {
                    Style::default().fg(Color::Green)
                } else if *passed > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Red)
                }
            ),
            Cell::from(format!("{:.1}%", pass_rate)).style(
                if i == app.selected_history {
                    row_style
                } else {
                    pass_rate_style
                }
            ),
            Cell::from(if *is_release { "Release" } else { "Debug" }).style(
                if i == app.selected_history {
                    row_style
                } else if *is_release {
                    Style::default().fg(Color::Magenta)
                } else {
                    Style::default().fg(Color::Blue)
                }
            ),
        ])
    }).collect();
    
    let header_cells = ["#", "Timestamp", "Passed/Total", "Pass Rate", "Build Mode"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD));
    
    let history_table = Table::new(rows, &[
            Constraint::Length(3),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .header(header)
        .block(
            Block::default()
                .title(" Test History ")
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
        )
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    
    frame.render_widget(history_table, chunks[1]);
}

// ポップアップを表示する関数
fn render_popup<B: Backend>(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(50, 30, area);
    
    // 背景を描画（完全な黒）
    let background = Block::default()
        .style(Style::default().bg(Color::Black));
    frame.render_widget(background, area);
    
    let popup_title = match app.popup_type {
        PopupType::RunTests => " Run Tests ",
        PopupType::RunRelease => " Run Tests in Release Mode ",
        PopupType::BuildToggle => " Toggle Build Mode ",
        PopupType::None => "",
        PopupType::ResultNotification => " Test Results ",
    };
    
    let popup_message = match app.popup_type {
        PopupType::RunTests => vec![
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled("Are you sure you want to run the tests?", 
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            ]),
            TextLine::from(""),
            TextLine::from("This will execute all tests defined in your configuration."),
            TextLine::from("Current results will be saved to history."),
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" to confirm or ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" to cancel", Style::default().fg(Color::Gray)),
            ]),
        ],
        PopupType::RunRelease => vec![
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled("Run tests in RELEASE mode?", 
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            ]),
            TextLine::from(""),
            TextLine::from("This will compile in release mode and then run all tests."),
            TextLine::from("This may take longer but will test optimized code."),
            TextLine::from("Current results will be saved to history."),
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" to confirm or ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" to cancel", Style::default().fg(Color::Gray)),
            ]),
        ],
        PopupType::BuildToggle => vec![
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled(
                    if app.config.build.as_ref().map_or(false, |b| b.release) {
                        "Switch to DEBUG mode?"
                    } else {
                        "Switch to RELEASE mode?"
                    }, 
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            ]),
            TextLine::from(""),
            TextLine::from(
                if app.config.build.as_ref().map_or(false, |b| b.release) {
                    "This will switch to debug mode for the next test run."
                } else {
                    "This will switch to release mode for the next test run."
                }
            ),
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled("Current mode: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if app.config.build.as_ref().map_or(false, |b| b.release) {
                        "RELEASE"
                    } else {
                        "DEBUG"
                    },
                    if app.config.build.as_ref().map_or(false, |b| b.release) {
                        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
                    }
                ),
            ]),
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" to confirm or ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" to cancel", Style::default().fg(Color::Gray)),
            ]),
        ],
        PopupType::ResultNotification => vec![
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled("Test Results", 
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            ]),
            TextLine::from(""),
            TextLine::from("The test execution has completed."),
            TextLine::from(""),
            TextLine::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to close this message", Style::default().fg(Color::Gray)),
            ]),
        ],
        PopupType::None => vec![],
    };
    
    let popup_block = Block::default()
        .title(popup_title)
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)  // 二重線の枠に変更
        .border_style(Style::default().fg(Color::Yellow))  // 枠線の色を黄色に変更
        .style(Style::default().bg(Color::Blue));  // 背景色を青に変更
    
    let popup = Paragraph::new(popup_message)
        .block(popup_block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));  // テキストを白に設定
    
    frame.render_widget(popup, popup_area);
}

// 結果ポップアップを表示する関数
fn render_result_popup<B: Backend>(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(40, 20, area);
    
    // 背景を描画（完全な黒）
    let background = Block::default()
        .style(Style::default().bg(Color::Black));
    frame.render_widget(background, area);
    
    let popup_block = Block::default()
        .title(" Test Results ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)  // 二重線の枠に変更
        .border_style(Style::default().fg(Color::Yellow))  // 枠線の色を黄色に変更
        .style(Style::default().bg(Color::Blue));  // 背景色を青に変更
    
    let popup = Paragraph::new(app.result_popup_message.clone())
        .block(popup_block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));  // テキストを白に設定
    
    frame.render_widget(popup, popup_area);
} 