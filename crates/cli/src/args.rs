use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dbml")]
#[command(about = "DB Modeling CLI - Visualizador e editor interativo de banco de dados")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Caminho para o arquivo .dbml (quando executado diretamente como `dbml schema.dbml`)
    #[arg(value_name = "ARQUIVO")]
    pub file: Option<PathBuf>,

    /// Porta do servidor web local
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Endereço de bind
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Não abre o navegador automaticamente após iniciar
    #[arg(long)]
    pub no_open: bool,

    /// Desativa o monitoramento de alterações externas no arquivo
    #[arg(long)]
    pub no_watch: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Inicia o servidor local e abre o canvas visual no navegador
    Run {
        /// Caminho para o arquivo .dbml
        #[arg(value_name = "ARQUIVO")]
        file: PathBuf,

        /// Porta do servidor web local
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Endereço de bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Não abre o navegador automaticamente
        #[arg(long)]
        no_open: bool,

        /// Desativa o monitoramento de alterações externas
        #[arg(long)]
        no_watch: bool,
    },
    /// Alias para iniciar o servidor web
    Serve {
        /// Caminho para o arquivo .dbml
        #[arg(value_name = "ARQUIVO")]
        file: PathBuf,

        /// Porta do servidor web local
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Endereço de bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Não abre o navegador automaticamente
        #[arg(long)]
        no_open: bool,

        /// Desativa o monitoramento de alterações externas
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
                    anyhow::bail!("Nenhum arquivo .dbml informado. Uso: dbml run <arquivo.dbml>");
                }
            }
        }
    }
}
