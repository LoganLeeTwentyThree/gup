use std::path::PathBuf;
use std::process::{Command};
use std::env::home_dir;

use crate::config::{Config, Dependency};
use crate::logging::*;
use crate::pdm::{add_dependency, get_dep_cfg, get_dep_filename, table_to_dep};
use colored::{ColoredString, Colorize};

fn run_hcc( command : String, args : Vec<String>) -> std::result::Result<String, colored::ColoredString> {
    let mut build_command = Command::new("hcc");
    build_command.arg(&command);

    for arg in &args{
        build_command.arg(arg);
    }

    debug("Run_hcc", &format!("Running \"hcc {} {}\"", &command, args.join(" ")));

    match build_command.output(){
        Ok(out)=> {
            match out.status.code()
            {
                Some(0) =>
                {
                    let stdout  = std::str::from_utf8(&out.stdout[..]);

                    match stdout {
                        Ok(result) => {
                            Ok(result.into())
                        },
                        Err(e) =>
                        {
                            Err(e.to_string().into())
                        }
                    }
                }
                _=> Err(format!("hcc failed to compile:\n{}",std::str::from_utf8(&out.stdout[..]).unwrap()).into())
            }
        },
        Err(e) => {
            Err(format!("{} (Do you have hcc installed?)",e.to_string()).into())
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

fn add_dep_to_source(dep : Dependency, source : &mut Vec<String>) -> Result<(), ColoredString>
{
    let cfg_path: PathBuf = {
        let dir_name = get_dep_filename(&dep)?;
        [home_dir().unwrap(), ".hc".into(), dir_name.into()].iter().collect()
    };

    for infile in get_dep_cfg(dep)?.build.infiles{
        debug("add_dep_to_source", &format!("Adding {} to source", &infile));

        source.push("-i".into());
        let full_path = cfg_path.join(infile);
        source.push(full_path.to_str().unwrap().into());
    }
    Ok(())
}

pub fn build(config : &Config) -> std::result::Result<(), colored::ColoredString> {
    
    let mut args: Vec<String> = Vec::new();

    if config.dependencies != None{
        for depfile in config.dependencies.clone().unwrap(){
            args.push("-i".into());

            let dep = table_to_dep(depfile.1.as_table().expect("Unable to create table from dependency"))?;

            match std::fs::exists(&dep.source) {
                Ok(true)=>{
                    add_dep_to_source(dep, &mut args)?;
                },
                Ok(false) =>
                {
                    add_dependency(dep.source.clone())?;
                    add_dep_to_source(dep, &mut args)?
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
            
            add_dep_to_source(dep, &mut args)?
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

    let timer = start_step("Run");
    let out = run_hcc("run".into(), args)?;
    println!("{out}");
    elapsed("Run", timer);
    Ok(())
}