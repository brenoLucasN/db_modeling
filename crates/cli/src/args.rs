use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dbml")]
#[command(about = "DB Modeling CLI - Interactive database schema viewer and editor")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to the .dbml file (when executed directly as `dbml schema.dbml`)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Local web server port
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Bind address
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Do not open browser automatically on startup
    #[arg(long)]
    pub no_open: bool,

    /// Disable watching for external file changes
    #[arg(long)]
    pub no_watch: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the local server and open the visual canvas in the browser
    Run {
        /// Path to the .dbml file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Local web server port
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Bind address
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Do not open browser automatically
        #[arg(long)]
        no_open: bool,

        /// Disable watching for external file changes
        #[arg(long)]
        no_watch: bool,
    },
    /// Alias to start the web server
    Serve {
        /// Path to the .dbml file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Local web server port
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Bind address
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Do not open browser automatically
        #[arg(long)]
        no_open: bool,

        /// Disable watching for external file changes
        #[arg(long)]
        no_watch: bool,
    },
}

pub struct RunConfig {
    pub file: PathBuf,
    pub port: u16,
    pub host: String,
    pub no_open: bool,
    pub no_watch: bool,
}

impl Cli {
    pub fn resolve_config(self) -> anyhow::Result<RunConfig> {
        match self.command {
            Some(Commands::Run {
                file,
                port,
                host,
                no_open,
                no_watch,
            })
            | Some(Commands::Serve {
                file,
                port,
                host,
                no_open,
                no_watch,
            }) => Ok(RunConfig {
                file,
                port,
                host,
                no_open,
                no_watch,
            }),
            None => {
                if let Some(file) = self.file {
                    Ok(RunConfig {
                        file,
                        port: self.port,
                        host: self.host,
                        no_open: self.no_open,
                        no_watch: self.no_watch,
                    })
                } else {
                    anyhow::bail!("No .dbml file specified. Usage: dbml run <file.dbml>");
                }
            }
        }
    }
}
