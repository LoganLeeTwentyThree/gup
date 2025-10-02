use std::path::PathBuf;
use std::process::Command;
use std::env::home_dir;

use crate::config::{Config};
use crate::logging::*;
use crate::pdm::{add_dependency, get_dep_cfg, table_to_dep};
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
                Ok(false) =>
                {
                    add_dependency(depfile.1.to_string())?;
                    args.push(depfile.1.to_string());
                },
                _=>
                {
                    let full_path: PathBuf = [home_dir().unwrap(), ".hc".into(), depfile.0.into()].iter().collect();
                    args.push(full_path.to_str().unwrap().into());
                }
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
        for dep_table in config.dependencies.clone().unwrap(){
            let dep = table_to_dep(dep_table.1.as_table().expect("Unable to create dependency table"))?;
            args.push("-i".into());
            println!("{}",dep.name);
            let cfg_path: PathBuf = {
                    let dir_name = format!("{}-{}", dep.name, dep.version);
                    [home_dir().unwrap(), ".hc".into(), dir_name.into()].iter().collect()
            };
                

            let dep_name_version = format!("{}-{}", dep.name, dep.version);
            for infile in get_dep_cfg(dep_name_version)?.build.infiles{
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