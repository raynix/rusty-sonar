use regex::Regex;
use hyper::Client;
use hyper_tls::HttpsConnector;

use std::{env, thread, time};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut metrics = HashMap::new();
    let metrics_file = env::var("METRICS_FILE").unwrap_or("./metrics".to_string());
    let re = Regex::new(r"^URL_").unwrap();
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    loop {
        for (n, v) in env::vars() {
            if re.is_match(&n) {
                let uri = v.parse()?;
                let resp = client.get(uri).await?;
                println!("{}: {}", resp.status(), v);
                metrics.insert(String::from(v), String::from(resp.status().as_str()));
            }
        }
        let mut f = File::create(&metrics_file)?;
        for (k, v) in &metrics {
            write!(f, "http_ping{{host=\"{}\"}} {}\n", k, v)?;
        }
        thread::sleep(time::Duration::from_millis(60000));
    }
}
