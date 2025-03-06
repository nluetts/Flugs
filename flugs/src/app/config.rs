use app_core::string_error::ErrorStringExt;
use std::{io::Read, path::PathBuf, str::FromStr};

#[derive(Debug)]
pub struct Config {
    pub search_path: PathBuf,
    pub svg_width: u64,
    pub svg_height: u64,
    pub x_label: String,
    pub y_label: String,
}

impl Default for Config {
    fn default() -> Self {
        let search_path = PathBuf::from("/tmp/");
        let svg_width = 800;
        let svg_height = 600;
        let x_label = "x-label".to_string();
        let y_label = "y-label".to_string();

        Self {
            search_path,
            svg_width,
            svg_height,
            x_label,
            y_label,
        }
    }
}

impl Config {
    pub fn from_config_file() -> Result<Self, String> {
        let mut config = Self::default();
        #[allow(deprecated)]
        let Some(home) = std::env::home_dir() else {
            return Err("could not determine home directory to load config file".into());
        };
        let config_raw = {
            let path = home.join(PathBuf::from(".flugs"));
            let mut file = std::fs::File::open(path).err_to_string("could not open config file")?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)
                .err_to_string("could not load config file")?;
            buf
        };
        for line in config_raw.lines() {
            // Lines starting with "#" are considered comments.
            if line.starts_with("#") {
                continue;
            }
            let mut iter = line.split("=");
            let key = iter.next();
            let val = iter.next();
            match (key, val) {
                (Some("search_path"), Some(path_str)) => {
                    let path = PathBuf::from_str(path_str)
                        .expect("could not parse 'search_path' as directory name");
                    config.search_path = path;
                }
                (Some("svg_width"), Some(width_str)) => {
                    if let Ok(width) = width_str.parse::<u64>() {
                        config.svg_width = width;
                    } else {
                        log::warn!("could not parse 'svg_width' as number")
                    }
                }
                (Some("svg_height"), Some(height_str)) => {
                    if let Ok(height) = height_str.parse::<u64>() {
                        config.svg_height = height;
                    } else {
                        log::warn!("could not parse 'svg_height' as number")
                    }
                }
                (Some("x_label"), Some(x_label)) => {
                    config.x_label = x_label.to_string();
                }
                (Some("y_label"), Some(y_label)) => {
                    config.y_label = y_label.to_string();
                }
                _ => continue,
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config_file() {
        #[allow(unused)]
        let res = Config::from_config_file();
        dbg!(res);
    }
}
