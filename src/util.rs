//! utility functions for the trashy bot

use regex::Regex;
use sqlx::postgres::PgPool;
use std::time::Instant;
use time::Duration;

use crate::error::TrashyCommandError;

// pub async fn timed_request(
//     client: &reqwest::Client,
//     url: &str,
// ) -> Result<(serde_json::Value, std::time::Duration), TrashyCommandError> {
//     let pre_request_time = Instant::now();
//     let res: serde_json::Value = client.get(url).send().await?.json().await?;

//     Ok((res, pre_request_time.elapsed()))
// }

/// parses a date or a duration str
///
/// duration str look like `1d` or `24h`, dates look like `2021-05-23 12:00`
pub fn parse_duration_or_date(duration_str: &str) -> Option<Duration> {
    //TODO: update to time 0.3 when sqlx supports it and then support dates with time
    // let format = time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]");
    // let date = time::OffsetDateTime::parse(duration_str, &format);

    // match date {
    //     Ok(datetime) => Some((time::OffsetDateTime::now_utc() - datetime).abs()),
    //     Err(_) => {
    //
    //     }
    // }

    let (digits, non_digits) = duration_str.chars().fold(
        (String::with_capacity(5), String::with_capacity(5)),
        |(mut d, mut nd), elem| {
            if elem.is_digit(10) {
                d.push(elem);
            } else {
                nd.push(elem);
            }
            (d, nd)
        },
    );

    if let Ok(n) = digits.parse::<i64>() {
        match non_digits.as_ref() {
            "s" => Some(Duration::seconds(n)),
            "m" => Some(Duration::minutes(n)),
            "h" => Some(Duration::hours(n)),
            "d" => Some(Duration::days(n)),
            "w" => Some(Duration::weeks(n)),
            _ => None,
        }
    } else {
        None
    }
}

pub fn humanize_duration(duration: &Duration) -> String {
    let days = duration.whole_days();
    let hours = duration.whole_hours() - days * 24_i64;
    let minutes = duration.whole_minutes() - (days * 24_i64 * 60_i64) - (hours * 60_i64);

    match (days, hours, minutes) {
        (0, 0, 0) => "less than one minute".to_string(),
        (0, 0, 1) => "1 minute".to_string(),
        (0, 0, x) => format!("{} minutes", x),
        (0, 1, 0) => "1 hour".to_string(),
        (0, x, 0) => format!("{} hours", x),
        (0, x, y) => format!("{} hours {} minutes", x, y),
        (1, 0, 0) => "1 day".to_string(),
        (x, 0, 0) => format!("{} days", x),
        (x, y, 0) => format!("{} days {} hours", x, y),
        (x, 0, y) => format!("{} days {} minutes", x, y),
        (x, y, z) => format!("{} days {} hours {} minutes", x, y, z),
    }
}

pub fn parse_message_link(regex: &Regex, link: &str) -> Result<(u64, u64, u64), String> {
    let caps = regex.captures(link).ok_or("No captures, invalid link?")?;
    let server_id = caps
        .get(1)
        .map_or("", |m| m.as_str())
        .parse::<u64>()
        .map_err(|_| "Failed parsing to u64")?;
    let channel_id = caps
        .get(2)
        .map_or("", |m| m.as_str())
        .parse::<u64>()
        .map_err(|_| "Failed parsing to u64")?;
    let msg_id = caps
        .get(3)
        .map_or("", |m| m.as_str())
        .parse::<u64>()
        .map_err(|_| "Failed parsing to u64")?;

    Ok((server_id, channel_id, msg_id))
}

#[cfg(test)]
mod tests {
    use super::{humanize_duration, parse_duration_or_date, Duration};

    #[test]
    fn check_humanized_duration() {
        let duration = humanize_duration(&Duration::minutes(1440));

        assert_eq!(duration, "1 day".to_string());

        let duration = humanize_duration(&Duration::seconds(86400));

        assert_eq!(duration, "1 day".to_string());

        let duration = humanize_duration(&Duration::seconds(86399));

        assert_eq!(duration, "23 hours 59 minutes".to_string());
    }

    #[test]
    fn parsing_duration_or_datetime() {
        // let r = parse_duration_or_date("16:00");
        // let format = time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]");

        // let date = time::OffsetDateTime::parse("16:00", &format);

        // println!("{:?}", date);
    }
}
