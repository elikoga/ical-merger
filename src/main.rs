use dashmap::DashMap;
use reqwest::Client;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use ical_merger::{calendars::Calendar, config::read_config_file};
use rocket::{get, http::ContentType, response::Responder, routes, State};

#[rocket::launch]
async fn rocket() -> _ {
    // load Arc<RwLock<ApplicationConfig>> from config.json
    let config = read_config_file().unwrap();
    let config = Arc::new(RwLock::new(config));
    let cache: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
    tokio::spawn({
        let cache = cache.clone();
        let client = Client::new();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // every minute
        async move {
            loop {
                interval.tick().await;
                // for all calendars in the config, fetch them
                let config = config.read().await.clone();
                for (path, calendar_config) in config.calendars {
                    let calendar = Calendar::from_config(client.clone(), calendar_config)
                        .await
                        .expect("Could not fetch calendar");
                    // insert the calendar into the cache
                    cache.insert(path.clone(), calendar.to_string());
                }
            }
        }
    });
    rocket::build().mount("/", routes![calendar]).manage(cache)
}

#[get("/<ident..>")]
async fn calendar(cache: &State<Arc<DashMap<String, String>>>, ident: PathBuf) -> impl Responder {
    let ident = ident.to_string_lossy().to_string();
    let response = cache
        .get(&ident)
        .ok_or_else(|| "No calendar found".to_string())?
        .to_string();
    Ok::<(ContentType, String), String>((ContentType::Calendar, response))
}
