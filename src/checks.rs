use std::{
    collections::HashMap,
    fs::File,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use duration_str::deserialize_duration;
use serde::Deserialize;

pub fn load() -> Config {
    let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| {
        let cwd = std::env::current_dir().unwrap();
        let cwd = cwd.to_string_lossy();
        warn!("CONFIG_PATH unset; using {}/config.yml", cwd);
        String::from("./config.yml")
    });

    let mut file = File::open(path).expect("to open the config file for reading");
    serde_yaml_ng::from_reader(&mut file).expect("to parse the config file")
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub checks: HashMap<String, Check>,
}

#[derive(Clone, Deserialize)]
pub struct Check {
    #[serde(deserialize_with = "deserialize_duration")]
    pub interval: Duration,
    #[serde(flatten)]
    pub kind: CheckKind,
    #[serde(default, skip)]
    pub last_run: Option<Instant>,
    #[serde(default, skip)]
    pub last_status: Option<Status>,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type")]
pub enum CheckKind {
    #[serde(rename = "http")]
    Http {
        url: String,
        #[serde(default = "default_status")]
        status: u16,
    },
    #[serde(rename = "tcp")]
    Tcp { host: String, port: u16 },
    #[serde(rename = "ping")]
    Ping { host: String },
}

fn default_status() -> u16 {
    200
}

#[derive(Debug, Clone)]
pub enum Status {
    Success(Duration),
    Fail(Duration, String),
}

pub async fn run(config: Arc<Mutex<Config>>, name: String) {
    let start = Instant::now();
    let kind = config.lock().unwrap().checks[&name].kind.clone();

    let result = match kind {
        CheckKind::Http { ref url, status } => check_http(url, status).await,
        CheckKind::Tcp { ref host, port } => {
            let _ = host;
            let _ = port;
            todo!("implement TCP check");
        }
        CheckKind::Ping { ref host } => {
            let _ = host;
            todo!("implement ping check");
        }
    };

    let duration = start.elapsed();
    let status = match result {
        Ok(()) => Status::Success(duration),
        Err(error) => Status::Fail(duration, error),
    };

    info!("CHECK '{}' STATUS: {:?}", name, status); // TODO: log something more useful

    {
        let mut config = config.lock().unwrap();
        let check = config.checks.get_mut(&name).unwrap();
        check.last_status = Some(status);
    }
}

async fn check_http(url: &str, target_status: u16) -> Result<(), String> {
    // TODO: use a shared `reqwest` client...
    match reqwest::get(url).await {
        Err(err) => Err(err.to_string()),
        Ok(res) => {
            let status = res.status().as_u16();

            if status == target_status {
                Ok(())
            } else {
                Err(format!("Expected {}, got status {}", target_status, status))
            }
        }
    }
}
