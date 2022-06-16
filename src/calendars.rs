use std::io::Cursor;

use handlebars::Handlebars;
use reqwest::Client;
use rocket::{http::ContentType, response::Responder, Response};
use serde::{Deserialize, Serialize};

use async_recursion::async_recursion;

use crate::config::{CalendarConfig, CalendarModification};

#[derive(Deserialize, Serialize, Clone)]
pub struct Calendar(String);

impl Calendar {
    pub fn new(calendar: String) -> Result<Self, String> {
        // verify that calendar is valid
        // by parsing it
        let mut parser = ical::IcalParser::new(calendar.as_bytes());
        let _ = parser.next().ok_or("Invalid calendar")?;
        Ok(Self(calendar))
    }

    async fn fetch_calendar(client: Client, url: String) -> Result<Self, String> {
        let response = client.get(url).send().await.map_err(|e| e.to_string())?;
        let body = response.text().await.map_err(|e| e.to_string())?;
        Self::new(body)
    }

    pub fn strip_calendar_header(self) -> String {
        // split into lines, then filter out lines that are headers
        // then join back into a single string
        let mut lines = self.0.lines();
        let mut stripped_calendar = String::new();
        while let Some(line) = lines.next() {
            if line.starts_with("BEGIN:VCALENDAR") || line.starts_with("END:VCALENDAR") {
                continue;
            }
            stripped_calendar.push_str(line);
            stripped_calendar.push_str("\r\n");
        }
        stripped_calendar
    }

    async fn merge_calendars(calendars: Vec<Self>) -> Result<Self, String> {
        let mut merged_calendar = String::new();
        merged_calendar.push_str("BEGIN:VCALENDAR\r\n");
        merged_calendar.push_str("VERSION:2.0\r\n");
        merged_calendar.push_str("PRODID:-//eli.kogan.wang/calendar");
        for calendar in calendars {
            // strip calendar header
            let stripped_calendar = Self::strip_calendar_header(calendar);
            merged_calendar.push_str(&stripped_calendar);
            merged_calendar.push_str("\r\n");
        }
        merged_calendar.push_str("END:VCALENDAR\r\n");
        // verify that merged calendar is valid
        // by parsing it
        Self::new(merged_calendar)
    }

    pub fn apply_modifications(
        self,
        modifications: Vec<CalendarModification>,
    ) -> Result<Self, String> {
        let mut calendar = self;
        for modification in modifications {
            calendar = modification.apply(calendar)?;
        }
        Ok(calendar)
    }

    #[async_recursion]
    pub async fn from_config(config: CalendarConfig) -> Result<Self, String> {
        match config {
            CalendarConfig::MergeCalendars(calendar_configs) => {
                let calendar_results = futures::future::join_all(
                    calendar_configs
                        .into_iter()
                        .map(|calendar_configs| Calendar::from_config(calendar_configs)),
                )
                .await;
                let mut calendars = Vec::new();
                for calendar_result in calendar_results {
                    calendars.push(calendar_result?);
                }
                Self::merge_calendars(calendars).await
            }
            CalendarConfig::ModifyCalendar(calendar, modifications) => {
                let calendar = Self::from_config(*calendar).await?;
                calendar.apply_modifications(modifications)
            }
            CalendarConfig::FetchCalendar(url) => Self::fetch_calendar(Client::new(), url).await,
            CalendarConfig::LiteralCalendar(calendar) => Ok(calendar),
        }
    }
}

// #[derive(Deserialize, Serialize, Clone)]
// pub enum CalendarModification {
//     BlacklistPrefix(Prefix),
//     WhitelistPrefix(Prefix),
//     DefaultWhitelist,
//     DefaultBlacklist,
//     BlankPrefix(Prefix),
//     ReplacePrefix(Prefix, TemplateString),
// }

