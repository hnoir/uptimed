use std::{
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};

use notify_rust::Notification;
use reqwest::{Client, StatusCode};

use crate::config::{AdditionalRequestHeader, Configuration};

#[derive(Debug)]
struct DownError {
    url: String,
    code: StatusCode,
}

impl std::fmt::Display for DownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error downloading {}: status code {}",
            self.url, self.code
        )
    }
}

impl std::error::Error for DownError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

async fn process_url(
    client: &Client,
    url: String,
    headers: &Vec<AdditionalRequestHeader>,
) -> Result<(), DownError> {
    let request_builder = client.get(&url);

    let request_builder = headers.iter().fold(request_builder, |builder, header| {
        builder.header(&header.name, &header.value)
    });

    let response = request_builder.send().await;

    match response {
        Ok(res) => {
            if !res.status().is_success() {
                return Err(DownError {
                    url,
                    code: res.status(),
                });
            }
        }
        Err(e) => {
            return Err(DownError {
                url,
                code: e.status().unwrap_or(StatusCode::NOT_FOUND),
            });
        }
    }

    Ok(())
}

pub async fn process_urls(config: &Configuration) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build().unwrap();

    let file = File::open(config.targets_path.clone())?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let url = line?;

        match process_url(&client, url, &config.custom_headers).await {
            Ok(_) => (),
            Err(e) => {
                Notification::new()
                    .summary(format!("{} is down!", e.url).as_str())
                    .body(format!("Responded with status code: {}", e.code).as_str())
                    .show()?;
            }
        }

        if config.request_interval > Duration::from_secs(0) {
            tokio::time::sleep(config.request_interval).await;
        }
    }

    Ok(())
}
