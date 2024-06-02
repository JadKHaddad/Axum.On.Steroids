use clap::Parser;

#[derive(Parser)]
#[command(author, about, version)]
pub struct CliArgs {
    /// Path to the configuration file.
    #[clap(long, env = "CONFIG_FILE", default_value = "config.yaml")]
    pub config_file: String,
}
