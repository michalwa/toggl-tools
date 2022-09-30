use crate::toggl::TogglClient;
use chrono::{Date, Duration, Local};
use clap::Args;
use itertools::Itertools;
use std::{
    env,
    error::Error as StdError,
    io::{stdout, Write},
};
use tabwriter::TabWriter;

#[derive(Args)]
pub struct SummaryArgs {
    #[arg(short, long, value_parser = parse_human_date)]
    start_date: Option<Date<Local>>,
    #[arg(short, long, value_parser = parse_human_date)]
    end_date: Option<Date<Local>>,
}

fn parse_human_date(string: &str) -> chrono_english::DateResult<Date<Local>> {
    chrono_english::parse_date_string(string, Local::now(), chrono_english::Dialect::Uk)
        .map(|dt| dt.date())
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

pub async fn run_summary(args: SummaryArgs) -> Result<(), Box<dyn StdError>> {
    let mut client =
        TogglClient::new(env::var("TOGGL_API_TOKEN").expect("TOGGL_API_TOKEN must be set"));

    let mut time_entries = client
        .fetch_time_entries(args.start_date(), args.end_date())
        .await?;

    time_entries.sort_by_key(|e| (e.workspace_id, e.project_id));

    let mut tabwriter = TabWriter::new(stdout());

    for ((workspace_id, project_id), time_entries) in &time_entries
        .iter()
        .group_by(|e| (e.workspace_id, e.project_id))
    {
        let mut time_entries = time_entries.collect::<Vec<_>>();
        time_entries.sort_by_key(|e| &e.description);

        let project = client.fetch_project(workspace_id, project_id).await?;
        let total_duration_secs: u32 = time_entries.iter().map(|e| e.duration_seconds).sum();
        let total_duration = Duration::seconds(total_duration_secs as i64);

        writeln!(
            &mut tabwriter,
            "{:02}:{:02}:{:02}\t{}\t{}",
            total_duration.num_hours(),
            total_duration.num_minutes() % 60,
            total_duration.num_seconds() % 60,
            project.name,
            time_entries.into_iter().map(|e| &e.description).join(", ")
        )
        .unwrap();
    }

    tabwriter.flush().unwrap();

    Ok(())
}
