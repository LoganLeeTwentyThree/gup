use colored::{Colorize,ColoredString};
use clap::*;
use log::error;
use url::Url;
use std::env::home_dir;
use std::path::PathBuf;

mod cli;
use crate::cli::*;
mod config;
use crate::config::*;
mod pdm;
use pdm::*;
mod logging;
use logging::*;
mod build;
use build::*;
mod parse;

const CONFIG_PATH : &str = "./Config.toml";
const OUTPUT_PATH : &str = "./a.wasm";
const DOCS_PATH : &str = "./docs.md";


fn gup_main() -> Result<(), ColoredString> {
    let args = CmdArgs::parse();
    env_logger::Builder::new()
        .filter_module("gup",args.verbose.log_level_filter())
        .init();
    match args.command {
        Commands::Check => {
            let cfg = create_config_from_path(&PathBuf::from(CONFIG_PATH))?;
            check_valid(&cfg)?;
        },
        Commands::Build => {
            let cfg = create_config_from_path(&PathBuf::from(CONFIG_PATH))?;
            build(&cfg)?;
        }
        Commands::Run(run_group) => {
            let cfg = create_config_from_path(&PathBuf::from(CONFIG_PATH))?;
            run(&cfg, run_group.paramaters)?;
        },
        Commands::Init(init_group) => {
            // Initialize a new halcyon project
            // check if config already exists
            match std::fs::exists(CONFIG_PATH) {
                Ok(true) => {
                    warn("Init",&format!("\"{CONFIG_PATH}\" already exists in this directory. Continue? (y/N)"));
                    loop {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).expect("Failed to read line");
                        match input.to_lowercase().trim() {
                            "y" => {break;}
                            "n" => {return Ok(())}
                            "" => {break;}
                            _ => {warn("Init","Invalid input. Try again."); }
                        }
                    }
                },
                Ok(false) => {}
                Err(e) => return Err(format!("{}: {}", "Init error".red(), e).into())
            } 

            println!("Input Project Name: ");
            let mut proj_name = String::new();
            std::io::stdin().read_line(&mut proj_name).expect("Failed to read line");
            
            
            // create a config from input/defaults
            let mut cfg = create_config(
                vec!["./main.hc".into()], 
                OUTPUT_PATH.into(), 
                Some(DOCS_PATH.into()),
                None)?;
                
            // add package
            cfg.package =  Some(
                Package {
                name: proj_name.trim().into(),
                version: "0.1.0".into(),
            });
            
            // write each infile as a .hc module
            for arg in cfg.build.infiles.clone() {
                let path = std::path::PathBuf::from(&arg);
                let module_name = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(name) => name,
                    None => return Err("Invalid or missing filename for module".red().into()),
                };
                let content = format!("module {} =\n(* Your code here! *)\nend", module_name);
                std::fs::write(path, content)
                    .map_err(|e| e.to_string().red())?;
            }
            // write the config
            write_config(&cfg, crate::CONFIG_PATH.into())?;

            if !init_group.no_git {
                // make a git repo
                let repo = git2::Repository::init(".")
                    .map_err(|e| e.to_string())?;

                let content = "# Halcyon build artifacts\n*.wasm";
                std::fs::write(".gitignore", content).map_err(|e| e.to_string())?;
                
                info("Init", &format!("Initialized empty Git repository at {:?}", repo.path()));
            }
            success("Successfully initialized halcyon project");
        },
        Commands::Doc => {
            let cfg = create_config_from_path(&PathBuf::from(CONFIG_PATH))?;
            parse::create_docs(cfg)?;
            success("Docs created");
        },
        Commands::Add(add_group) => {
            match (add_group.path, add_group.url){
                (Some(path), None) =>{
                    let cfg = create_config_from_path(&PathBuf::from(path.clone()).join(CONFIG_PATH))?;
                    let pack = cfg.package.expect("Dpendency has invalid config!");
                    let package_name = format!("{}-{}", pack.name.chars().filter(|c| !c.is_whitespace()).collect::<String>(), pack.version );
                    let new_dir_name : PathBuf = get_hc_filepath()?.join::<PathBuf>(package_name.into());
                    if std::fs::exists(&new_dir_name).map_err(|e|e.to_string())?
                    {
                        std::fs::remove_dir_all(&new_dir_name).map_err(|e|e.to_string())?;
                    }
                    
                    copy_dir::copy_dir(&path, new_dir_name)
                        .map_err(|e|e.to_string())?;

                    
                    let new_dep = Dependency {
                        name: pack.name.chars().filter(|c| !c.is_whitespace()).collect(),
                        source: path.clone(),
                        version: pack.version,
                    };
                    add_dep_to_config(new_dep, CONFIG_PATH)?;   
                    success(&format!("Successfully added {} as a dependency.", &path));
                },
                (None,Some(url))=> {
                    let new_package = add_dependency(url.clone())?;
                    add_dep_to_config(new_package, CONFIG_PATH)?;            
                    success(&format!("Successfully added {} as a dependency.", &url));
                },
                _ => unreachable!()
            }

            if add_group.tree {
                print_dep_tree()?
            }
            
        },
        Commands::Update => {
            for dep_table in create_config_from_path(&PathBuf::from(CONFIG_PATH))?.dependencies.unwrap()
            {
                let dep = table_to_dep(dep_table.1.as_table().expect("Dependency entry should be a table!"))?;
                let url = Url::parse(&dep.source)
                    .map_err(|e|e.to_string())?;

                if url.has_host()
                {
                    let dest : PathBuf = [get_hc_filepath()?, get_dep_filename(&dep)?.into()].into_iter().collect();
                    git2::Repository::clone(&url.to_string(), dest)
                        .map_err(|e|e.to_string())?;
                }
            }
        },
        Commands::Version => {
            println!("gup version: {}", env!("CARGO_PKG_VERSION"))
        },
        Commands::Tree => {
            print_dep_tree()?
        }
    }
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
