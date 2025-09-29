use std::path::PathBuf;

use colored::{ColoredString, Colorize};
use toml::Table;

use crate::logging::*;
use crate::pdm::validate_dependency;

//config file struct
#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
pub struct Config {
    pub package: Option<Package>,
    pub build: Build,
    pub dependencies: Option<Table>,
}

//config file struct
#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
pub struct Build{
    pub infiles: Vec<String>,
    pub outfile: String,
    pub docfile: Option<String>
}

#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

pub fn resolve_config(
    source: Source,
    output: Option<String>,
    docs: Option<String>
) -> std::result::Result<Config, ColoredString> {
    match (source.config_file, source.input_path, output) {
        (Some(cfg), None, None) => create_config_from_path(PathBuf::from(cfg)),
        (Some(cfg), None, Some(out)) => {

            let dependencies = create_dependencies(cfg.clone())?;

            let config = create_config_from_path(PathBuf::from(cfg))?;
            create_config(
                config.build.infiles, 
                out, 
                docs,
                dependencies)
        },
        (_, Some(inp), out) => create_config(
            vec![inp], 
            out.unwrap_or("./a.wasm".into()), 
            docs,
        None),
        _ => unreachable!("Clap enforces mutual exclusion"),
    }
}

pub fn create_dependencies(path : String) -> std::result::Result<Option<Table>, ColoredString>
{
    let cfgfile = std::fs::read_to_string(std::path::PathBuf::from(path)).map_err(|e| format!("{} {}", "Config error:\n".red(), e.to_string()))?;
    let cfg : Config = toml::from_str(&cfgfile).map_err(|e| e.to_string() + &"\nCould not create config".red())?;
    validate_config(&cfg)?;
    Ok(cfg.dependencies)
}

pub fn create_config_from_path(path : PathBuf) -> std::result::Result<Config, ColoredString>
{
    debug("Config",&format!("Creating config from \"{}\"", path.to_str().unwrap()));
    let cfgfile = std::fs::read_to_string(path)
        .map_err(|e| format!("{} {}", "Config error:\n".red(), e.to_string()))?;
    let cfg : Config = toml::from_str(&cfgfile).map_err(|e| e.to_string() + &"\nCould not create config".red())?;
    validate_config(&cfg)?;
    Ok(cfg)
}

pub fn create_config(ins : Vec<String>, out : String, dfile : Option<String>, deps : Option<Table>) -> std::result::Result<Config, ColoredString>
{
    log::debug!("Config: Creating config");

    let cfg : Config =
    Config {
        build: Build {
            infiles: ins,
            outfile: out,
            docfile: dfile,

        },
        dependencies: deps,
        package: None
    };

    validate_config(&cfg)?;
    Ok(cfg)
}

pub fn validate_config(cfg : &Config) -> Result<(), ColoredString>
{
    debug("Config","Validating config");
    //check infiles for errors
    for arg in &cfg.build.infiles {
        debug("Config",&format!("Checking input file \"{}\" ", arg));
        let path= std::path::Path::new(&arg);
        if std::fs::exists(path).unwrap() == true {
            match path.extension().unwrap().to_str() {
                Some("hc") => {},
                Some("wasm") =>{},
                _ => return std::result::Result::Err(format!("{}: {} \"{}\"","Config Error:".red(), "Invalid input filename", &arg).into()),
            }
        }
        
    }

    debug("Config",&format!("Checking output file {} ", cfg.build.outfile));
    //check outfile for errors
    match std::path::Path::new(&cfg.build.outfile).extension().unwrap().to_str() {
        Some("wasm") => {},
        _ => return std::result::Result::Err(format!("{}: {} \"{}\"","Config error".red(), "Invalid output filename:", &cfg.build.outfile).into()),
    }

    if cfg.build.infiles.len() < 1
    {
        return std::result::Result::Err(String::from(format!("{} {}", "Config Error:".red(), "Please provide one or more input files!")).into())
    }

    if cfg.build.outfile.is_empty()
    {
        return std::result::Result::Err(String::from(format!("{} {}", "Config Error:".red(), "Please provide exactly one output file!")).into())
    }

    
    //check docfile if it exists
    match &cfg.build.docfile {
        None => {},
        Some(path) => {
            debug("Config",&format!("checking docfile \"{}\"", path));
            match std::path::Path::new(&path).extension().unwrap().to_str() {
                Some("md") => {},
                _ => return std::result::Result::Err(format!("{}: {} \"{}\"","Config error:".red(), "Invalid doc filename: ".red(), &path).into()),
            }
        }
    }

    
    
    //check dependencies if they exist
    match &cfg.dependencies {
        None => {},
        Some(list) => {
            for url in list.clone() {
                validate_dependency(url.1.as_str().unwrap().into())?;
            }
        }
    }
     

    Ok(())
}

pub fn write_config( cfg : &Config, path : String ) -> Result<(), ColoredString>
{
    let config_contents = toml::to_string(cfg).unwrap();
    std::fs::write(std::path::PathBuf::from(path), &config_contents)
        .map_err(|e| e.to_string().red())?;
    Ok(())
}

pub fn add_dep_to_config (dep_name : &str, dep_url : &str, config_path : &str) -> Result<(), ColoredString>
{
    let config = create_config_from_path(config_path.into())?;
    
    let mut new_deps = match config.dependencies {
        Some(list) => list,
        None => Table::new(),
    };

    new_deps.insert(dep_name.into(), toml::Value::String(dep_url.into()));

    let new_config = Config {
        dependencies: Some(new_deps),
        ..config
    };

    write_config(&new_config, config_path.into())?;let config = create_config_from_path(config_path.into())?;
    
    let mut new_deps = match config.dependencies {
        Some(list) => list,
        None => Table::new(),
    };

    new_deps.insert(dep_name.into(), toml::Value::String(dep_url.into()));

    let new_config = Config {
        dependencies: Some(new_deps),
        ..config
    };

    write_config(&new_config, config_path.into())?;
    Ok(())
}
