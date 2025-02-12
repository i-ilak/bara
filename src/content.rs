use crate::map::{fetch_route, Coordinate, MapData};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Taxonomies {
    pub title: String,
    pub description: String,
    pub date: NaiveDate,
    pub map: Option<MapData>,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct Content {
    taxonomies: Taxonomies,
    post_html: String,
    source_file_path: Option<PathBuf>,
    serve_file_path: Option<PathBuf>,
    rss_item: Option<String>,
}

impl Content {
    pub fn new(taxonomies: Taxonomies, content: String) -> Self {
        Content {
            taxonomies,
            post_html: content,
            source_file_path: None,
            serve_file_path: None,
            rss_item: None,
        }
    }

    pub fn source_file_path(&self) -> Option<&PathBuf> {
        self.source_file_path.as_ref()
    }

    pub fn set_source_file_path(&mut self, path: PathBuf) {
        if path.as_os_str().is_empty() {
            return;
        }
        self.source_file_path = Some(path.clone());
        let mut file_path = path.to_string_lossy().replace("/content", "");
        file_path = file_path.replace(".md", ".html");
        self.serve_file_path = Some(PathBuf::from(&file_path[1..]));
    }

    pub fn post_title(&self) -> &str {
        &self.taxonomies.title
    }

    pub fn post_date(&self) -> &NaiveDate {
        &self.taxonomies.date
    }

    pub fn post_description(&self) -> &str {
        &self.taxonomies.description
    }
}

pub struct Project {
    content: Content,
}

impl Project {
    pub fn new(taxonomies: Taxonomies, content: String) -> Self {
        Project {
            content: Content::new(taxonomies, content),
        }
    }
}

#[derive(Debug)]
pub struct Post {
    content: Content,
    external_link: Option<String>,
    downloaded_link: Option<String>,
}

impl Post {
    pub fn new(taxonomies: Taxonomies, content: String) -> Self {
        let external_link = None;
        let downloaded_link = None;
        let route: Vec<Coordinate>;

        if let Some(map) = &taxonomies.map {
            route = fetch_route(&map.waypoints).expect("Failed to fetch route!");
        }
        let post = Post {
            content: Content::new(taxonomies, content),
            external_link,
            downloaded_link,
        };
        post.content.taxonomies.map.unwrap().route = Some(route);

        post
    }

    pub fn serialize(&self) -> serde_json::Value {
        let mut result = serde_json::json!({
            "title": self.content.post_title(),
            "date": self.content.post_date().format("%Y-%m-%d").to_string(),
            "description": self.content.post_description(),
        });

        if let Some(serve_file_path) = &self.content.serve_file_path {
            result["serve_file_path"] =
                serde_json::json!(format!("BASEPATH/{}", serve_file_path.display()));
        }

        if !self.content.taxonomies.tags.is_empty() {
            result["tags"] = serde_json::json!(self.content.taxonomies.tags);
        }

        result
    }

    pub fn uuid(&self) -> String {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, format!("{:?}", self).as_bytes()).to_string()
    }
}
