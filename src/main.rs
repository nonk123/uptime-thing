#[macro_use]
extern crate log;

use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

mod checks;

use checks::Config;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    pretty_env_logger::init();

    let config = Arc::new(Mutex::new(checks::load()));

    loop {
        think(config.clone());
        std::thread::yield_now();
    }
}

fn think(config: Arc<Mutex<Config>>) {
    let checks = {
        let config = config.lock().unwrap();
        config.checks.clone()
    };

    for (name, check) in checks {
        let should_run = match check.last_run {
            Some(last_run) => last_run.elapsed() >= check.interval,
            None => true,
        };

        if !should_run {
            continue;
        }

        {
            let mut config = config.lock().unwrap();
            config.checks.get_mut(&name).unwrap().last_run = Some(Instant::now());
        }

        tokio::spawn(checks::run(config.clone(), name));
    }
}
