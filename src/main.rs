use colored::ColoredString;
use clap::derive;
use log::error;

/// Args for building
#[derive(Debug, Args)]
pub struct BuildGroup {
    /// Input file
    #[arg(short, long, required = true)]
    pub source: String,

    /// Optional output file (only meaningful with --input-path)
    #[arg(short, long)]
    pub output_path: Option<String>,
}

/// Args for running
#[derive(Debug, Args)]
pub struct RunGroup {
    /// Launch parameters for Halcyon program
    #[arg(short, long, num_args = 0..)]
    pub parameters: Option<Vec<String>>,
}

/// Args for docs
#[derive(Debug, Args)]
pub struct DocGroup {
    /// File to write docs to
    #[arg(short, long, default_value = "./docs.md")]
    pub doc_file: Option<String>,
}

/// Args for initializing
#[derive(Debug, Args)]
pub struct InitGroup {
    /// Output path
    #[arg(short, long, default_value = "./a.wasm")]
    pub output_path: Option<String>,
    /// Optional flag to skip git init
    #[arg(short, long, action)]
    pub no_git: bool,
}

/// Args for adding a dependency
#[derive(Debug, Args)]
pub struct AddGroup {
    /// Arg URL
    #[arg(short, long, required = true)]
    pub url: String,
    /// Config file path
    #[arg(short, long, default_value = "./Config.toml")]
    pub config_file: String,
}

/// Subcommands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Validate the project
    Check(BuildGroup),
    /// Compile and link the project
    Build(BuildGroup),
    /// Compile, link, and execute the project
    Run(RunGroup),
    /// Initialize a new Halcyon project in the current directory
    Init(InitGroup),
    /// Create documentation based off line comments
    Doc(DocGroup),
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

// checks if the infiles in a given config are valid
pub fn check_valid(config : &Config) -> std::result::Result<(), colored::ColoredString> {
    // TODO: More checks?
    // does each infile compile?
    for infile in &config.build.infiles {
        info("Check",&format!("Checking: {}", infile.blue()));
        let path = std::path::PathBuf::from(infile);
        match path.extension().unwrap().to_str() {
            Some("hc") => {
                /*let file_as_string = std::fs::read_to_string(std::path::PathBuf::from(infile))
                    .map_err(|e| format!("{} {} {}", "Config error:".red(),  e.to_string(), &infile.red()))?; //this shouldnt trigger because the config should be valid, but just in case
                let gag = Gag::stdout().unwrap();
                let compilation_result = compile(&file_as_string);
                drop(gag);
                match compilation_result {
                    std::result::Result::Err(err) => return std::result::Result::Err(format!("{}\n{}",infile.clone().red(), &err).into()),
                    _ => {info("Check", &format!("Success Checking {}", infile.blue()));},
                }*/
            }
            Some("wasm") => {// maybe something here later... idk 
                },
            _ => error(&format!("Check: Invalid infile type detected \"{}\"", infile)),
        }
        
        
    }
    Ok(())

} 

fn gup_main() -> Result<(), ColoredString> {
    Ok(())
}

fn main() {
    human_panic::setup_panic!();
    match gup_main() {
        Ok(()) => (),
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    }
}
