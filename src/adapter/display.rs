use crate::adapter::{Link, Topic};
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{
    Block, Borders, Cell, HighlightSpacing, Padding, Paragraph, Row, Scrollbar,
    ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use ratatui::Frame;
use ratatui::Terminal;
use std::error::Error;
use std::io::Stdout;
use unicode_width::UnicodeWidthStr;

fn calc_topic_len_constraint(items: &[Topic]) -> (u16, u16, u16) {
    let title_len = items
        .iter()
        .map(|topic| topic.title.width())
        .max()
        .unwrap_or(0);
    let command_len = items
        .iter()
        .map(|topic| topic.command.width())
        .max()
        .unwrap_or(0);

    let category_len = items
        .iter()
        .map(|topic| topic.category.width())
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (category_len as u16, title_len as u16, command_len as u16)
}

fn calc_link_len_constraint(items: &[Link]) -> (u16, u16) {
    let title_len = items
        .iter()
        .map(|link| link.title.width())
        .max()
        .unwrap_or(0);
    let url_len = items.iter().map(|link| link.url.width()).max().unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (title_len as u16, url_len as u16)
}

fn create_title() -> Paragraph<'static> {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    Paragraph::new(Text::styled(
        "TERMNOTE 📝",
        Style::default().fg(Color::Cyan),
    ))
    .block(title_block)
}

fn create_url_table(library_list: &mut LibraryUrlList) -> (Table<'_>, &mut LibraryUrlList) {
    let header_style = Style::default().bg(Color::Blue).fg(Color::White);
    let selected_row_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(Color::LightCyan);

    let selected_col_style = Style::default().fg(Color::Green);
    let header = ["Index", "Title", "URL"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
    let rows = library_list.links.iter().enumerate().map(|(index, link)| {
        [
            Cell::from(Text::from(index.to_string())),
            Cell::from(Text::from(link.title.clone())),
            Cell::from(Text::from(link.url.clone())),
        ]
        .into_iter()
        .map(|cell| cell.style(Style::default().fg(Color::White)))
        .collect::<Row>()
    });
    (
        Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Length(library_list.longest_item_lens.0),
                Constraint::Min(library_list.longest_item_lens.1),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .highlight_symbol(Text::from(vec![" █ ".into()]))
        .bg(Color::Black)
        .highlight_spacing(HighlightSpacing::Always),
        library_list,
    )
}

fn create_topic_table(library_list: &mut LibraryList) -> (Table<'_>, &mut LibraryList) {
    let header_style = Style::default().bg(Color::Blue).fg(Color::White);
    let selected_row_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(Color::LightCyan);

    let selected_col_style = Style::default().fg(Color::Green);
    let header = ["Index", "Category", "Title", "Command"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
    let rows = library_list
        .topics
        .iter()
        .enumerate()
        .map(|(index, topic)| {
            [
                Cell::from(Text::from(index.to_string())),
                Cell::from(Text::from(topic.category.clone())),
                Cell::from(Text::from(topic.title.clone())),
                Cell::from(Text::from(topic.command.clone())),
            ]
            .into_iter()
            .map(|cell| cell.style(Style::default().fg(Color::White)))
            .collect::<Row>()
        });
    (
        Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Length(library_list.longest_item_lens.0),
                Constraint::Length(library_list.longest_item_lens.1),
                Constraint::Min(library_list.longest_item_lens.2),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .highlight_symbol(Text::from(vec![" █ ".into()]))
        .bg(Color::Black)
        .highlight_spacing(HighlightSpacing::Always),
        library_list,
    )
}

fn create_topic_footer_info() -> Paragraph<'static> {
    const INFO_TEXT: [&str; 1] =
        ["(q) quit | (↑) move up | (↓) move down | (e) execute command | (ENTER) return command"];
    Paragraph::new(Text::from_iter(INFO_TEXT))
        .style(Style::new().fg(Color::Blue).bg(Color::Black))
        .centered()
        .block(
            Block::bordered()
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(Style::new().fg(Color::Blue)),
        )
}

fn create_url_footer_info() -> Paragraph<'static> {
    const INFO_TEXT: [&str; 1] = ["(q) quit | (↑) move up | (↓) move down | (ENTER) return URL"];
    Paragraph::new(Text::from_iter(INFO_TEXT))
        .style(Style::new().fg(Color::Blue).bg(Color::Black))
        .centered()
        .block(
            Block::bordered()
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(Style::new().fg(Color::Blue)),
        )
}

