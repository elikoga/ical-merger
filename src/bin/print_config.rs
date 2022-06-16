use std::collections::HashMap;

use ical_merger::config::{ApplicationConfig, CalendarConfig, CalendarModification};

fn main() {
    print_config();
}

pub fn print_config() {
    let config: ApplicationConfig = HashMap::from([(
        "/calendar.ics".to_string(),
        CalendarConfig::MergeCalendars(
            Vec::from([
                CalendarConfig::ModifyCalendar(
                    Box::new(
                            CalendarConfig::FetchCalendar("https://calendar.google.com/calendar/ical/.../basic.ics".to_string())
                    ),
                    Vec::from([
                        CalendarModification::DefaultBlacklist,
                        CalendarModification::DefaultWhitelist,
                        CalendarModification::BlankPrefix("SUMMARY".to_string()),
                        CalendarModification::ReplacePrefix(
                            "DESCRIPTION".to_string(),
                            "[Personal]".to_string(),
                        ),
                    ])
                ),
                CalendarConfig::ModifyCalendar(
                    Box::new(
                        CalendarConfig::FetchCalendar("https://calendar.google.com/calendar/ical/.../basic.ics".to_string())
                    ), 
                    Vec::from([
                        CalendarModification::DefaultBlacklist,
                        CalendarModification::DefaultWhitelist,
                        CalendarModification::ReplacePrefix(
                            "SUMMARY".to_string(),
                            "[Uni] {{value}}".to_string(),
                        ),
                    ])
                ),
                CalendarConfig::FetchCalendar("https://...-caldav.icloud.com/published/...".to_string())
            ])
        ),
    )]);
    println!("{}", serde_json::to_string_pretty(&config).unwrap());
}
