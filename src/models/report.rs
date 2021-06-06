use anyhow::Result;
use std::env;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Report {
    title: String,
    url: String,
    reported_on: String,
    login_name: String,
}

impl Report {
    pub fn fetch() -> Result<Vec<Report>> {
        let mut page = 1;
        let mut reports = vec![];
        loop {
            let url = format!(
                "https://bootcamp.fjord.jp/api/reports/unchecked.json?page={}",
                page
            );
            let resp = ureq::get(&url)
                .set("Authorization", &env::var("FJORD_JWT_TOKEN")?)
                .call()?;
            let json: serde_json::Value = resp.into_json()?;
            let report_array = json["reports"].as_array().unwrap();
            if report_array.is_empty() {
                break;
            }
            for r in json["reports"].as_array().unwrap().iter() {
                reports.push(Report {
                    title: r["title"].as_str().unwrap().to_string(),
                    url: r["url"].as_str().unwrap().to_string(),
                    reported_on: r["reportedOn"].as_str().unwrap().to_string(),
                    login_name: r["user"]["login_name"].as_str().unwrap().to_string(),
                })
            }
            page += 1;
            thread::sleep(Duration::from_millis(500));
        }
        Ok(reports)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn reported_on(&self) -> &str {
        &self.reported_on
    }

    pub fn login_name(&self) -> &str {
        &self.login_name
    }

    pub fn open(&self) {
        open::that(&self.url).unwrap();
    }
}
