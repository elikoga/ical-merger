use std::sync::Arc;

use config::{ApplicationConfig, CalendarConfig};
use rocket::{
    http::Status,
    request::{self, FromRequest, Request},
    State,
};
use tokio::sync::RwLock;

pub mod calendars;
pub mod config;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CalendarConfig {
    type Error = String;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let calendar_config = req
            .guard::<&State<Arc<RwLock<ApplicationConfig>>>>()
            .await
            .map_failure(|_| {
                (
                    Status::ServiceUnavailable,
                    "No calendar config found in state".to_string(),
                )
            });
        let calendar_config = match calendar_config {
            rocket::outcome::Outcome::Success(config) => {
                let config = config
                .clone();
                let config = config.read().await;
                let path = req.uri().path().as_str();
                match config.get(path) {
                    Some(calendar_config) => request::Outcome::Success(calendar_config.clone()),
                    None => request::Outcome::Forward(()),
                }
            }
            rocket::outcome::Outcome::Failure(_) => todo!(),
            rocket::outcome::Outcome::Forward(_) => todo!(),
        };

        calendar_config
    }
}