impl CalendarModification {
    const DEFAULT_WHITELIST: [&'static str; 20] = [
        "BEGIN",
        "END",
        "DTSTART",
        "DTEND",
        "SUMMARY",
        "DESCRIPTION",
        "UID",
        "DTSTAMP",
        "ACTION",
        "TRIGGER",
        "TZID",
        "TZOFFSETTO",
        "TZOFFSETFROM",
        "RRULE",
        "TZNAME",
        "X-LIC-LOCATION",
        "CALSCALE",
        "X-WR-TIMEZONE",
        "TZNAME",
        "RECURRENCE-ID",
    ];
    const DEFAULT_BLACKLIST: [&'static str; 12] = [
        "LOCATION",
        "STATUS",
        "ORGANIZER",
        "TRANSP",
        "CREATED",
        "LAST-MODIFIED",
        "ATTENDEE",
        "X-MICROSOFT",
        "X-MOZ",
        "SEQUENCE",
        "X-GOOGLE",
        "ATTACH",
    ];

    fn apply_whitelist(line: String, prefix: String) -> Option<String> {
        if line.starts_with(&prefix) {
            return Some(line);
        }
        None
    }
    fn apply_default_whitelist(line: String) -> Option<String> {
        for default_whitelist_item in Self::DEFAULT_WHITELIST.iter() {
            if line.starts_with(default_whitelist_item) {
                return Some(line);
            }
        }
        None
    }
    fn apply_blacklist(line: String, prefix: String) -> Option<String> {
        if line.starts_with(&prefix) {
            return None;
        }
        Some(line)
    }
    fn apply_default_blacklist(line: String) -> Option<String> {
        for default_blacklist_item in Self::DEFAULT_BLACKLIST.iter() {
            if line.starts_with(default_blacklist_item) {
                return None;
            }
        }
        Some(line)
    }
    fn apply_blank_prefix(line: String, prefix: String) -> Option<String> {
        if line.starts_with(&prefix) {
            return None;
        }
        Some(line)
    }

    fn apply_replace_prefix(
        line: String,
        prefix: String,
        replacement: String,
    ) -> Result<String, String> {
        if line.starts_with(&prefix) {
            // first, skip the prefix
            let value = line.split_at(prefix.len()).1.to_string();
            // then, replace the content with handlebars.render_template
            let handlebars = Handlebars::new();
            let rendered_template = handlebars
                .render_template(&replacement, &RenderingValue { value })
                .map_err(|e| e.to_string())?;
            let new_line = format!("{}:{}", prefix, rendered_template);
            return Ok(new_line);
        }
        Ok(line)
    }
    pub fn apply(self, calendar: Calendar) -> Result<Calendar, String> {
        let calendar_lines = calendar.0.lines();
        let calendar_lines = calendar_lines.filter_map(|line| match &self {
            CalendarModification::BlacklistPrefix(prefix) => {
                Self::apply_blacklist(line.to_string(), prefix.to_string())
            }
            CalendarModification::WhitelistPrefix(prefix) => {
                Self::apply_whitelist(line.to_string(), prefix.to_string())
            }
            CalendarModification::DefaultWhitelist => {
                Self::apply_default_whitelist(line.to_string())
            }
            CalendarModification::DefaultBlacklist => {
                Self::apply_default_blacklist(line.to_string())
            }
            CalendarModification::BlankPrefix(prefix) => {
                Self::apply_blank_prefix(line.to_string(), prefix.to_string())
            }
            CalendarModification::ReplacePrefix(prefix, replacement) => Some(
                Self::apply_replace_prefix(
                    line.to_string(),
                    prefix.to_string(),
                    replacement.to_string(),
                )
                .unwrap(),
            ),
        });
        let calendar = calendar_lines.collect::<Vec<String>>().join("\n");
        Calendar::new(calendar)
    }
}

#[derive(Serialize, Deserialize)]
struct RenderingValue {
    value: String,
}

impl<'r> Responder<'r, 'static> for Calendar {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .sized_body(self.0.len(), Cursor::new(self.0))
            .header(ContentType::Calendar)
            .ok()
    }
}
