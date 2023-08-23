use {clap::Parser, std::path::Path};

/// The lightweight link manager
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// The config path
    #[arg(short, long, default_value_t = String::from("config.toml"))]
    config: String,
}

impl Cli {
    pub fn conf_path(&self) -> &Path {
        self.config.as_ref()
    }
}
