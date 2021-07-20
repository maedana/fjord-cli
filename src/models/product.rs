use std::env;
use std::thread;
use std::time::Duration;

// MEMO: Produtは個別ファイルにしたい
#[derive(Debug, Clone)]
pub struct Product {
    title: String,
    url: String,
    updated_on: String,
    login_name: String,
    assigned: bool,
}
impl Product {
    pub fn fetch() -> Vec<Product> {
        let mut page = 1;
        let mut products = vec![];
        loop {
            let url = format!(
                "https://bootcamp.fjord.jp/api/products/not_responded.json?page={}",
                page
            );
            let resp = ureq::get(&url)
                .set("Authorization", &env::var("FJORD_JWT_TOKEN").unwrap())
                .call()
                .unwrap();
            let json: serde_json::Value = resp.into_json().unwrap();
            let product_array = json["products"].as_array().unwrap();
            if product_array.is_empty() {
                break;
            }
            for p in product_array.iter() {
                products.push(Product {
                    title: p["practice"]["title"].as_str().unwrap().to_string(),
                    url: p["url"].as_str().unwrap().to_string(),
                    updated_on: p["updated_at"].as_str().unwrap().to_string(),
                    login_name: p["user"]["login_name"].as_str().unwrap().to_string(),
                    assigned: !p["checker_name"].is_null(),
                })
            }
            page += 1;
            thread::sleep(Duration::from_millis(200));
        }
        products
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn updated_on(&self) -> &str {
        &self.updated_on
    }

    pub fn login_name(&self) -> &str {
        &self.login_name
    }

    pub fn assigned(&self) -> bool {
        self.assigned
    }

    pub fn open(&self) {
        open::that(&self.url).unwrap();
    }
}
