use std::{env::home_dir, path::PathBuf};

use colored::{ColoredString, Colorize};
use url::Url;
use termtree::*;

use crate::{config, logging::*};

pub fn validate_dependency( location : String ) -> Result<(), ColoredString> {
    debug("validate_dependency", &format!("Validating dependency \"{}\"", &location));
    match Url::parse(&location){
        Ok(url) => {

            if !url.has_host()
            {
                if std::fs::exists(location).unwrap() {return Ok(())} // file path dependency
                else {return std::result::Result::Err(format!("{}: Invalid dependency url - \"{}\"", "validate_dependency", url).into())}
            }

            

            if url.host_str() != Some("github.com") {
                return std::result::Result::Err(format!("{}: Invalid dependency url - \"{}\" (Is it a github link?)", "validate_dependency", url).into())
            }else {
                Ok(())
            }
        },
        Err(err) => {
            return std::result::Result::Err(format!("{}: Unable to parse dependency url - \"{}\"", "validate_dependency", err).into())
        },
    }
}


pub fn add_dependency (url : String) -> Result<String, ColoredString>
{
    let hcc_path : PathBuf = [home_dir().unwrap(), PathBuf::from(".hc")].iter().collect();
    let dep_path = hcc_path.join("temp");
    
    // create hc directory if not exist
    if !std::fs::exists(hcc_path.clone()).unwrap() {
        std::fs::create_dir(hcc_path.clone())
            .map_err(|e| e.to_string().red())?;
    }

    git2::Repository::clone(&url, dep_path.clone())
        .map_err(|e| e.to_string().red())?;

    
    // find package name from config file
    let dep_config_path : PathBuf = [dep_path.clone(), PathBuf::from("Config.toml")].iter().collect();
    let dep_config = crate::config::create_config_from_path(&dep_config_path.to_str().expect("Dependency path not found.").into())?;
    match dep_config.package {
        Some(pack) => {
            let package_name = format!("{}-{}",pack.name, pack.version);
            let new_dep_path = hcc_path.join(package_name.clone());
            debug("PDM", &format!("Path to new dependency - \"{}\"", new_dep_path.to_str().unwrap()));

            match std::fs::exists(new_dep_path.clone()){
                Ok(false) => {
                    std::fs::rename(dep_path, new_dep_path)
                        .map_err(|e| e.to_string().red())?;
                },
                Ok(true) => {
                    std::fs::remove_dir_all(dep_path)
                        .map_err(|e| e.to_string().red())?;
                    warn("PDM", &format!("{} package already exists.", new_dep_path.to_string_lossy()))
                },
                Err(e) => return Err(format!("Unable to create directory: {}", e).into())
            }
            

            Ok(package_name)
        },
        None => Err("Dependency has invalid config!".into())
    }
    
}

pub fn get_dep_cfg(dep : (String, toml::Value)) -> Result<crate::Config, ColoredString>
{
    let cfg_path: PathBuf = match std::fs::exists(&dep.1.to_string()) {
        Ok(true) =>{dep.1.to_string().into()},
        Ok(false) =>{[home_dir().unwrap(), ".hc".into(), dep.0.into()].iter().collect()},
        Err(_)=> {[home_dir().unwrap(), ".hc".into(), dep.0.into()].iter().collect()},
    };
    debug("get_dep_cfg", &format!("Getting config from {}", cfg_path.join("Config.toml").to_str().unwrap()));
    config::create_config_from_path(&PathBuf::from(cfg_path).join("Config.toml"))
}

pub fn get_dep_filename_from_cfg(cfg : &config::Config) -> Result<String, ColoredString>
{
    let pack = cfg.package.as_ref().unwrap();
    Ok(format!("{}-{}", pack.name, pack.version))
}

pub fn get_dep_tree(cfg : crate::Config) -> Result<Tree<String>, ColoredString>
{
    // TODO: Deal with circular dependencies
    let tree_name = get_dep_filename_from_cfg(&cfg)?;
    let mut tree = Tree::new(tree_name);
    if cfg.dependencies != None
    {
        for dep in cfg.dependencies.unwrap()
        {    
            let child_subtree = get_dep_tree(get_dep_cfg(dep)?)?;
            tree.push(child_subtree);
        }   
    }

    Ok(tree)

}
