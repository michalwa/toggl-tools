use chrono::{Date, Local};
use hex_color::HexColor;
use reqwest::Url;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error as StdError;

pub struct TogglClient {
    http: reqwest::Client,
    api_token: String,
    projects: HashMap<(u32, u32), Project>,
}

impl TogglClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_token: api_key.into(),
            projects: HashMap::new(),
        }
    }

    pub async fn fetch_time_entries(
        &self,
        start_date: Date<Local>,
        end_date: Date<Local>,
    ) -> Result<Vec<TimeEntry>, Box<dyn StdError>> {
        let url = Url::parse_with_params(
            "https://api.track.toggl.com/api/v9/me/time_entries",
            [
                ("start_date", start_date.format("%Y-%m-%d").to_string()),
                ("end_date", end_date.format("%Y-%m-%d").to_string()),
            ],
        )?;

        Ok(self
            .http
            .get(url)
            .basic_auth(&self.api_token, Some("api_token"))
            .send()
            .await?
            .json::<Vec<TimeEntry>>()
            .await?)
    }

    pub async fn fetch_project(
        &mut self,
        workspace_id: u32,
        project_id: u32,
    ) -> Result<&Project, Box<dyn StdError>> {
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

#[derive(Debug, Deserialize)]
pub struct TimeEntry {
    pub description: String,
    pub workspace_id: u32,
    pub project_id: u32,
    #[serde(rename = "duration")]
    pub duration_seconds: u32,
}

#[derive(Deserialize)]
pub struct Project {
    pub name: String,
    pub color: HexColor,
}
