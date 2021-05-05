use tui::widgets::TableState;

pub struct StatefulTable {
    // MEMO: 可変参照を要求する箇所があるのでpublicにしたけど本来よくないのでは
    pub state: TableState,
    items: Vec<Vec<String>>,
}

impl StatefulTable {
    pub fn new(items: Vec<Vec<String>>) -> StatefulTable {
        StatefulTable {
            state: TableState::default(),
            items,
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

    pub fn items(&self) -> &Vec<Vec<String>> {
        &self.items
    }
}
