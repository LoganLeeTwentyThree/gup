use std::path::PathBuf;
use std::process::Command;
use std::env::home_dir;

use crate::config::{create_config_from_path, Config};
use crate::logging::*;
use colored::Colorize;

fn run_hcc( command : String, args : Vec<String>) -> std::result::Result<String, colored::ColoredString> {
    let mut build_command = Command::new("hcc");
    build_command.arg(&command);

    for arg in &args{
        build_command.arg(arg);
    }

    debug("Run_hcc", &format!("Running \"hcc {} {}\"", &command, args.join(" ")));

    match build_command.output(){
        Ok(out)=> {
            let stdout  = std::str::from_utf8(&out.stdout[..]);
            let stderr = std::str::from_utf8(&out.stderr[..]);

            match stderr {
                Ok(result) => {
                    println!("STDERR: \n{result}");
                },
                Err(e) =>
                {
                    return Err(e.to_string().into());
                }
            }

            match stdout {
                Ok(result) => {
                    return Ok(result.into());
                },
                Err(e) =>
                {
                    return Err(e.to_string().into());
                }
            }

            
        },
        Err(e) => {
            return Err(format!("{} (Do you have hcc installed?)",e.to_string()).into());
        }
    }
}

// checks if the infiles in a given config are valid
pub fn check_valid(config : &Config) -> std::result::Result<(), colored::ColoredString> {
    // TODO: More checks?
    // does each infile compile?
    for infile in &config.build.infiles {
        info("Check",&format!("Checking: {}", infile.blue()));
        let path = std::path::PathBuf::from(infile);
        match path.extension().unwrap().to_str() {
            Some("hc") => {}
            Some("wasm") => {// maybe something here later... idk 
                },
            _ => error(&format!("Check: Invalid infile type detected \"{}\"", infile)),
        }
        
        
    }
    Ok(())

} 

pub fn build(config : &Config) -> std::result::Result<(), colored::ColoredString> {
    
    let mut args: Vec<String> = Vec::new();

    if config.dependencies != None{
        for depfile in config.dependencies.clone().unwrap(){
            args.push("-i".into());
            match std::fs::exists(depfile.1.to_string()) {
                Ok(true)=>{
                    args.push(depfile.1.to_string());
                },
                Ok(false)=>{
                    let full_path: PathBuf = [home_dir().unwrap(), ".hc".into(), depfile.0.into()].iter().collect();
                    args.push(full_path.to_str().unwrap().into());
                },
                Err(e)=>{error(&e.to_string());},
            }
            
        }
    }
    

    for infile in &config.build.infiles{
        args.push("-i".into());
        args.push(infile.into());
    }

    let out = run_hcc("build".into(), args)?;
    println!("{out}");
    Ok(())
    
}

pub fn run(config : &Config, params : Vec<String>) -> std::result::Result<(), colored::ColoredString> {
    let mut args: Vec<String> = Vec::new();

    if config.dependencies != None {
        for depfile in config.dependencies.clone().unwrap(){
            args.push("-i".into());
            let cfg_path: PathBuf = match std::fs::exists(depfile.1.to_string()) {
                Ok(true) =>{depfile.1.to_string().into()},
                Ok(false) =>{[home_dir().unwrap(), ".hc".into(), depfile.0.into()].iter().collect()},
                Err(e)=> return Err(e.to_string().into()),
            };
            
            for infile in create_config_from_path(&cfg_path.join("Config.toml"))?.build.infiles{
                debug("Run", &format!("Adding {} to source", &infile));
                let full_path = cfg_path.join(infile);
                args.push(full_path.to_str().unwrap().into());
            }
        }
    }
    

    for infile in &config.build.infiles{
        args.push("-i".into());
        args.push(infile.into());
    }

    for param in params {
        args.push("-p".into());
        args.push(param);
    }

    let out = run_hcc("run".into(), args)?;
    println!("{out}");
    Ok(())
}