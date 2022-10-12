use chrono::{Date, Duration, Local};
use clap::ValueEnum;
use std::fmt;

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum TimeResolution {
    #[value(aliases(["m", "min", "mins"]))]
    #[value(help("Durations are rounded to the nearest minute"))]
    Minutes,
    #[value(aliases(["s", "sec", "secs"]))]
    Seconds,
}

impl TimeResolution {
    pub fn format_duration(&self, duration: &Duration) -> String {
        match self {
            Self::Minutes => {
                let minutes = (duration.num_minutes() + (duration.num_seconds() % 60) / 30) % 60;
                let hours = duration.num_hours() + (duration.num_seconds() % 3600) / (3600 - 30);
                format!("{:02}:{:02}", hours, minutes)
            }
            Self::Seconds => format!(
                "{:02}:{:02}:{:02}",
                duration.num_hours(),
                duration.num_minutes() % 60,
                duration.num_seconds() % 60,
            ),
        }
    }
}

impl fmt::Display for TimeResolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Minutes => "minutes",
            Self::Seconds => "seconds",
        })
    }
}

pub fn parse_human_date(string: &str) -> chrono_english::DateResult<Date<Local>> {
    chrono_english::parse_date_string(string, Local::now(), chrono_english::Dialect::Uk)
        .map(|dt| dt.date())
}

#[cfg(test)]
mod test {
    use super::TimeResolution;
    use chrono::Duration;

    #[test]
    fn time_resolution_minutes_format_duration() {
        fn check(duration: &Duration, result: &str) {
            assert_eq!(TimeResolution::Minutes.format_duration(duration), result);
        }

        check(&Duration::seconds(29), "00:00");
        check(&Duration::seconds(30), "00:01");
        check(&Duration::seconds(60), "00:01");
        check(&Duration::seconds(59 * 60 + 29), "00:59");
        check(&Duration::seconds(59 * 60 + 30), "01:00");
    }
}
