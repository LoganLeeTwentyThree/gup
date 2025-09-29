use regex::Regex;
use crate::config::Config;

pub fn create_docs(cfg : Config) -> Result<(),String>
{
    let mut content = String::new();
    for path in cfg.build.infiles{
        let source = std::fs::read_to_string(std::path::PathBuf::from(&path))
        .unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"));
        
        let title_matches = Regex::new(r"@title:\s*([^\r\n]*)").unwrap();
        let signature_matches = Regex::new(r"@signature:\s*([^\r\n]*)").unwrap();
        let description_matches = Regex::new(r"@description:\s*([^@]*)").unwrap();

        let titles: Vec<_> = title_matches.captures_iter(&source)
            .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
            .collect();

        let signatures: Vec<_> = signature_matches.captures_iter(&source)
            .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
            .collect();

        let descriptions: Vec<_> = description_matches.captures_iter(&source)
            .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
            .map(|d| d.replace("--", "")) // strip "--"
            .collect();

        // Use the longest length
        let max_len = titles.len().max(signatures.len()).max(descriptions.len());

        
        for i in 0..max_len {
            let title = titles.get(i).cloned().unwrap_or(String::from("No title provided"));
            let signature = signatures.get(i).cloned().unwrap_or(String::from("No signature provided"));
            let description = descriptions.get(i).cloned().unwrap_or(String::from("No description provided"));
            content.push_str(&format!("## {}: {}  \n{}  \n---\n", title, signature, description));
        }
    }
    
    std::fs::write(cfg.build.docfile.unwrap_or(String::from("./docs.md")), content).unwrap();
    Ok(())
}