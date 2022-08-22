use dashmap::DashMap;
use eyre::{eyre, Context, Result};
use reqwest::Client;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use ical_merger::{
    calendars::Calendar,
    config::{read_config_file, ApplicationConfig},
};
use rocket::{get, http::ContentType, response::Responder, routes, State};

#[rocket::launch]
async fn rocket() -> _ {
    // load Arc<RwLock<ApplicationConfig>> from config.json
    let config = read_config_file().unwrap();
    let config = Arc::new(RwLock::new(config));
    let cache: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
    tokio::spawn(worker_thread(config.clone(), cache.clone(), Client::new()));
    rocket::build()
        .mount("/", routes![calendar])
        .manage(cache)
        .manage(config)
}

async fn worker_thread(
    config: Arc<RwLock<ApplicationConfig>>,
    cache: Arc<DashMap<String, String>>,
    client: Client,
) -> Result<()> {
    let interval_time = {
        let config = config.read().await.clone();
        if config.fetch_on_demand {
            return Err(eyre!("Done"));
        }
        config.fetch_interval_seconds
    };
    let mut interval =
        tokio::time::interval(tokio::time::Duration::from_secs(interval_time.ok_or(
            eyre!("Worker thread not running, since calendars are not polled"),
        )?));
    loop {
        interval.tick().await;
        // for all calendars in the config, fetch them
        let config = config.read().await.clone();
        // if fetch_on_demand is true, continue
        if config.fetch_on_demand {
            continue;
        }
        for (path, calendar_config) in config.calendars {
            let _ = Calendar::from_config(client.clone(), calendar_config)
                .await
                .map(|calendar| cache.insert(path.clone(), calendar.to_string()))
                .wrap_err(eyre!("Failed to build calendar {}", path))
                .map_err(|e| println!("{:?}", e));
        }
    }
}

#[get("/<ident..>")]
async fn calendar<'a>(
    cache: &'a State<Arc<DashMap<String, String>>>,
    config: &'a State<Arc<RwLock<ApplicationConfig>>>,
    ident: PathBuf,
) -> impl Responder<'a, 'a> {
    let ident = ident.to_string_lossy().to_string();
    let config = { config.read().await.clone() };
    if config.fetch_on_demand {
        // populate cache with this calendar, iff it exists
        if !config.calendars.contains_key(&ident) {
            return Err("Calendar not found".to_string());
        }
        let _ = Calendar::from_config(Client::new(), config.calendars[&ident].clone())
            .await
            .map(|calendar| cache.insert(ident.clone(), calendar.to_string()))
            .wrap_err(eyre!("Failed to build calendar {}", ident))
            .map_err(|e| println!("{:?}", e));
    }
    let response = cache
        .get(&ident)
        .ok_or_else(|| "No calendar found".to_string())?
        .to_string();
    Ok::<(ContentType, String), String>((ContentType::Calendar, response))
}
