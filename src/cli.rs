use clap::*;
use clap_verbosity_flag::*;
/// Args for initializing
#[derive(Debug, Args)]
pub struct InitGroup {
    /// Optional flag to skip git init
    #[arg(short, long, action)]
    pub no_git: bool,
}

/// Args for running
#[derive(Debug, Args)]
pub struct RunGroup {
    /// Arg URL
    #[arg(short, long, num_args = 0..)]
    pub paramaters: Vec<String>,
}

/// Args for adding a dependency

#[derive(Debug, Args)]
#[clap(group = ArgGroup::new("input")
    .required(true))]
pub struct AddGroup {
    /// Arg URL
    #[arg(short, long, group = "input")]
    pub url: Option<String>,
    /// Arg path
    #[arg(short, long, group = "input")]
    pub path: Option<String>,
}

/// Subcommands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Validate the project
    Check,
    /// Compile and link the project
    Build,
    /// Compile, link, and execute the project
    Run(RunGroup),
    /// Initialize a new Halcyon project in the current directory
    Init(InitGroup),
    /// Create documentation based off line comments
    Doc,
    /// Add a dependency to your project
    Add(AddGroup),
    /// Update dependencies to the most recent versions
    Update,
    Version,
}

#[derive(Parser, Debug)]
#[command(name = "gup", about = "Halcyon Package Manager")]
pub struct CmdArgs {
    #[command(subcommand)]
    pub command: Commands,
    /// Verbosity
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}