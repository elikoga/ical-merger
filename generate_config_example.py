#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p "python310.withPackages(ps: with ps; [pyyaml])"

import yaml
from generate_config_lib import (
    CalendarConfig,
    fetch_calendar,
    merge_calendars,
)

calendar_1 = fetch_calendar("https://calendar.google.com/calendar/ical/...")
calendar_2 = fetch_calendar("https://calendar.google.com/calendar/ical/...")
calendar_3 = fetch_calendar("https://p43-caldav.icloud.com/published")

calendar_1.keep_components_if_name_in(["VEVENT", "TIMEZONE"])


def blank_event(e: CalendarConfig):
    e.keep_properties_if_name_in(
        [
            "DTSTAMP",
            "UID",
            "DTSTART",
            "DTEND",
            "EXDATE",
            "EXRULE",
            "RDATE",
            "RRULE",
            "ATTENDEE",
        ]
    )
    e.insert_property("SUMMARY:[Personal]")


calendar_1.modify_components_if_name_is("VEVENT", blank_event)

calendar_2.keep_components_if_name_in(["VEVENT", "VTIMEZONE"])


def template_uni_event(e: CalendarConfig):
    e.replace_properties_value_if_name_is("SUMMARY", "[Uni] {{ value }}")


calendar_2.modify_components_if_name_is("VEVENT", template_uni_event)


def template_fcio_event(e: CalendarConfig):
    e.replace_properties_value_if_name_is("SUMMARY", "[fcio] {{ value }}")


calendar_3.modify_components_if_name_is(
    "VEVENT",
    template_fcio_event,
)

# merge all of them
calendar_all = merge_calendars([calendar_1, calendar_2, calendar_3])

# dump to stdout
print(
    yaml.dump(
        {
            "calendars": {
                "example.ics": calendar_all,
            }
        },
        default_flow_style=False,
    )
)
