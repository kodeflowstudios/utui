use ratatui::{Frame, layout::{Alignment, Constraint, Layout, Margin}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap}};

pub struct HelpEntry {
    pub key: &'static str,
    pub desc: &'static str,
    pub detail: &'static str,
}

pub struct HelpGroup {
    pub title: &'static str,
    pub entries: &'static [HelpEntry],
}

pub struct HelpState {
    pub selected: usize,      // index into flat list of entries (headers excluded)
    pub scroll_offset: usize, // index into `rows` to fix that stupid invisible line bug
}

pub const HELP_GROUPS: &[HelpGroup] = &[
    HelpGroup {
        title: "Project List",
        entries: &[
            HelpEntry { key: "j/k, Ctrl-n/p", desc: "Navigate",       detail: "Move the selection up or down the project list." },
            HelpEntry { key: "Enter",          desc: "Details",        detail: "Expand or collapse details for the selected project." },
            HelpEntry { key: "o",              desc: "Open",           detail: "Open the selected project in Unity." },
            HelpEntry { key: "c",              desc: "Create",         detail: "Start the new project creation wizard." },
            HelpEntry { key: "d",              desc: "Delete",         detail: "Delete the selected project, keeping its files on disk." },
            HelpEntry { key: "D",              desc: "Delete w/ Dir",  detail: "Delete the selected project and its directory." },
            HelpEntry { key: "e",              desc: "Editors",        detail: "Switch to the editor versions screen." },
            HelpEntry { key: "r",              desc: "Refresh",        detail: "Reload the project list from disk." },
        ],
    },
    HelpGroup {
        title: "Editor List",
        entries: &[
            HelpEntry { key: "j/k, Ctrl-n/p", desc: "Navigate", detail: "Move the selection up or down the editor list." },
            HelpEntry { key: "i",              desc: "Install",   detail: "Install the selected Unity editor version." },
            HelpEntry { key: "d",              desc: "Uninstall", detail: "Uninstall the selected Unity editor version." },
            HelpEntry { key: "Esc",            desc: "Back",      detail: "Return to the project list." },
        ],
    },
    HelpGroup {
        title: "Dialogs & Input",
        entries: &[
            HelpEntry { key: "Enter",   desc: "Confirm",         detail: "Confirm the current dialog or input step." },
            HelpEntry { key: "Esc",     desc: "Cancel",          detail: "Cancel the current dialog or input step." },
            HelpEntry { key: "Tab",     desc: "Complete Path",   detail: "Autocomplete the currently highlighted path entry." },
            HelpEntry { key: "Ctrl-v",  desc: "Paste",           detail: "Paste clipboard contents into the active input field." },
            HelpEntry { key: "h/l",     desc: "Select Ok/Cancel",detail: "Move focus between the Ok and Cancel buttons." },
        ],
    },
    HelpGroup {
        title: "Global",
        entries: &[
            HelpEntry { key: "?", desc: "Help", detail: "Toggle this help menu." },
            HelpEntry { key: "q", desc: "Quit", detail: "Quit the application." },
        ],
    },
];

