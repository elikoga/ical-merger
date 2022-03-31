use bytes::buf::{Buf, Reader};
use ical;
use std::io::BufReader;

pub async fn fetch_and_parse_calendar(
    url: &str,
) -> Result<ical::LineReader<BufReader<Reader<bytes::Bytes>>>, String> {
    let res = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let reader = BufReader::new(res.bytes().await.map_err(|e| e.to_string())?.reader());
    Ok(ical::LineReader::new(reader))
}

pub fn whitelist_blacklist(line: String) -> Option<String> {
    let allowed_prefixes = [
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
    for prefix in allowed_prefixes {
        if line.starts_with(prefix) {
            return Some(line.to_string());
        }
    }
    let disallowed_prefixes = [
        "LOCATION",
        // "DESCRIPTION",
        "STATUS",
        // "UID",
        "ORGANIZER",
        "TRANSP",
        // "DTSTAMP",
        "CREATED",
        "LAST-MODIFIED",
        "ATTENDEE",
        "X-MICROSOFT",
        "X-MOZ",
        "SEQUENCE",
        // "TRIGGER",
        "X-GOOGLE",
        // "ACTION",
        "ATTACH",
    ];
    for prefix in disallowed_prefixes {
        if line.starts_with(prefix) {
            return None;
        }
    }
    println!("{}", line);

    None
}

pub fn filter_map_summary(
    fun: impl Fn(String) -> Option<String>,
) -> impl Fn(String) -> Option<String> {
    move |line: String| {
        if line.starts_with("SUMMARY:") {
            // operate on SUMMARY:<THIS>
            let old_summary = line.split_at("SUMMARY:".len()).1;
            let new_summary = fun(old_summary.to_string())?;
            Some(format!("SUMMARY:{}", new_summary))
        } else {
            Some(line)
        }
    }
}

pub fn filter_map_description(
    fun: impl Fn(String) -> Option<String>,
) -> impl Fn(String) -> Option<String> {
    move |line: String| {
        if line.starts_with("DESCRIPTION:") {
            // operate on DESCRIPTION:<THIS>
            let old_description = line.split_at("DESCRIPTION:".len()).1;
            let new_description = fun(old_description.to_string())?;
            Some(format!("DESCRIPTION:{}", new_description))
        } else {
            Some(line)
        }
    }
}

pub fn strip_calendar_header(line: String) -> Option<String> {
    // if line is BEGIN:VCALENDAR or END:VCALENDAR
    // return None
    // else return Some(line)
    if line.starts_with("BEGIN:VCALENDAR") || line.starts_with("END:VCALENDAR") {
        return None;
    }
    Some(line)
}

pub fn merge_and_add_calendar_headers(calendars: Vec<String>) -> Result<String, String> {
    // merge all calendars
    // add BEGIN:VCALENDAR and END:VCALENDAR
    // return merged calendar
    let mut merged_calendar = String::new();
    merged_calendar.push_str("BEGIN:VCALENDAR\r\n");
    merged_calendar.push_str("VERSION:2.0\r\n");
    merged_calendar.push_str("PRODID:-//eli.kogan.wang/calendar");
    for calendar in calendars {
        merged_calendar.push_str(&calendar);
        merged_calendar.push_str("\r\n");
    }
    merged_calendar.push_str("END:VCALENDAR\r\n");
    // verify that merged calendar is valid
    // by parsing it
    let mut parser = ical::IcalParser::new(merged_calendar.as_bytes());
    let _ = parser.next().ok_or("Invalid calendar")?;
    Ok(merged_calendar)
}
