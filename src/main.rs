mod network;

use std::{error::Error, io};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use reqwest::Method;
use select::document::Document;
use select::predicate::{Class, Predicate};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

#[derive(Debug)]
struct Board {
    state: TableState,
    items: Vec<Vec<BoardData>>,
    // items: Vec<Vec<&'a str>>,
}

struct BoardData {
    title: String,
    url: String,
    // board: String,
    comment_count: u32,
    nickname: String,
    hit_count: String,
    timestamp: String
}

impl Board {
    fn new(doc: String) -> Board {
        Board {
            state: TableState::default(),
            items: BoardData::get_board_data(doc)
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl BoardData {
    fn get_board_data(doc: String) -> Vec<Vec<BoardData>> {
        let mut boards = vec![];

        let document = Document::from(doc.as_str());
        let list_items = document.select(Class("symph_row"));

        // let mut board_lists: Vec<BoardData> = vec![];
        for list_item in list_items {
            let title = list_item.select(Class("subject_fixed")).next().unwrap().text();
            let url = list_item.select(Class("list_subject")).next().unwrap().attr("href").unwrap();
            // let board = list_item.select(Class("shortname")).next().unwrap().text();
            let comment_count = list_item.select(Class("rSymph05")).next().unwrap().text();
            let nickname = list_item.select(Class("list_author").descendant(Class("nickname"))).next().unwrap().text();
            let hit_count = list_item.select(Class("list_hit").descendant(Class("hit"))).next().unwrap().text();
            let timestamp = list_item.select(Class("list_time").descendant(Class("timestamp"))).next().unwrap().text();
            // println!("{:?}", nickname.trim());

            let board = BoardData {
                title,
                url: String::from(url),
                comment_count: comment_count.parse::<u32>().unwrap(),
                nickname,
                hit_count,
                timestamp
            };
            boards.push(board);
        }

        let mut boards_vec = vec![];
        boards_vec.push(boards);
        boards_vec
    }
}

fn main() -> crossterm::Result<()> {
    let resp = network::request_url(Method::GET, "https://www.clien.net/service/board/park");
    let doc = resp.text().unwrap().replace(&['\n', '\t'], "");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let board = Board::new(doc);
    let res = run_app(&mut terminal, board);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())

    // loop {
    //     match read().unwrap() {
    //         Event::Key(key_event) => {
    //             let KeyEvent { code, modifiers } = key_event;
    //             match (code, modifiers) {
    //                 (KeyCode::Char(c), _) => {  },
    //                 (KeyCode::Esc, _) => {
    //
    //                 }
    //                 (_, _) => {}
    //             }
    //         }
    //         Event::Mouse(_) => {}
    //         Event::Resize(w, h) => {
    //             println!("window resized to {w} x {h}");
    //         }
    //     }
    // }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: Board) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut Board) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(5)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Header1", "Header2", "Header3"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            // .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}