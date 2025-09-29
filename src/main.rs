use colored::{Colorize,ColoredString};
use clap::*;
use log::error;
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
        Commands::Run => {
            let cfg = create_config_from_path(&PathBuf::from(CONFIG_PATH))?;
            run(&cfg)?;
        },
        Commands::Init(init_group) => {
            // Initialize a new halcyon project
            // check if config already exists
            match std::fs::exists(CONFIG_PATH).unwrap() {
                true => {
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
                false => {}
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
                let content = String::from("module ") + std::path::PathBuf::from(&arg).file_stem().unwrap().to_str().expect("Filename contains invalid characters") + " =\n(* Your code here! *)\nend";
                std::fs::write(std::path::PathBuf::from(arg), content)
                    .map_err(|e| e.to_string().red())?;
            }
            // write the config
            write_config(&cfg, "./Config.toml".into())?;

            if !init_group.no_git {
                // make a git repo
                let repo = git2::Repository::init(".")
                    .map_err(|e| e.to_string())?;

                let content = "# Halcyon build artifacts\n*.wasm";
                std::fs::write(".gitignore", content).map_err(|e| e.to_string())?;
                
                info("Init", &format!("Initialized empty Git repository at {:?}", repo.path()));
            }
        },
        Commands::Doc => {
            let cfg = create_config_from_path(&PathBuf::from(CONFIG_PATH))?;
            parse::create_docs(cfg)?;
        },
        Commands::Add(add_group) => {
            let new_package = add_dependency(add_group.url.clone())?;
            add_dep_to_config(&new_package, &add_group.url, CONFIG_PATH)?;            
            info("Add", &format!("Successfully added {} as a dependency.", &new_package));
        },
        Commands::Update => todo!(),
        Commands::Version => {
            println!("gup version: {}", env!("CARGO_PKG_VERSION"))
        },
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
