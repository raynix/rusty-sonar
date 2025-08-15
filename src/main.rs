use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::{env, time};

use poem::{get, handler, EndpointExt, Route, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let metrics_file = env::var("METRICS_FILE").unwrap_or("./metrics".to_string());
    let metrics_file_arc = Arc::new(metrics_file);

    // Spawn metrics update loop in background
    let metrics_file_bg = metrics_file_arc.clone();
    tokio::spawn(async move {
        let re = Regex::new(r"^URL_").unwrap();

        loop {
            let mut metrics = HashMap::new();
            // Move env::vars() out of async context to avoid Send error
            let env_vars: Vec<(String, String)> = env::vars().collect();
            for (n, v) in env_vars {
                if re.is_match(&n) {
                    // Send the request and await the response
                    let client = reqwest::Client::new();
                    let resp = match client.get(&v).send().await {
                        Ok(response) => response,
                        Err(e) => {
                            eprintln!("Failed to fetch {}: {}", v, e);
                            if let Some(src) = e.source() {
                                eprintln!("{:?}", src);
                            }
                            continue;
                        }
                    };
                    println!("{}: {}", resp.status(), v);
                    metrics.insert(String::from(v), String::from(resp.status().as_str()));
                }
            }
            let mut f = match File::create(&*metrics_file_bg) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to create metrics file: {}", e);
                    continue;
                }
            };
            let _ = write!(f, "# HELP http_ping HTTP status code of a URL\n");
            let _ = write!(f, "# TYPE http_ping gauge\n");
            for (k, v) in &metrics {
                let _ = write!(f, "http_ping{{host=\"{}\"}} {}\n", k, v);
            }
            tokio::time::sleep(time::Duration::from_millis(60000)).await;
        }
    });

    // Poem HTTP server
    #[handler]
    async fn metrics_handler(state: poem::web::Data<&Arc<String>>) -> String {
        match std::fs::read_to_string(&***state) {
            Ok(content) => content,
            Err(e) => format!("Failed to read metrics file: {}", e),
        }
    }

    let app = Route::new()
        .at("/metrics", get(metrics_handler))
        .data(metrics_file_arc);

    println!("Serving /metrics on 0.0.0.0:3000");
    Server::new(poem::listener::TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;
    Ok(())
}
