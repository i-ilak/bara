use crate::config::ConfigFile;
use crate::map::MapData;
use crate::markdown_parsing::markdown_to_html;
use chrono::NaiveDate;
use minijinja::context;
use minijinja::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

use regex::Regex;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternInfo {
    pub author: String,
    pub date: NaiveDate,
    pub link: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Taxonomies {
    pub title: String,
    pub description: Option<String>,
    pub date: NaiveDate,
    pub map: Option<MapData>,
    pub tags: Vec<String>,
    pub extern_info: Option<ExternInfo>,
    pub downloaded_link: Option<String>,
}

#[derive(Debug)]
pub struct Post {
    pub taxonomies: Taxonomies,
    pub post_html: String,
    pub source_file_path: Option<PathBuf>,
    pub serve_file_path: Option<PathBuf>,
    pub link_file_path: Option<PathBuf>,
}

pub fn transform_tags(tags: &Vec<String>) -> Vec<String> {
    let mut tags_with_hash: Vec<String> = Vec::with_capacity(tags.len());
    for tag in tags {
        tags_with_hash.push(format!("#{}", tag));
    }
    tags_with_hash
}

impl Post {
    pub fn new(path: &Path, root: &Path) -> Self {
        let (taxonomies, post_html) = parse_content(path)
            .expect("Could not parse taxonomies and content. Double check markdown!");

        let stripped_path = path.strip_prefix(root.join("content")).unwrap();

        let serve_path = stripped_path
            .parent()
            .unwrap()
            .join(stripped_path.file_stem().unwrap())
            .join("index.html");

        let post = Post {
            taxonomies,
            post_html,
            source_file_path: Some(path.to_path_buf()),
            serve_file_path: Some(serve_path.clone()),
            link_file_path: Some(Path::new("BASEPATH").join(serve_path.parent().expect("X"))),
        };

        post
    }

    pub fn serialize(&self) -> serde_json::Value {
        let mut result = serde_json::json!({
            "title": self.taxonomies.title,
            "date": self.taxonomies.date.format("%Y-%m-%d").to_string(),
            "description": self.taxonomies.description,
        });

        if let Some(serve_file_path) = &self.link_file_path {
            result["serve_file_path"] = serde_json::json!(serve_file_path);
        }

        if !self.taxonomies.tags.is_empty() {
            result["tags"] = serde_json::json!(self.taxonomies.tags);
        }

        result
    }

    pub fn card_jinja_context(&self) -> minijinja::Value {
        let external = &self.taxonomies.extern_info;
        let description = match &self.taxonomies.description {
            Some(d) => d,
            None => "",
        };
        match external {
            None => {
                context! {
                    description => description,
                    link => self.link_file_path,
                    title => self.taxonomies.title,
                    date => self.taxonomies.date,
                    tags => transform_tags(&self.taxonomies.tags),
                }
            }
            Some(value) => {
                context! {
                    description => description,
                    link => self.link_file_path,
                    title => self.taxonomies.title,
                    date => self.taxonomies.date,
                    extern_link => value.link,
                    tags => transform_tags(&self.taxonomies.tags),
                }
            }
        }
    }

    pub fn jinja_context(&self) -> minijinja::Value {
        let extern_info = &self.taxonomies.extern_info;
        let map = &self.taxonomies.map;

        // Start with a HashMap to build the context
        let mut context_data = HashMap::new();

        // Add common fields
        context_data.insert("content", Value::from(self.post_html.clone()));
        context_data.insert(
            "page_name",
            Value::from(
                self.source_file_path
                    .clone()
                    .unwrap()
                    .file_stem()
                    .and_then(|os_str| os_str.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| String::from(""))
                    + ".md",
            ),
        );
        context_data.insert("title", Value::from(self.taxonomies.title.clone()));

        // Add extern_info fields if present
        if let Some(value) = extern_info {
            context_data.insert("extern_link", Value::from(value.link.clone()));
            context_data.insert("extern_author", Value::from(value.author.clone()));
            context_data.insert("extern_date", Value::from(value.date.clone().to_string()));
            context_data.insert("extern_title", Value::from(value.title.clone()));
        }

        // Add map fields if present
        if let Some(map_data) = map {
            context_data.insert("map", Value::from_serialize(map_data));
        }

        context! {
            ..context_data
        }
    }

    pub fn uuid(&self) -> String {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, format!("{:?}", self).as_bytes()).to_string()
    }
}

/// Reads a Markdown file, extracts the YAML front matter, and converts the Markdown to HTML.
pub fn parse_content(
    input_file: &Path,
) -> Result<(Taxonomies, String), Box<dyn std::error::Error>> {
    let file_text = fs::read_to_string(input_file)?;

    let re = Regex::new(r"---\s*")?;
    let matches: Vec<&str> = re.split(file_text.trim()).collect();

    let (taxonomies, after) = if matches.len() > 1 {
        let between = matches[1].trim();
        let after = matches.get(2).map(|s| s.trim()).unwrap_or("");
        let taxonomies: Taxonomies = serde_yaml::from_str(between)?;
        (taxonomies, after)
    } else {
        panic!("Parsing content file did not work. There seems to be too many taxonomies sections.")
    };

    // Convert Markdown to HTML
    let html_output = markdown_to_html(after);

    Ok((taxonomies, html_output))
}

pub fn create_posts(config: &ConfigFile) -> Vec<Post> {
    let entries: Vec<_> = fs::read_dir(config.content.clone())
        .expect("Cannot read directory!")
        .collect();

    let mut posts: Vec<Post> = Vec::with_capacity(entries.len());
    for file in entries {
        let entry = file.expect("Error when looking at file!");
        let path = entry.path();
        posts.push(Post::new(&path, &config.root));
    }
    posts
}
