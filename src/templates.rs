use crate::config::ConfigFile;
use crate::content::{create_posts, transform_tags, Post};
use crate::markdown_parsing::markdown_to_html;
use crate::projects::create_projects;
use crate::util::write_file;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Tz;
use minijinja::{context, Environment, Template};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use versions::Version;
use walkdir::WalkDir;

#[derive(Serialize)]
struct RssItem {
    title: String,
    link: String,
    description: String,
    pub_date: String,
    tags: Vec<String>,
}

#[derive(Serialize)]
struct TemplateContext {
    site_title: String,
    site_url: String,
    site_description: String,
    items: Vec<RssItem>,
}

fn generate_rss(env: &Environment, posts: &[Post], working_dir: &Path) {
    let gmt = "GMT".parse::<Tz>().unwrap();
    let mut items = Vec::with_capacity(posts.len());

    for post in posts {
        let naive_datetime = NaiveDateTime::new(
            post.taxonomies.date,
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
        let pub_date = gmt.from_utc_datetime(&naive_datetime).to_rfc2822();

        // Get relative URL path
        let link = post
            .link_file_path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .trim_start_matches("/");

        items.push(RssItem {
            title: post.taxonomies.title.clone(),
            link: format!("BASEPATH/{}", link),
            description: post.taxonomies.description.clone().unwrap_or(String::new()),
            pub_date,
            tags: post.taxonomies.tags.clone(),
        });
    }

    // Create template context
    let ctx = TemplateContext {
        site_title: String::from("ilak.ch"),
        site_url: String::from("https://www.ilak.ch"),
        site_description: String::from("Personal blog"),
        items,
    };

    // Render template
    let output = env
        .get_template("rss_feed.html.jinja2")
        .unwrap()
        .render(ctx)
        .unwrap();

    // Write to file
    let rss_path = working_dir.join("rss.xml");
    std::fs::write(rss_path, output).unwrap();
}

fn render_and_write<T>(
    template: &Template,
    item: &T,
    context_fn: impl Fn(&T) -> minijinja::Value,
    output_path: Option<&Path>,
) -> Result<String, Box<dyn std::error::Error>>
where
    T: ?Sized,
{
    let rendered = template
        .render(context_fn(item))
        .map_err(|e| format!("Could not render template: {}", e))?;

    if let Some(path) = output_path {
        write_file(path, rendered.clone());
    }

    Ok(rendered)
}

fn get_all_versions(time_machine_dir: &Path) -> Vec<Version> {
    let mut versions = Vec::new();

    if let Ok(entries) = fs::read_dir(time_machine_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    if let Some(dir_name_str) = dir_name.to_str() {
                        versions.push(Version::new(dir_name_str).unwrap());
                    }
                }
            }
        }
    }

    versions.sort();
    versions
}

fn create_database(posts: &Vec<Post>, working_dir: &Path, config: &ConfigFile) {
    let mut database = json!({
        "posts": {},
        "tags": {},
    });

    for post in posts {
        if post.taxonomies.tags.is_empty() {
            continue;
        }

        let post_entry = json!({
            "metadata": post.serialize(),
            "html": "<div>Fix the code in bara, you were too lazy to fix...</div>",
        });

        database["posts"][post.uuid()] = post_entry;

        for tag in &post.taxonomies.tags {
            if database["tags"].get(tag).is_none() {
                database["tags"][tag] = json!([]);
            }
            database["tags"][tag]
                .as_array_mut()
                .unwrap()
                .push(json!(post.uuid()));
        }
    }

    // Add versions to the database
    let versions: Vec<Value> = get_all_versions(&config.time_machine.clone())
        .iter()
        .rev()
        .map(|v| json!(format!("{}", v)))
        .collect();
    database["versions"] = json!(versions);

    // Write the database to a file
    let mut database_filepath = working_dir.to_path_buf();
    database_filepath.push("database.json");
    write_file(
        &database_filepath,
        serde_json::to_string_pretty(&database).unwrap(),
    );
}

#[derive(Debug, Deserialize)]
pub struct Links {
    pub title: String,
    pub description: String,
    pub author: String,
    pub date_published: NaiveDate,
    pub date_linked: NaiveDate,
    pub external_link: String,
    pub downloaded_link: String,
    pub tags: Vec<String>,
}

