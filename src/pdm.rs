use std::{env::home_dir, path::PathBuf};

use colored::{ColoredString, Colorize};
use url::Url;

use crate::logging::*;

pub fn validate_dependency( url : String ) -> Result<(), ColoredString> {
    debug("PDM", &format!("Validating dependency \"{}\"", &url));
    let dep_url = Url::parse(&url);
    match dep_url{
        Ok(url) => {

            if !url.has_host()
            {
                return std::result::Result::Err(format!("{}: Invalid dependency url - \"{}\"", "Config error".red(), url).into())
            }

            if url.host_str() != Some("github.com") {
                return std::result::Result::Err(format!("{}: Invalid dependency url - \"{}\" (Is it a github link?)", "Config error".red(), url).into())
            }else {
                Ok(())
            }
        },
        Err(err) => {
            return std::result::Result::Err(format!("{}: Unable to parse dependency url - \"{}\"", "Config error".red(), err).into())
        },
    }
}



pub fn add_dependency (url : String) -> Result<String, ColoredString>
{
    validate_dependency(url.clone())?;
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
                    warn("PDM", &format!("{} package already exists.", new_dep_path.to_string_lossy()))
                },
                Err(e) => return Err(format!("Unable to create directory: {}", e).into())
            }
            

            Ok(package_name)
        },
        None => Err("Dependency has invalid config!".into())
    }
    
}
