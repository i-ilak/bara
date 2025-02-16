use crate::config::ConfigFile;
use crate::map::MapData;
use chrono::NaiveDate;
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

use pulldown_cmark::{html, Parser};
use regex::Regex;
use serde_yaml;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Taxonomies {
    pub title: String,
    pub description: String,
    pub date: NaiveDate,
    pub map: Option<MapData>,
    pub tags: Vec<String>,
    pub external_link: Option<String>,
    pub downloaded_link: Option<String>,
}

#[derive(Debug)]
pub struct Post {
    pub taxonomies: Taxonomies,
    pub post_html: String,
    pub source_file_path: Option<PathBuf>,
    pub serve_file_path: Option<PathBuf>,
}

impl Post {
    pub fn new(path: PathBuf, root: String, working_dir: PathBuf) -> Self {
        let (taxonomies, post_html) = parse_content(&path)
            .expect("Could not parse taxonomies and content. Double check markdown!");

        let source_file_path = Some(path.clone());
        let mut file_path = path.to_string_lossy().replace("/content", "");
        file_path = file_path.replace(".md", ".html").replace(&root, "");
        let serve_file_path = Some(PathBuf::from_str(&file_path).unwrap());

        let post = Post {
            taxonomies,
            post_html,
            source_file_path,
            serve_file_path,
        };

        post
    }

    pub fn serialize(&self) -> serde_json::Value {
        let mut result = serde_json::json!({
            "title": self.taxonomies.title,
            "date": self.taxonomies.date.format("%Y-%m-%d").to_string(),
            "description": self.taxonomies.description,
        });

        if let Some(serve_file_path) = &self.serve_file_path {
            result["serve_file_path"] = serde_json::json!(serve_file_path);
        }

        if !self.taxonomies.tags.is_empty() {
            result["tags"] = serde_json::json!(self.taxonomies.tags);
        }

        result
    }

    pub fn card_jinja_context(&self) -> minijinja::Value {
        context! {
            post_description => self.taxonomies.description,
            post_link => self.serve_file_path,
            post_title => self.taxonomies.title,
            post_date => self.taxonomies.date,
            external_link => self.taxonomies.external_link,
        }
    }

    pub fn jinja_context(&self) -> minijinja::Value {
        context! {
           content => self.post_html,
           page_name => self.source_file_path
                .clone()
                .unwrap()
                .file_stem()
                .and_then(|os_str| os_str.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| String::from("")) + ".md",
        }
    }

    pub fn uuid(&self) -> String {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, format!("{:?}", self).as_bytes()).to_string()
    }
}

/// Converts Markdown content to HTML with syntax highlighting.
fn markdown_to_html(markdown: &str) -> String {
    // Create a Markdown parser
    let parser = Parser::new(markdown);

    // Convert Markdown to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

/// Reads a Markdown file, extracts the YAML front matter, and converts the Markdown to HTML.
fn parse_content(input_file: &Path) -> Result<(Taxonomies, String), Box<dyn std::error::Error>> {
    let file_text = fs::read_to_string(input_file)?;

    let re = Regex::new(r"---\s*")?;
    let matches: Vec<&str> = re.split(&file_text.trim()).collect();

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

pub fn create_posts(config: &ConfigFile, working_dir: &Path) -> Vec<Post> {
    let entries: Vec<_> = fs::read_dir(config.content.clone())
        .expect("Cannot read directory!")
        .collect();

    let mut posts: Vec<Post> = Vec::with_capacity(entries.len() as usize);
    for file in entries {
        let entry = file.expect("Error when looking at file!");
        let path = entry.path();
        posts.push(Post::new(
            path,
            config.root.clone(),
            working_dir.to_path_buf(),
        ));
    }
    posts
}