impl Links {
    pub fn card_jinja_context(&self) -> minijinja::Value {
        context! {
            description => self.description,
            title => self.title,
            date_published => self.date_published,
            date_linked => self.date_linked,
            author => self.author,
            link => self.external_link,
            downloaded_link => self.downloaded_link,
            tags => transform_tags(&self.tags),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LinkFile {
    pub links: Vec<Links>,
}

pub fn create_links(content_folder: &Path) -> Vec<Links> {
    let mut link_file = content_folder
        .parent()
        .expect("Parent does not exist!")
        .join("links");
    link_file = link_file.join("links.yml");

    let file_content = fs::read_to_string(link_file).unwrap();

    let links: LinkFile = serde_yaml::from_str(&file_content)
        .expect("Deserializing links-yaml file was not possible!");
    links.links
}

fn create_posts_and_projects(config: &ConfigFile, working_dir: &Path, env: &Environment) {
    let posts = create_posts(config);
    let links = create_links(&config.content);
    let projects = create_projects(&config.projects);

    let card_template = env
        .get_template("card.html.jinja2")
        .expect("Template does not exist!");

    let post_template = env
        .get_template("post/post.html.jinja2")
        .expect("Template does not exist!");

    let post_and_project_overview_template = env
        .get_template("post_overview.html.jinja2")
        .expect("Could not load template: post_overview.html.jinja2");

    let sitemap_template = env
        .get_template("sitemap.xml.jinja2")
        .expect("Template does not exist!");

    let mut cards_posts_html: BTreeMap<&NaiveDate, String> = Default::default();
    let mut cards_projects_html: Vec<String> = Vec::with_capacity(projects.len());
    let mut cards_links_html: Vec<String> = Vec::with_capacity(projects.len());

    // Process posts
    for post in posts.iter() {
        let file_path = working_dir.join(post.serve_file_path.clone().unwrap());
        let card_html = render_and_write(
            &card_template,
            post,
            |p| p.card_jinja_context(),
            Some(&file_path),
        )
        .unwrap();
        cards_posts_html.insert(&post.taxonomies.date, card_html);

        render_and_write(
            &post_template,
            post,
            |p| p.jinja_context(),
            Some(&file_path),
        )
        .unwrap();
    }
    create_database(&posts, working_dir, config);

    // Process projects
    for project in projects.iter() {
        let card_html =
            render_and_write(&card_template, project, |p| p.card_jinja_context(), None).unwrap();
        cards_projects_html.push(card_html);
    }

    // Process links
    for link in links.iter() {
        let card_html =
            render_and_write(&card_template, link, |p| p.card_jinja_context(), None).unwrap();
        let link_folder = config.root.join("content");

        if !working_dir.join("links").exists() {
            fs::create_dir(working_dir.join("links")).unwrap();
        }
        fs::copy(
            link_folder.join(&link.downloaded_link[1..]),
            working_dir.join(&link.downloaded_link[1..]),
        )
        .unwrap();
        cards_links_html.push(card_html);
    }
    cards_links_html.reverse();

    // Write post overview
    let posts_index_path = working_dir.join("posts/index.html");
    let mut ordered_cards = cards_posts_html.into_values().collect::<Vec<_>>();
    ordered_cards.reverse();
    write_file(
        &posts_index_path,
        post_and_project_overview_template
            .render(context! {
                posts => ordered_cards,
                current_page => "posts"
            })
            .unwrap(),
    );

    // Write project overview
    let projects_index_path = working_dir.join("projects/index.html");
    write_file(
        &projects_index_path,
        post_and_project_overview_template
            .render(context! {
                posts => cards_projects_html,
                project => true,
                current_page => "projects"
            })
            .unwrap(),
    );
    // Write project overview
    let links_index_path = working_dir.join("links/index.html");
    write_file(
        &links_index_path,
        post_and_project_overview_template
            .render(context! {
                posts => cards_links_html,
                current_page => "links"
            })
            .unwrap(),
    );

    generate_rss(env, &posts, working_dir);

    let sitemap = SiteMap::new(&posts);
    write_file(
        working_dir.join("sitemap.xml").as_path(),
        sitemap_template
            .render(sitemap.jinja_context())
            .expect("Error processing template for sitemap!"),
    );
}

#[derive(Serialize)]
struct SiteMapEntry {
    loc: PathBuf,
    lastmod: NaiveDate,
    #[serde(serialize_with = "serialize_two_decimals")]
    priority: f32,
}

fn serialize_two_decimals<S>(value: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let formatted = format!("{:.2}", value);
    serializer.serialize_str(&formatted)
}

#[derive(Serialize)]
struct SiteMap {
    entries: Vec<SiteMapEntry>,
}

impl SiteMap {
    fn new(posts: &[Post]) -> SiteMap {
        let mut entries: Vec<SiteMapEntry> = Vec::with_capacity(posts.len());

        for post in posts.iter() {
            entries.push(SiteMapEntry {
                loc: post.serve_file_path.clone().unwrap(),
                lastmod: post.taxonomies.date,
                priority: 0.9,
            });
        }

        SiteMap { entries }
    }

    fn jinja_context(&self) -> minijinja::Value {
        context! {
            sitemap_entries => self.entries
        }
    }
}

pub fn load_templates(config: &ConfigFile) -> Environment {
    let mut env = Environment::new();
    for entry in WalkDir::new(config.templates.clone())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("jinja2") {
            let template_source = fs::read_to_string(path).expect("Cannot read template!");
            let template_name = path
                .strip_prefix(&config.templates)
                .expect("Could not strip the prefix!")
                .to_string_lossy()
                .into_owned();

            env.add_template_owned(template_name, template_source)
                .expect("Could not add template!");
        }
    }
    env
}

fn write_landing(config: &ConfigFile, working_dir: &Path, env: &Environment<'_>) {
    let landing_page_loc = working_dir.join("index.html");
    let landing_page_markdown = config.root.join("content").join("landing.md");
    let content = markdown_to_html(&fs::read_to_string(landing_page_markdown).unwrap());
    write_file(
        &landing_page_loc,
        env.get_template("landing.html.jinja2")
            .expect("Cannot get landing page template!")
            .render(context! {
                markdown => content,
                home => true
            })
            .unwrap(),
    );
}

fn write_privacy_policy(working_dir: &Path, env: &Environment<'_>) {
    let privacy_file_loc = working_dir.join("privacy_policy.html");
    write_file(
        &privacy_file_loc,
        env.get_template("privacy_policy.html.jinja2")
            .expect("Cannot create privacy policy!")
            .render(context! {})
            .unwrap(),
    )
}

fn write_terms(working_dir: &Path, env: &Environment<'_>) {
    let terms_file_loc = working_dir.join("terms.html");
    write_file(
        &terms_file_loc,
        env.get_template("terms.html.jinja2")
            .expect("Cannot create terms!")
            .render(context! {})
            .unwrap(),
    )
}

pub fn process_jinja(config: &ConfigFile, working_dir: &Path) {
    let env = load_templates(config);
    create_posts_and_projects(config, working_dir, &env);
    write_landing(config, working_dir, &env);
    write_privacy_policy(working_dir, &env);
    write_terms(working_dir, &env);
}
