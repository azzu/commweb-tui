use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use reqwest::Method;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{io, thread};
use thiserror::Error;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{
    Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
};
use tui::Terminal;

mod network;
mod board;

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Boards,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> Self {
        match input {
            MenuItem::Home => 0,
            MenuItem::Boards => 1,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(10000);

    thread::spawn(move || {
        let mut last_tic = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tic.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tic.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tic = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_title = vec!["Home", "Boards", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut board_list_state = ListState::default();
    board_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let copyright = Paragraph::new("commweb-tui 2022 - all right reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );

            let menu = menu_title
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Boards => {
                    let boards_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_boards(&board_list_state);
                    rect.render_stateful_widget(left, boards_chunks[0], &mut board_list_state);
                    rect.render_widget(right, boards_chunks[1]);
                }
            }
            rect.render_widget(copyright, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('b') => active_menu_item = MenuItem::Boards,
                KeyCode::Down => {
                    if let Some(selected) = board_list_state.selected() {
                        let amount_boards = read_boards().unwrap().len();
                        if selected >= amount_boards - 1 {
                            board_list_state.select(Some(0));
                        } else {
                            board_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = board_list_state.selected() {
                        let amount_boards = read_boards().unwrap().len();
                        if selected > 0 {
                            board_list_state.select(Some(selected - 1));
                        } else {
                            board_list_state.select(Some(amount_boards - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
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

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "commweb-tui",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'b' to access boards.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

fn render_boards<'a>(board_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let board = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Boards")
        .border_type(BorderType::Plain);

    let boards = read_boards().expect("can fetch board list");
    let items: Vec<_> = boards
        .iter()
        .map(|board| {
            ListItem::new(Spans::from(vec![Span::styled(
                board.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_board = boards
        .get(
            board_list_state
                .selected()
                .expect("there is always a selected board"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(board).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let board_rows = read_board_rows(selected_board.uri.as_str());
    let mut cells = vec![];
    for board_row in board_rows.unwrap() {
        let row = Row::new(vec![
            Cell::from(Span::raw(board_row.title.to_string())),
            Cell::from(Span::raw(board_row.comment_count.to_string())),
            Cell::from(Span::raw(board_row.nickname.to_string())),
            Cell::from(Span::raw(board_row.hit_count.to_string())),
            Cell::from(Span::raw(board_row.timestamp.to_string())),
        ]);
        cells.push(row);
    }
    let board_row = Table::new(cells)
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "제목",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "댓글",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "작성자",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "읽음",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "작성시간",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("목록")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Percentage(8),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(22),
        ]);

    (list, board_row)
}

fn read_boards() -> Result<Vec<board::Board>, Error> {
    let board_list = vec![
        board::Board::new("모두의공원", "board/park"),
        board::Board::new("새로운소식", "board/news"),
        board::Board::new("유용한사이트", "board/useful"),
        board::Board::new("자료실", "board/pds"),
        board::Board::new("팁과강좌", "board/lecture"),
        board::Board::new("사용기", "board/use"),
        board::Board::new("추천글", "recommend"),
    ];

    Ok(board_list)
}

fn read_board_rows(board_code: &str) -> Result<Vec<board::BoardRow>, Error> {
    let mut url = "https://www.clien.net/service/".to_owned();
    url.push_str(board_code);
    let resp = network::request_url(Method::GET, url);
    let doc = resp.text().unwrap().replace(&['\n', '\t'], "");

    let board_row = board::BoardRow::get_board_data(doc);
    Ok(board_row)
}
