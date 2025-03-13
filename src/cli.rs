use clap::Parser;

#[derive(Parser)]
#[clap(
    author,
    version,
    about = "Static site generator for my personal website."
)]
pub struct Cli {
    #[clap(short, long)]
    pub archive: bool,
    #[clap(short, long)]
    pub config: Option<String>,
    #[clap(short, long)]
    pub watch: bool,
}
