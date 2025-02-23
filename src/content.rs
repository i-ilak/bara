use crate::config::ConfigFile;
use crate::map::MapData;
use minijinja::context;
use minijinja::Value;
use pulldown_cmark::{html, CodeBlockKind, Event, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use two_face::re_exports::syntect;
use uuid::Uuid;

use chrono::NaiveDate;
use std::path::Path;

use regex::Regex;
use serde_yaml;
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
    pub description: String,
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
}

impl Post {
    pub fn new(path: &Path, root: &Path) -> Self {
        let (taxonomies, post_html) = parse_content(&path)
            .expect("Could not parse taxonomies and content. Double check markdown!");

        let stripped_path = path.strip_prefix(root.join("content")).unwrap();

        let post = Post {
            taxonomies,
            post_html,
            source_file_path: Some(path.to_path_buf()),
            serve_file_path: Some(stripped_path.with_extension("html")),
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
        let external = &self.taxonomies.extern_info;
        match external {
            None => {
                context! {
                    post_description => self.taxonomies.description,
                    post_link => self.serve_file_path,
                    post_title => self.taxonomies.title,
                    post_date => self.taxonomies.date,
                }
            }
            Some(value) => {
                context! {
                    post_description => self.taxonomies.description,
                    post_link => self.serve_file_path,
                    post_title => self.taxonomies.title,
                    post_date => self.taxonomies.date,
                    extern_link => value.link
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

/// Converts Markdown content to HTML with syntax highlighting.
pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);

    // Load syntax set and theme for syntax highlighting
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = two_face::theme::extra();
    let theme = theme_set.get(two_face::theme::EmbeddedThemeName::InspiredGithub);

    let mut html_output = String::new();
    let mut in_code_block = false;
    let mut current_lang = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                current_lang = match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => "plaintext".to_string(),
                };
                html_output.push_str(r#"<div class="code-highlight">"#);
                html_output.push_str(r#"<div class="line-numbers">"#);
                html_output.push_str("<pre><code>");
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                html_output.push_str("</code></pre>");
                // Close the line numbers wrapper
                html_output.push_str("</div>");
                // Close the wrapper div
                html_output.push_str("</div>");
            }
            Event::Text(text) => {
                if in_code_block {
                    // Syntax highlighting for code blocks
                    let syntax = syntax_set
                        .find_syntax_by_token(&current_lang)
                        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                    let html =
                        highlighted_html_for_string(&text, &syntax_set, syntax, theme).unwrap();
                    let lines: Vec<&str> = html.lines().collect();
                    let numbered_html = lines
                        .iter()
                        .enumerate()
                        .map(|(i, line)| {
                            // Skip line numbers for the first and last lines if they are empty
                            if i == 0 || i == lines.len() - 1 {
                                line.to_string()
                            } else {
                                format!("<span class=\"line-number\">{:<3}</span>{}", i, line)
                            }
                        })
                        .collect::<Vec<String>>()
                        .join("\n");
                    html_output.push_str(&numbered_html);
                } else {
                    // Regular text
                    html_output.push_str(&text);
                }
            }
            _ => {
                // Handle other Markdown events
                html::push_html(&mut html_output, std::iter::once(event));
            }
        }
    }

    html_output
}

/// Reads a Markdown file, extracts the YAML front matter, and converts the Markdown to HTML.
pub fn parse_content(
    input_file: &Path,
) -> Result<(Taxonomies, String), Box<dyn std::error::Error>> {
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

pub fn create_posts(config: &ConfigFile) -> Vec<Post> {
    let entries: Vec<_> = fs::read_dir(config.content.clone())
        .expect("Cannot read directory!")
        .collect();

    let mut posts: Vec<Post> = Vec::with_capacity(entries.len() as usize);
    for file in entries {
        let entry = file.expect("Error when looking at file!");
        let path = entry.path();
        posts.push(Post::new(&path, &config.root));
    }
    posts
}
