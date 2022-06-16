use std::{path::PathBuf, fs::File, sync::{Arc}};
use tokio::sync::RwLock;

use ical_merger::{calendars::Calendar, config::{CalendarConfig, ApplicationConfig}};
use rocket::{get, routes};

#[rocket::launch]
fn rocket() -> _ {
    // load Arc<RwLock<ApplicationConfig>> from config.json
    let config = File::open("config.json").unwrap();
    let config: ApplicationConfig = serde_json::from_reader(config).unwrap();
    let config = Arc::new(RwLock::new(config));
    rocket::build()
    .mount("/", routes![calendar])
    .manage(
        config
    )
}

#[get("/<ident..>")]
async fn calendar(config: CalendarConfig, ident: PathBuf) -> Result<Calendar, String> {
    Calendar::from_config(config).await
}