fn create_scrollbar() -> Scrollbar<'static> {
    Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
}

fn render_all_topic_ui_blocks(library_list: &mut LibraryList, frame: &mut Frame) {
    let [header_area, main_area] =
        Layout::vertical([Constraint::Length(4), Constraint::Fill(3)]).areas(frame.area());

    let [list_area, info_area, item_area] = Layout::vertical([
        Constraint::Percentage(80),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .areas(main_area);

    let (table, lib_list) = create_topic_table(library_list);
    frame.render_widget(create_title(), header_area);
    frame.render_stateful_widget(table, list_area, &mut lib_list.state);
    let scroll_area = list_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    frame.render_stateful_widget(create_scrollbar(), scroll_area, &mut lib_list.scroll_state);
    frame.render_widget(create_topic_footer_info(), info_area);
    frame.render_widget(create_footer_selected_topic(lib_list), item_area);
}

fn render_all_url_ui_blocks(library_list: &mut LibraryUrlList, frame: &mut Frame) {
    let [header_area, main_area] =
        Layout::vertical([Constraint::Length(4), Constraint::Fill(3)]).areas(frame.area());

    let [list_area, info_area] =
        Layout::vertical([Constraint::Percentage(90), Constraint::Length(3)]).areas(main_area);

    let (table, lib_list) = create_url_table(library_list);
    frame.render_widget(create_title(), header_area);
    frame.render_stateful_widget(table, list_area, &mut lib_list.state);
    let scroll_area = list_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    frame.render_stateful_widget(create_scrollbar(), scroll_area, &mut lib_list.scroll_state);
    frame.render_widget(create_url_footer_info(), info_area);
}

fn create_footer_selected_topic(library_list: &LibraryList) -> Paragraph<'static> {
    let info = if let Some(index) = library_list.state.selected() {
        format!("$ {}", library_list.topics[index].command)
    } else {
        "--".to_string()
    };
    let block = Block::new().padding(Padding::horizontal(1));
    Paragraph::new(info).fg(Color::LightGreen).block(block)
}

pub fn display_text(text: &str) {
    println!("{}", text.to_string().bright_blue());
}

struct LibraryUrlList {
    links: Vec<Link>,
    state: TableState,
    scroll_state: ScrollbarState,
    longest_item_lens: (u16, u16),
}

struct LibraryList {
    topics: Vec<Topic>,
    state: TableState,
    scroll_state: ScrollbarState,
    longest_item_lens: (u16, u16, u16),
}

pub enum MenuEvent {
    None,
    Display,
    Execute,
}

pub struct TerminalUrlUI {
    exit: bool,
    library_list: LibraryUrlList,
    item_height: usize,
    pub selected_url: String,
}

impl TerminalUrlUI {
    pub fn new(links: Vec<Link>) -> Self {
        let item_height: usize = 1;
        let scroll_state: ScrollbarState = if links.len() > 1 {
            ScrollbarState::new((links.len() - 1) * item_height)
        } else {
            ScrollbarState::default()
        };
        Self {
            exit: false,
            library_list: LibraryUrlList {
                links: links.clone(),
                state: TableState::default(),
                scroll_state,
                longest_item_lens: calc_link_len_constraint(&links),
            },
            item_height,
            selected_url: "".to_string(),
        }
    }

    pub fn menu_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        while !self.exit {
            terminal.draw(|frame| render_all_url_ui_blocks(&mut self.library_list, frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        match event::read()? {
            Event::Key(event) if event.kind == KeyEventKind::Press => match event.code {
                KeyCode::Char('q') => {
                    self.exit = true;
                }
                KeyCode::Up => self.select_previous(),
                KeyCode::Down => self.select_next(),
                KeyCode::Enter => {
                    self.handle_selected();
                    self.exit = true;
                }
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }

    fn select_next(&mut self) {
        let index = match self.library_list.state.selected() {
            Some(idx) => {
                if idx >= self.library_list.links.len() - 1 {
                    0
                } else {
                    idx + 1
                }
            }
            None => 0,
        };
        self.library_list.state.select(Some(index));
        self.library_list.scroll_state = self
            .library_list
            .scroll_state
            .position(index * self.item_height);
    }

    fn select_previous(&mut self) {
        let index = match self.library_list.state.selected() {
            Some(idx) => {
                if idx == 0 {
                    self.library_list.links.len() - 1
                } else {
                    idx - 1
                }
            }
            None => 0,
        };
        self.library_list.state.select(Some(index));
        self.library_list.scroll_state = self
            .library_list
            .scroll_state
            .position(index * self.item_height);
    }

    fn handle_selected(&mut self) {
        if let Some(index) = &self.library_list.state.selected() {
            self.selected_url = self.library_list.links[*index].url.to_string();
        };
    }
}

pub struct TerminalUI {
    exit: bool,
    library_list: LibraryList,
    item_height: usize,
    pub selected_cmd: String,
    pub event: MenuEvent,
}

impl TerminalUI {
    pub fn new(topics: Vec<Topic>) -> Self {
        let item_height: usize = 1;
        let scroll_state: ScrollbarState = if topics.len() > 1 {
            ScrollbarState::new((topics.len() - 1) * item_height)
        } else {
            ScrollbarState::default()
        };
        Self {
            exit: false,
            library_list: LibraryList {
                topics: topics.clone(),
                state: TableState::default(),
                scroll_state,
                longest_item_lens: calc_topic_len_constraint(&topics),
            },
            item_height,
            selected_cmd: "".to_string(),
            event: MenuEvent::None,
        }
    }

    pub fn menu_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        while !self.exit {
            terminal.draw(|frame| render_all_topic_ui_blocks(&mut self.library_list, frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        match event::read()? {
            Event::Key(event) if event.kind == KeyEventKind::Press => match event.code {
                KeyCode::Char('q') => {
                    self.event = MenuEvent::None;
                    self.exit = true;
                }
                KeyCode::Char('e') => {
                    self.event = MenuEvent::Execute;
                    self.handle_selected();
                    self.exit = true;
                }
                KeyCode::Up => self.select_previous(),
                KeyCode::Down => self.select_next(),
                KeyCode::Enter => {
                    self.event = MenuEvent::Display;
                    self.handle_selected();
                    self.exit = true;
                }
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }

    fn select_next(&mut self) {
        let index = match self.library_list.state.selected() {
            Some(idx) => {
                if idx >= self.library_list.topics.len() - 1 {
                    0
                } else {
                    idx + 1
                }
            }
            None => 0,
        };
        self.library_list.state.select(Some(index));
        self.library_list.scroll_state = self
            .library_list
            .scroll_state
            .position(index * self.item_height);
    }

    fn select_previous(&mut self) {
        let index = match self.library_list.state.selected() {
            Some(idx) => {
                if idx == 0 {
                    self.library_list.topics.len() - 1
                } else {
                    idx - 1
                }
            }
            None => 0,
        };
        self.library_list.state.select(Some(index));
        self.library_list.scroll_state = self
            .library_list
            .scroll_state
            .position(index * self.item_height);
    }

    fn handle_selected(&mut self) {
        if let Some(index) = &self.library_list.state.selected() {
            self.selected_cmd = self.library_list.topics[*index].command.to_string();
        };
    }
}
