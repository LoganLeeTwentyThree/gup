use std::{env::home_dir, path::{PathBuf}};

use colored::{ColoredString, Colorize};
use toml::Table;
use termtree::*;

use crate::{config::{self, Dependency}, logging::*};

/*pub fn validate_dependency( location : String ) -> Result<(), ColoredString> {
    debug("validate_dependency", &format!("Validating dependency \"{}\"", &location));
    match Url::parse(&location){
        Ok(url) => {

            if !url.has_host()
            {
                //it might be a file path 
                match std::fs::exists(location) {
                    Ok(true) => return Ok(()),
                    Ok(false) => return std::result::Result::Err(format!("{}: Unable to find dependency by path - \"{}\" (Is it in this directory?)", "validate_dependency", url).into()),
                    _ => return std::result::Result::Err(format!("{}: Invalid dependency url - \"{}\"", "validate_dependency", url).into())
                } 
                
            }else {Ok(())}
        },
        Err(err) => {
            return std::result::Result::Err(format!("{}: Unable to parse dependency url - \"{}\"", "validate_dependency", err).into())
        },
    }
}*/


pub fn add_dependency (url : String) -> Result<Dependency, ColoredString>
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
    match &dep_config.package {
        Some(pack) => {
            let package_name = format!("{}-{}", pack.name, pack.version);
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
            
            let return_value = Dependency {
                name : pack.name.clone(),
                version : pack.version.clone(),
                source : url,
            };
            Ok(return_value)
        },
        None => Err("Dependency has invalid config!".into())
    }
    
}

pub fn get_dep_cfg(dep_name_version : String) -> Result<crate::Config, ColoredString>
{

    let full_path : PathBuf = [home_dir().unwrap(), PathBuf::from(".hc"), dep_name_version.into(), "Config.toml".into()].iter().collect();
    debug("get_dep_cfg", &format!("Getting config from {}", full_path.to_string_lossy()));
    config::create_config_from_path(&full_path)
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
        for dep in &cfg.dependencies.clone().unwrap()
        {    
            let new_dep = table_to_dep(dep.1.as_table().expect("Unable to create table from dependency"))?;
            let child_subtree = get_dep_tree(get_dep_cfg(new_dep.name)?)?;
            tree.push(child_subtree);
        }   
    }

    Ok(tree)

}

pub fn table_to_dep (table : &Table) -> Result<Dependency, ColoredString>
{
    Ok(Dependency {
        name: table.get("name").expect("Table should have name field").as_str().unwrap().into(),
        version: table.get("version").expect("Table should have version field").as_str().unwrap().into(),
        source: table.get("source").expect("Table should have source field").as_str().unwrap().into(),
    })
}

pub fn print_dep_tree () -> Result<(), ColoredString>
{
    let cfg = config::create_config_from_path(&crate::CONFIG_PATH.into())?;
    let tree = get_dep_tree(cfg)?;
    println!("{}", tree);
    Ok(())
}
