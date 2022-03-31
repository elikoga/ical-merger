use rocket::{http::ContentType, routes};

mod do_calendar;
use do_calendar::do_calendar;

#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![ekwics])
}

#[rocket::get("/fcio-ekw.ics")]
async fn ekwics() -> (ContentType, String) {
    (ContentType::Calendar, do_calendar().await.unwrap())
}
