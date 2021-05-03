extern crate open;

#[derive(Debug)]
pub struct Report {
    title: String,
    url: String,
}

impl Report {
    pub fn fetch() -> Vec<Report> {
        return vec![
            Report {
                title: "Yahoo".to_string(),
                url: "https://yahoo.co.jp".to_string(),
            },
            Report {
                title: "Google".to_string(),
                url: "https://google.co.jp".to_string(),
            },
            Report {
                title: "ブートキャンプ".to_string(),
                url: "https://bootcamp.fjord.jp".to_string(),
            },
        ];
    }

    pub fn screen_label(&self) -> String {
        format!("{} {}", &self.title, &self.url)
    }

    pub fn open(&self) {
        open::that(&self.url);
    }
}
