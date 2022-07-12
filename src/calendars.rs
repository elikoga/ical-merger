use core::fmt;
use std::{
    fmt::{Display, Formatter},
    io::Cursor,
};

use elikoga_ical_rs::ICalObject;
use eyre::{eyre, Result};
use reqwest::Client;
use rocket::{http::ContentType, response::Responder, Response};
use serde::{Deserialize, Serialize};

use async_recursion::async_recursion;

use crate::config::{CalendarConfig, ComponentModification};

#[derive(Debug, Clone)]
pub struct Calendar(ICalObject);

impl Display for Calendar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Calendar {
    async fn fetch_calendar(client: Client, url: String) -> Result<Self> {
        println!("Fetching calendar from {}", url);
        let response = client.get(url).send().await?;
        let body = response.bytes().await?;
        let calendar = ICalObject::from_bufread(&mut Cursor::new(body))?;
        Ok(Calendar(calendar))
    }

    fn merge_calendars(calendars: Vec<Self>) -> Result<Self> {
        let modifications_all = [ComponentModification::RemovePropertiesIfNameIn(
            ["PRODID", "VERSION", "CALSCALE", "METHOD"]
                .map(String::from)
                .to_vec(),
        )];
        let modificaions_last = ["PRODID:ICal Merger".parse()?, "VERSION:2.0".parse()?]
            .map(ComponentModification::InsertProperty);
        // apply modifications to all calendars
        let mut new_calendars = Vec::new();
        for calendar in calendars
            .into_iter()
            .map(|calendar| calendar.apply_modifications(modifications_all.to_vec()))
        {
            let calendar = calendar?;
            // we also assert, that all object_types are "VCALENDAR"
            if !calendar.0.object_type.eq_ignore_ascii_case("VCALENDAR") {
                return Err(eyre!(
                    "Calendar object type is not VCALENDAR, but {}",
                    calendar.0.object_type
                ));
            }
            new_calendars.push(calendar);
        }
        // merge by mergin properties, components
        let properties = new_calendars
            .iter()
            .flat_map(|calendar| calendar.0.properties.clone())
            .collect::<Vec<_>>();
        let sub_objects = new_calendars
            .iter()
            .flat_map(|calendar| calendar.0.sub_objects.clone())
            .collect::<Vec<_>>();
        let calendar = Calendar(ICalObject {
            object_type: "VCALENDAR".to_string(),
            properties,
            sub_objects,
        });
        // apply modifications to last calendar
        let calendar = calendar.apply_modifications(modificaions_last.to_vec())?;
        Ok(calendar)
    }

    pub fn apply_modifications(&self, modifications: Vec<ComponentModification>) -> Result<Self> {
        let mut calendar = self.clone();
        for modification in modifications {
            calendar = modification.apply(calendar)?;
        }
        Ok(calendar)
    }

    #[async_recursion]
    pub async fn from_config(client: Client, config: CalendarConfig) -> Result<Self> {
        match config {
            CalendarConfig::FetchCalendar(url) => Self::fetch_calendar(client.clone(), url).await,
            CalendarConfig::MergeCalendars(calendars) => Self::merge_calendars({
                let mut new_calendars = Vec::new();
                for calendar in calendars
                    .into_iter()
                    .map(|calendar| Calendar::from_config(client.clone(), calendar))
                {
                    new_calendars.push(calendar.await?);
                }
                new_calendars
            }),
            CalendarConfig::ModifyCalendar(calendar, modifications) => {
                let calendar = Calendar::from_config(client.clone(), *calendar).await?;
                let calendar = calendar.apply_modifications(modifications)?;
                Ok(calendar)
            }
        }
    }
}

impl ComponentModification {
    pub fn apply_component(&self, mut calendar: ICalObject) -> Result<ICalObject> {
        match self {
            ComponentModification::InsertProperty(property) => {
                calendar.properties.push(property.clone());
                Ok(calendar)
            }
            ComponentModification::KeepComponentsIfNameIn(names) => {
                // compare with eq_ignore_ascii_case
                let mut new_sub_objects = Vec::new();
                for sub_object in calendar.sub_objects.clone() {
                    if names
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&sub_object.object_type))
                    {
                        new_sub_objects.push(sub_object);
                    }
                }
                calendar.sub_objects = new_sub_objects;
                Ok(calendar)
            }
            ComponentModification::KeepPropertiesIfNameIn(names) => {
                // compare with eq_ignore_ascii_case
                let mut new_properties = Vec::new();
                for property in calendar.properties.clone() {
                    if names
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&property.name))
                    {
                        new_properties.push(property);
                    }
                }
                calendar.properties = new_properties;
                Ok(calendar)
            }
            ComponentModification::ModifyComponentsIfNameIs(name, modifications) => {
                let mut new_sub_objects = Vec::new();
                for mut sub_object in calendar.sub_objects.clone() {
                    if sub_object.object_type.eq_ignore_ascii_case(name) {
                        for modification in modifications {
                            sub_object = modification.apply_component(sub_object)?;
                        }
                        new_sub_objects.push(sub_object);
                    } else {
                        new_sub_objects.push(sub_object);
                    }
                }
                calendar.sub_objects = new_sub_objects;
                Ok(calendar)
            }
            ComponentModification::RemoveComponentsIfNameIn(names) => {
                // compare with eq_ignore_ascii_case
                let mut new_sub_objects = Vec::new();
                for sub_object in calendar.sub_objects.clone() {
                    if !names
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&sub_object.object_type))
                    {
                        new_sub_objects.push(sub_object);
                    }
                }
                calendar.sub_objects = new_sub_objects;
                Ok(calendar)
            }
            ComponentModification::RemovePropertiesIfNameIn(names) => {
                // compare with eq_ignore_ascii_case
                let mut new_properties = Vec::new();
                for property in calendar.properties.clone() {
                    if !names
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&property.name))
                    {
                        new_properties.push(property);
                    }
                }
                calendar.properties = new_properties;
                Ok(calendar)
            }
            ComponentModification::ReplacePropertiesValueIfNameIs(name, replacement) => {
                let mut new_properties = Vec::new();
                for mut property in calendar.properties.clone() {
                    if property.name.eq_ignore_ascii_case(name) {
                        // template {{ value }}
                        property.value = replacement.replace("{{ value }}", &property.value);
                        new_properties.push(property);
                    } else {
                        new_properties.push(property);
                    }
                }
                calendar.properties = new_properties;
                Ok(calendar)
            }
        }
    }

    pub fn apply(self, calendar: Calendar) -> Result<Calendar> {
        let calendar = calendar.0;
        let calendar = self.apply_component(calendar)?;
        Ok(Calendar(calendar))
    }
}

#[derive(Serialize, Deserialize)]
struct RenderingValue {
    value: String,
}

impl<'r> Responder<'r, 'static> for Calendar {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let serialized = self.to_string();
        Response::build()
            .sized_body(serialized.len(), Cursor::new(serialized))
            .header(ContentType::Calendar)
            .ok()
    }
}
