use std::env;
use regex::Regex;
use hyper::Client;
use hyper_tls::HttpsConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // This is where we will setup our HTTP client requests.
    let re = Regex::new(r"^URL_").unwrap();
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    for (n, v) in env::vars() {
        if re.is_match(&n) {
            let uri = v.parse()?;
            let resp = client.get(uri).await?;
            println!("{}: {}", resp.status(), v);
        }
    }
    Ok(())
}
