use std::{collections::HashMap, fs::File};

use elikoga_ical_rs::ContentLine;
use eyre::{eyre, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ApplicationConfig {
    pub calendars: HashMap<String, CalendarConfig>,
}

pub type TemplateString = String;
pub type Url = String;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum CalendarConfig {
    MergeCalendars(Vec<CalendarConfig>),
    ModifyCalendar(Box<CalendarConfig>, Vec<ComponentModification>),
    FetchCalendar(Url),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum ComponentModification {
    KeepPropertiesIfNameIn(Vec<String>),
    RemovePropertiesIfNameIn(Vec<String>),
    ReplacePropertiesValueIfNameIs(String, TemplateString),
    #[serde(with = "serde_with::rust::display_fromstr")]
    InsertProperty(ContentLine),
    KeepComponentsIfNameIn(Vec<String>),
    RemoveComponentsIfNameIn(Vec<String>),
    ModifyComponentsIfNameIs(String, Vec<ComponentModification>),
}

pub fn read_config_file() -> Result<ApplicationConfig> {
    // if there is a config.yaml file, read it and return it
    if let Ok(file) = File::open("config.yaml") {
        serde_yaml::from_reader(file).wrap_err(eyre!("Could not read config.yaml"))
    } else {
        serde_json::from_reader(
            File::open("config.json")
                .wrap_err(eyre!("Could not read config.yaml or config.json"))?,
        )
        .wrap_err(eyre!("Could not read config.json"))
    }
}
