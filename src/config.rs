//! Process configurations for Lightning sites.


// First-party
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;

// Third-party
use yaml_rust::{Yaml, YamlLoader};


const CONFIG_FILE_NAME: &'static str = "lightning.yaml";
const CONTENT_DIRECTORY: &'static str = "content_directory";
const OUTPUT_DIRECTORY: &'static str = "output_directory";
const TEMPLATE_DIRECTORY: &'static str = "directory";


pub struct Config {
    pub site: Site,
    pub directories: Directories,
    pub taxonomies: Vec<Taxonomy>,
}


pub struct Directories {
    pub content: PathBuf,
    pub output: PathBuf,
    pub template: PathBuf,
}


pub enum Taxonomy {
    Binary { templates: Templates },
    Multiple {
        name: String,
        limit: Option<u8>,
        required: bool,
        hierarchical: bool,
        templates: Templates,
    },
    Temporal {
        required: bool,
        templates: Templates,
    },
}


pub struct Site {
    pub name: String,
    pub description: String,
    pub metadata: HashMap<Yaml, Yaml>,
    pub url: ValidatedUrl,
}


pub struct Templates {
    item: String,
}


pub struct ValidatedUrl(String);

impl ValidatedUrl {
    /// Get a URL. `Err` if the item passed in is not a spec-conformant URL.
    pub fn new(unvalidated_url: String) -> Result<ValidatedUrl, String> {
        // TODO: validate the URLs!
        Ok(ValidatedUrl(unvalidated_url))
    }

    pub fn value(&self) -> String {
        self.0.clone()
    }
}


fn path_buf_from_yaml(yaml: &Yaml, key: &str, config_path: &PathBuf) -> Result<PathBuf, String> {
    match yaml {
        &Yaml::String(ref path_str) => Ok(PathBuf::from(path_str)),
        value => Err(format!("invalid `{:}` value {:?} in {:?}", key, value, config_path)),
    }
}


pub fn load(directory: &PathBuf) -> Result<Config, String> {
    let config_path = directory.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Err(format!("The specified configuration path {:?} does not exist.",
                           config_path.to_string_lossy()));
    }

    let mut file = File::open(&config_path)
        .map_err(|reason| format!("Error reading {:?}: {:?}", config_path, reason))?;

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(err) => return Err(String::from(err.description())),
    };

    // We need all these intermediate bindings because the temporaries created
    // along the way don't live long enough otherwise.
    let yaml_config = YamlLoader::load_from_str(&contents)
        .map_err(|err| format!("{} ({:?})", err, &config_path))?;
    let yaml_config = yaml_config.into_iter().next().ok_or("Empty configuration file")?;
    let yaml_config = yaml_config.as_hash().ok_or("Configuration is not a map")?;

    let content_directory_yaml = yaml_config.get(&Yaml::from_str(CONTENT_DIRECTORY))
        .ok_or(format!("No `{:}` key in {:?}", CONTENT_DIRECTORY, config_path))?;

    let content_directory =
        path_buf_from_yaml(&content_directory_yaml, CONTENT_DIRECTORY, &config_path)?;

    let output_directory_yaml = yaml_config.get(&Yaml::from_str(OUTPUT_DIRECTORY))
        .ok_or(format!("No `{:} key in `{:?}", OUTPUT_DIRECTORY, config_path))?;

    let output_directory =
        path_buf_from_yaml(output_directory_yaml, OUTPUT_DIRECTORY, &config_path)?;

    let structure = yaml_config.get(&Yaml::from_str("structure"))
        .ok_or(format!("No `structure` key in {:?}", config_path))?
        .as_hash()
        .ok_or(format!("`structure` is not a map in {:?}", config_path))?;

    let template_directory_yaml = structure.get(&Yaml::from_str(TEMPLATE_DIRECTORY))
        .ok_or(format!("No `directory` key in `structure` in {:?}", config_path))?;

    let template_directory =
        path_buf_from_yaml(&template_directory_yaml, TEMPLATE_DIRECTORY, &config_path)?;

    Ok(Config {
        site: Site {  // TODO
            name: String::new(),
            description: String::new(),
            metadata: HashMap::new(),
            url: ValidatedUrl(String::new()),
        },
        directories: Directories {
            content: content_directory,
            output: output_directory,
            template: template_directory,
        },
        taxonomies: Vec::new(),
    })
}