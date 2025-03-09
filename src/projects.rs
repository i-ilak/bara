use minijinja::context;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Project {
    pub title: String,
    pub link: String,
    pub description: String,
}

impl Project {
    pub fn card_jinja_context(&self) -> minijinja::Value {
        context! {
            description => self.description,
            link => self.link,
            title => self.title,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Projects {
    pub projects: Vec<Project>,
}

pub fn create_projects(projects_path: &Path) -> Vec<Project> {
    let projects_str = fs::read_to_string(&projects_path).expect("Could not read file!");
    let projects: Projects =
        serde_yaml::from_str(&projects_str).expect("Error parsing projects.yml!");
    projects.projects
}
