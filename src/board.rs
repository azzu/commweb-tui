use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};

pub(crate) struct Board {
    pub(crate) name: String,
    pub uri: String,
}

impl Board {
    pub fn new(name: &str, uri: &str) -> Self {
        Self {
            name: name.to_string(),
            uri: uri.to_string(),
        }
    }
}

pub(crate) struct BoardRow {
    pub title: String,
    pub url: String,
    // board: String,
    pub comment_count: u32,
    pub nickname: String,
    pub hit_count: String,
    pub timestamp: String,
}

impl BoardRow {
    pub fn get_board_data(doc: String) -> Vec<BoardRow> {
        let mut boards = vec![];

        let document = Document::from(doc.as_str());
        let list_items = document.select(Class("symph_row"));

        // let mut board_lists: Vec<BoardData> = vec![];
        for list_item in list_items {
            let title = list_item
                .select(Class("subject_fixed"))
                .next()
                .unwrap()
                .text();
            let url = list_item
                .select(Class("list_subject"))
                .next()
                .unwrap()
                .attr("href")
                .unwrap();
            // let board = list_item.select(Class("shortname")).next().unwrap().text();
            let comment_count = BoardRow::get_comment_count(&list_item);
            let nickname = BoardRow::get_nickname(&list_item);
            let hit_count = list_item
                .select(Class("list_hit").descendant(Class("hit")))
                .next()
                .unwrap()
                .text();
            let timestamp = list_item
                .select(Class("list_time").descendant(Class("timestamp")))
                .next()
                .unwrap()
                .text();
            // println!("{:?}", nickname.trim());

            let board = BoardRow {
                title,
                url: String::from(url),
                comment_count: comment_count.parse::<u32>().unwrap_or(0),
                nickname,
                hit_count,
                timestamp,
            };
            boards.push(board);
        }

        boards
    }

    fn get_comment_count(list_item: &Node) -> String {
        let mut item_comment_count = list_item.select(Class("rSymph05"));
        if item_comment_count.next().is_none() {
            return "0".to_string();
        }
        list_item.select(Class("rSymph05")).next().unwrap().text()
    }

    fn get_nickname(list_item: &Node) -> String {
        let mut item_nickname = list_item
            .select(Class("list_author").descendant(Class("nickname")))
            .next()
            .unwrap()
            .text();
        if item_nickname.trim().is_empty() {
            item_nickname = list_item
                .select(Class("list_author").descendant(Name("img")))
                .next()
                .unwrap()
                .attr("alt")
                .unwrap()
                .to_string();
        }
        item_nickname.trim().to_string()
    }
}