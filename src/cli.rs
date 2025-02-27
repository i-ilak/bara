use clap::Parser;

#[derive(Parser)]
#[clap(
    name = "bara",
    version = "0.1",
    author = "Ivan Ilak <ivan.ilak@hotmail.com>",
    about = "Static site generator for my personal website."
)]
pub struct Cli {
    #[clap(short, long)]
    pub archive: bool,
    #[clap(short, long)]
    pub config: String,
    #[clap(short, long)]
    pub watch: bool,
}
