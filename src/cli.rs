use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to a YAML config file. If not provided, the YAML will be taken from an environment variable (`CONTAINER_INIT_CONF`) instead of a file
    #[arg(short = 'c', long)]
    pub config: Option<String>,
}
