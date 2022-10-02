use crate::time_utils::{parse_human_date, TimeResolution};
use crate::toggl::{Project, TogglClient};
use chrono::{Date, Duration, Local};
use clap::Args;
use colored::Colorize;
use itertools::Itertools;
use std::env;

#[derive(Args)]
pub struct SummaryArgs {
    #[arg(short, long, value_parser = parse_human_date, help = "[default: today]")]
    start_date: Option<Date<Local>>,
    #[arg(short, long, value_parser = parse_human_date, help = "[default: start_date + 1 day]")]
    end_date: Option<Date<Local>>,
    #[arg(short, long, default_value_t = TimeResolution::Minutes)]
    time_resolution: TimeResolution,
}

impl SummaryArgs {
    fn start_date(&self) -> Date<Local> {
        self.start_date.unwrap_or_else(Local::today)
    }

    fn end_date(&self) -> Date<Local> {
        self.end_date
            .unwrap_or_else(|| self.start_date() + Duration::days(1))
    }
}

/// Fetches, calculates and prints the summary
pub async fn run_summary(args: SummaryArgs) -> reqwest::Result<()> {
    use hex_color::HexColor;

    let default_project = Project {
        name: "(no project)".into(),
        color: HexColor {
            r: 0x7f,
            g: 0x7f,
            b: 0x7f,
            a: 0xff,
        },
    };

    let mut client =
        TogglClient::new(env::var("TOGGL_API_TOKEN").expect("TOGGL_API_TOKEN must be set"));

    let mut time_entries = client
        .fetch_time_entries(args.start_date(), args.end_date())
        .await?;

    time_entries.sort_by_key(|e| (e.workspace_id, e.project_id));

    for ((workspace_id, project_id), time_entries) in &time_entries
        .iter()
        .group_by(|e| (e.workspace_id, e.project_id))
    {
        let time_entries = time_entries
            .filter(|e| !e.description.is_empty())
            .sorted_by_key(|e| &e.description)
            .collect::<Vec<_>>();

        let project = if let Some(project_id) = project_id {
            client.fetch_project(workspace_id, project_id).await?
        } else {
            &default_project
        };

        let total_duration_secs: u32 = time_entries.iter().map(|e| e.duration_seconds).sum();
        let total_duration = Duration::seconds(total_duration_secs as i64);
        let total_duration_formatted = args
            .time_resolution
            .format_duration(&total_duration)
            .bright_black();

        println!(
            "{} {:-20} {}",
            total_duration_formatted,
            project
                .name
                .truecolor(project.color.r, project.color.g, project.color.b),
            time_entries.into_iter().map(|e| &e.description).join(", ")
        );
    }

    Ok(())
}
