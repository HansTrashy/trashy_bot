use chrono::Duration;

static OTHER_MOD_CMD: [char; 3] = ['%', '=', '$'];

pub fn sanitize_for_other_bot_commands(output: &str) -> String {
    output
        .chars()
        .filter(|&c| !OTHER_MOD_CMD.contains(&c))
        .collect::<String>()
}

pub fn parse_duration(duration_str: &str) -> Option<Duration> {
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
    let days = duration.num_days();
    let hours = duration.num_hours() - days * 24i64;
    let minutes = duration.num_minutes() - (days * 24i64 * 60i64) - (hours * 60i64);

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

#[cfg(test)]
mod tests {
    use super::{humanize_duration, Duration};

    #[test]
    fn check_humanized_duration() {
        let duration = humanize_duration(&Duration::minutes(1440));

        assert_eq!(duration, "1 day".to_string());

        let duration = humanize_duration(&Duration::seconds(86400));

        assert_eq!(duration, "1 day".to_string());

        let duration = humanize_duration(&Duration::seconds(86399));

        assert_eq!(duration, "23 hours 59 minutes".to_string());
    }
}
