use crate::config::ConfigFile;
use crate::content::create_posts;
use crate::projects::create_projects;
use crate::util::write_file;
use minijinja::{context, Environment, Template};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn render_and_write<T>(
    template: &Template,
    item: &T,
    context_fn: impl Fn(&T) -> minijinja::Value,
    output_path: Option<PathBuf>,
) -> Result<String, Box<dyn std::error::Error>>
where
    T: ?Sized,
{
    let rendered = template
        .render(&context_fn(item))
        .map_err(|e| format!("Could not render template: {}", e))?;

    if let Some(path) = output_path {
        write_file(path, rendered.clone());
    }

    Ok(rendered)
}

fn create_posts_and_projects(config: &ConfigFile, working_dir: PathBuf, env: &Environment) {
    let posts = create_posts(config.clone(), working_dir.clone());
    let projects = create_projects(&config.projects);

    let card_post_template = env
        .get_template("cards/post.jinja2")
        .expect("Template does not exist!");

    let post_template = env
        .get_template("post.jinja2")
        .expect("Template does not exist!");

    let post_overview_template = env
        .get_template("post_overview.jinja2")
        .expect("Could not load template: post_overview.jinja2");

    let mut cards_posts_html = Vec::with_capacity(posts.len());
    let mut cards_projects_html = Vec::with_capacity(projects.len());

    // Process posts
    for post in posts.iter() {
        let card_html =
            render_and_write(&card_post_template, post, |p| p.card_jinja_context(), None).unwrap();
        cards_posts_html.push(card_html);

        render_and_write(
            &post_template,
            post,
            |p| p.jinja_context(),
            post.serve_file_path.clone(),
        )
        .unwrap();
    }

    // Process projects
    for project in projects.iter() {
        let card_html = render_and_write(
            &card_post_template,
            project,
            |p| p.card_jinja_context(),
            None,
        )
        .unwrap();
        cards_projects_html.push(card_html);
    }

    // Write post overview
    let mut posts_index_path = working_dir.clone();
    posts_index_path.push("posts/index.html");
    write_file(
        posts_index_path,
        post_overview_template
            .render(context! {
                overview_title => "Posts",
                posts => cards_posts_html,
            })
            .unwrap(),
    );

    // Write project overview
    let mut projects_index_path = working_dir.clone();
    projects_index_path.push("projects/index.html");
    write_file(
        projects_index_path,
        post_overview_template
            .render(context! {
                overview_title => "Projects",
                posts => cards_projects_html,
                project => "",
            })
            .unwrap(),
    );
}

fn load_templates(config: &ConfigFile, env: &mut Environment) {
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
}

pub fn process_jinja(config: ConfigFile, working_dir: PathBuf) {
    let mut env = Environment::new();
    load_templates(&config, &mut env);
    create_posts_and_projects(&config, working_dir.clone(), &env);

    let mut privacy_file_loc = working_dir.clone();
    privacy_file_loc.push("privacy_policy.html");
    write_file(
        privacy_file_loc,
        env.get_template("privacy_policy.jinja2")
            .expect("Cannot create privacy policy!")
            .render(context! {})
            .unwrap(),
    )
}