enum HelpRow {
    Header(&'static str),
    Entry(&'static HelpEntry),
}

fn help_rows() -> Vec<HelpRow> {
    let mut rows = Vec::new();
    for group in HELP_GROUPS {
        rows.push(HelpRow::Header(group.title));
        for entry in group.entries {
            rows.push(HelpRow::Entry(entry));
        }
    }
    rows
}

fn owning_header_indices(rows: &[HelpRow]) -> Vec<usize> {
    let mut owners = Vec::with_capacity(rows.len());
    let mut current_header = 0;
    for (i, row) in rows.iter().enumerate() {
        if matches!(row, HelpRow::Header(_)) {
            current_header = i;
        }
        owners.push(current_header);
    }
    owners
}

impl HelpState {
    pub fn new() -> Self {
        Self { selected: 0, scroll_offset: 0 }
    }

    pub fn move_selection(&mut self, next: bool) {
        let entry_count = help_rows().iter().filter(|r| matches!(r, HelpRow::Entry(_))).count();
        if entry_count == 0 {
            return;
        }
        if next {
            self.selected = (self.selected + 1) % entry_count;
        } else {
            self.selected = (self.selected + entry_count - 1) % entry_count;
        }
    }
}

pub fn render_help_menu(frame: &mut Frame, help_state: &mut HelpState) {
    let area = frame.area();
    let popup_width = ((area.width as f32 * 0.7) as u16).max(60).min(area.width);
    let popup_height = ((area.height as f32 * 0.7) as u16).max(16).min(area.height);
    let popup_area = area.centered(
        Constraint::Length(popup_width),
        Constraint::Length(popup_height),
    );

    frame.render_widget(Clear, popup_area);

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Help ")
        .title_alignment(Alignment::Left);
    let inner = outer_block.inner(popup_area);
    frame.render_widget(outer_block, popup_area);

    let sections = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(inner);
    let columns = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(sections[0]);

    let rows = help_rows();
    let owners = owning_header_indices(&rows);
    let entry_positions: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter_map(|(i, r)| matches!(r, HelpRow::Entry(_)).then_some(i))
        .collect();

    let list_area = columns[0].inner(Margin { vertical: 1, horizontal: 1 });
    let available_height = list_area.height as usize;

    let selected_row = entry_positions
        .get(help_state.selected)
        .copied()
        .unwrap_or(0);

    if selected_row < help_state.scroll_offset {
        help_state.scroll_offset = selected_row;
    } else if selected_row >= help_state.scroll_offset + available_height {
        help_state.scroll_offset = selected_row + 1 - available_height;
    }

    // If the top visible row isn't a header, pin its section's header above it. (Fixes the bug :D)
    let top = help_state.scroll_offset;
    let needs_pin = !matches!(rows[top], HelpRow::Header(_));
    if needs_pin {
        let body_height = available_height.saturating_sub(1);
        if selected_row >= help_state.scroll_offset + body_height {
            help_state.scroll_offset = selected_row + 1 - body_height;
        }
    }

    let top = help_state.scroll_offset;
    let body_height = if needs_pin { available_height.saturating_sub(1) } else { available_height };

    let mut lines: Vec<Line> = Vec::with_capacity(available_height);
    if needs_pin {
        let header_title = match rows[owners[top]] {
            HelpRow::Header(title) => title,
            _ => unreachable!(),
        };
        lines.push(Line::from(Span::styled(
            header_title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )));
    }

    for (offset, row) in rows.iter().skip(top).take(body_height).enumerate() {
        let row_idx = top + offset;
        let line = match row {
            HelpRow::Header(title) => Line::from(Span::styled(
                    *title,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )),
            HelpRow::Entry(entry) => {
                let is_selected = Some(row_idx) == entry_positions.get(help_state.selected).copied();
                let key_style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Blue).bold()
                } else {
                    Style::default().fg(Color::Blue).bold()
                };
                let desc_style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Blue)
                } else {
                    Style::default()
                };
                Line::from(vec![
                    Span::styled(format!(" {:<16}", entry.key), key_style),
                    Span::styled(entry.desc, desc_style),
                ])
            }
        };
        lines.push(line);
    }

    let list_paragraph = Paragraph::new(lines);
    frame.render_widget(list_paragraph, list_area);

    let list_block = Block::default()
        .borders(Borders::RIGHT)
        .border_type(BorderType::Rounded);
    frame.render_widget(list_block, columns[0]);

    let detail_entry = entry_positions
        .get(help_state.selected)
        .and_then(|&idx| rows.get(idx))
        .and_then(|row| match row {
            HelpRow::Entry(entry) => Some(*entry),
            HelpRow::Header(_) => None,
        });

    let detail_paragraph = match detail_entry {
        Some(entry) => Paragraph::new(vec![
            Line::from(Span::styled(entry.key, Style::default().fg(Color::Blue).bold())),
            Line::from(Span::styled(entry.desc, Style::default().bold())),
            Line::from(""),
            Line::from(entry.detail),
        ]),
        None => Paragraph::new(""),
    }
    .wrap(Wrap { trim: true })
    .block(Block::default().padding(Padding::horizontal(2)));

    frame.render_widget(detail_paragraph, columns[1]);

    let footer = Line::from(vec![
        Span::styled("j/k", Style::default().fg(Color::Blue).bold()),
        Span::from(" Navigate   "),
        Span::styled("?/Esc", Style::default().fg(Color::Blue).bold()),
        Span::from(" Close"),
    ])
    .centered();
    frame.render_widget(footer, sections[1]);
}
