use std::{collections::hash_map, env::home_dir, path::PathBuf};

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
    let dep_path = get_hc_filepath()?.join("temp");

    git2::Repository::clone(&url, dep_path.clone())
        .map_err(|e| e.to_string().red())?;

    
    // find package name from config file
    let dep_config_path : PathBuf = [dep_path.clone(), PathBuf::from("Config.toml")].iter().collect();
    let dep_config = crate::config::create_config_from_path(&dep_config_path.to_str().expect("Dependency path not found.").into())?;
    match &dep_config.package {
        Some(pack) => {
            let package_name = format!("{}-{}", pack.name.chars().filter(|c| !c.is_whitespace()).collect::<String>(), pack.version);
            let new_dep_path = get_hc_filepath()?.join(package_name.clone());
            debug("PDM", &format!("Path to new dependency - \"{}\"", new_dep_path.to_string_lossy()));

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
                name : pack.name.clone().chars().filter(|c| !c.is_whitespace()).collect(),
                version : pack.version.clone(),
                source : url,
            };
            Ok(return_value)
        },
        None => Err("Dependency has invalid config!".into())
    }
    
}

pub fn get_dep_cfg(dep : Dependency) -> Result<crate::Config, ColoredString>
{
    let name_version = get_dep_filename(&dep)?;
    let full_path : PathBuf = [get_hc_filepath()?, name_version.into(), "Config.toml".into()].iter().collect();
    debug("get_dep_cfg", &format!("Getting config from {}", full_path.to_string_lossy()));
    config::create_config_from_path(&full_path)
}

pub fn get_dep_filename(dep : &Dependency) -> Result<String, ColoredString>
{
    Ok(format!("{}-{}", dep.name.chars().filter(|c| !c.is_whitespace()).collect::<String>(), dep.version))
}

pub fn get_dep_tree(cfg : crate::Config) -> Result<Tree<String>, ColoredString>
{
    fn get_tree_recursive(cfg : crate::Config, hm : &mut hash_map::HashMap<String, bool>) -> Result<Tree<String>, ColoredString>
    {
        let pack = cfg.package.expect("Dependency should have a package field!");
        let cur_dep = Dependency {
            name: pack.name,
            version: pack.version,
            source: String::new(),
        };
        let tree_name = get_dep_filename(&cur_dep)?;
        let mut tree = Tree::new(tree_name);
        if let Some(deps) = cfg.dependencies.as_ref()
        {
            for dep in deps
            {    
                let new_dep = table_to_dep(dep.1.as_table().expect("Unable to create table from dependency"))?;
                if hm.contains_key(&get_dep_filename(&new_dep)?)
                {
                    tree.push(format!("{} *", get_dep_filename(&new_dep)?));
                }else {
                    hm.insert(get_dep_filename(&new_dep)?, true);
                    let child_subtree = get_tree_recursive(get_dep_cfg(new_dep)?, hm)?;
                    tree.push(child_subtree);
                }
                
            }   
        }
        Ok(tree)
    }
    Ok(get_tree_recursive(cfg, &mut hash_map::HashMap::new())?)

    

}

pub fn table_to_dep (table : &Table) -> Result<Dependency, ColoredString>
{
    Ok(Dependency {
        name: table.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Dependency table missing or invalid 'name' field".red())?
            .into(),
        version: table.get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Dependency table missing or invalid 'version' field".red())?
            .into(),
        source: table.get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Dependency table missing or invalid 'source' field".red())?
            .into(),
    })
}

pub fn print_dep_tree () -> Result<(), ColoredString>
{
    let cfg = config::create_config_from_path(&crate::CONFIG_PATH.into())?;
    let tree = get_dep_tree(cfg)?;
    println!("{}", tree);
    Ok(())
}

pub fn get_hc_filepath() -> Result<PathBuf, ColoredString>
{
    if let Some(home) = home_dir() 
    {
        let hc_path = home.join(".hc");

        // create hc directory if not exist
        match std::fs::exists(&hc_path) {
            Ok(true) => {},
            Ok(false) => {
                std::fs::create_dir(&hc_path)
                    .map_err(|e| e.to_string().red())?;
            },
            Err(e) => return Err(format!("Unable to create directory: {}", e).into())
        }

        Ok(hc_path)
    }else {
        Err("Unable to find home directory.".into())
    }

}

pub fn update_dependencies() -> Result<(), ColoredString>
{
    for dep_table in config::create_config_from_path(&PathBuf::from(crate::CONFIG_PATH))?.dependencies.unwrap()
    {
        let dep = table_to_dep(dep_table.1.as_table().expect("Dependency entry should be a table!"))?;
        let url = url::Url::parse(&dep.source)
            .map_err(|e|e.to_string())?;

        if url.has_host()
        {
            let dest : PathBuf = get_hc_filepath()?.join(get_dep_filename(&dep)?);
            git2::Repository::clone(&url.to_string(), dest)
                .map_err(|e|e.to_string())?;
        }
    }
    Ok(())
}