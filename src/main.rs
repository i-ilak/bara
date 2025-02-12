mod content;
mod map;

use content::{Post, Taxonomies};
use pulldown_cmark::{html, Parser};
use regex::Regex;
use serde_yaml;
use std::fs;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

/// Converts Markdown content to HTML with syntax highlighting.
fn markdown_to_html(markdown: &str) -> String {
    // Create a Markdown parser
    let parser = Parser::new(markdown);

    // Convert Markdown to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Optional: Add syntax highlighting using syntect
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let syntax = syntax_set.find_syntax_by_extension("rs").unwrap(); // Example: Rust syntax
    let mut h = HighlightLines::new(syntax, &theme_set.themes["base16-ocean.dark"]);

    // Highlight code blocks (this is a simplified example)
    for line in markdown.lines() {
        if line.trim().starts_with("```") {
            // Handle code blocks
            let ranges: Vec<(Style, &str)> = h.highlight(line, &syntax_set);
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            html_output.push_str(&escaped);
        } else {
            html_output.push_str(line);
            html_output.push('\n');
        }
    }

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_file = Path::new("/Users/iilak/prg/internal/raw_blog/content/posts/create_map.md");
    let (taxonomies, html_output) = parse_content(input_file)?;
    let post = Post::new(taxonomies, html_output);

    println!("{}", post.serialize());

    let input_file =
        Path::new("/Users/iilak/prg/internal/raw_blog/content/posts/another_example.md");
    let (taxonomies, html_output) = parse_content(input_file)?;
    let post = Post::new(taxonomies, html_output);

    println!("{:?}", post);

    Ok(())
}
