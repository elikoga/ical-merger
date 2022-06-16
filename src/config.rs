use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::calendars::Calendar;

pub type ApplicationConfig = HashMap<String, CalendarConfig>;

pub type TemplateString = String;
pub type Prefix = String;
pub type Url = String;

#[derive(Deserialize, Serialize, Clone)]
pub enum CalendarConfig {
    MergeCalendars(Vec<CalendarConfig>),
    ModifyCalendar(Box<CalendarConfig>, Vec<CalendarModification>),
    FetchCalendar(Url),
    LiteralCalendar(Calendar),
}

#[derive(Deserialize, Serialize, Clone)]
pub enum CalendarModification {
    BlacklistPrefix(Prefix),
    WhitelistPrefix(Prefix),
    DefaultWhitelist,
    DefaultBlacklist,
    BlankPrefix(Prefix),
    ReplacePrefix(Prefix, TemplateString),
}
