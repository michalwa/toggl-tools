use chrono::{Date, Local, DateTime};
use hex_color::HexColor;
use reqwest::Url;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct TimeEntry {
    pub description: String,
    pub workspace_id: u32,
    pub project_id: Option<u32>,
    #[serde(rename = "duration")]
    duration_seconds: i32,
    pub start: DateTime<Local>,
}

impl TimeEntry {
    pub fn duration_seconds(&self) -> u32 {
        if self.duration_seconds >= 0 {
            self.duration_seconds as u32
        } else {
            (Local::now() - self.start).num_seconds() as u32
        }
    }
}

#[derive(Deserialize)]
pub struct Project {
    pub name: String,
    pub color: HexColor,
}

pub struct TogglClient {
    http: reqwest::Client,
    api_token: String,
    projects: HashMap<(u32, u32), Project>,
}

impl TogglClient {
    pub fn new(api_token: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_token: api_token.into(),
            projects: HashMap::new(),
        }
    }

    pub async fn fetch_time_entries(
        &self,
        start_date: Date<Local>,
        end_date: Date<Local>,
    ) -> reqwest::Result<Vec<TimeEntry>> {
        let url = Url::parse_with_params(
            "https://api.track.toggl.com/api/v9/me/time_entries",
            [
                ("start_date", start_date.format("%Y-%m-%d").to_string()),
                ("end_date", end_date.format("%Y-%m-%d").to_string()),
            ],
        )
        .unwrap();

        self.http
            .get(url)
            .basic_auth(&self.api_token, Some("api_token"))
            .send()
            .await?
            .json::<Vec<TimeEntry>>()
            .await
    }

    pub async fn fetch_project(
        &mut self,
        workspace_id: u32,
        project_id: u32,
    ) -> reqwest::Result<&Project> {
        if self.projects.contains_key(&(workspace_id, project_id)) {
            let project = &self.projects[&(workspace_id, project_id)];
            return Ok(project);
        }

        let project = self.http
            .get(format!("https://api.track.toggl.com/api/v9/workspaces/{workspace_id}/projects/{project_id}"))
            .basic_auth(&self.api_token, Some("api_token"))
            .send()
            .await?
            .json::<Project>()
            .await?;

        self.projects.insert((workspace_id, project_id), project);
        Ok(&self.projects[&(workspace_id, project_id)])
    }
}
