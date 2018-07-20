#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub voice: Option<String>,
    pub speak_rate: Option<String>,
    pub aws_access_key: Option<String>,
    pub aws_secret_key: Option<String>,
    pub aws_region: Option<String>,
}

use dirs;
use ini::Ini;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use toml;

impl Default for Config {
    fn default() -> Self {
        let region = if let Ok(region) = env::var("AWS_REGION") {
            Some(region)
        } else {
            Some("eu-west-1".to_string())
        };

        Config {
            voice: Some("Salli".to_string()),
            speak_rate: Some("fast".to_string()),
            aws_access_key: env::var("AWS_ACCESS_KEY").ok(),
            aws_secret_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            aws_region: region,
        }
    }
}

impl Config {
    pub fn new() -> Config {
        let mut config = Config::default();

        if let Ok(file) = Ini::load_from_file(dirs::home_dir().unwrap().join(".aws/credentials")) {
            if let Some(aws) = file.section(Some("default".to_owned())) {
                if let Some(access_key) = aws.get("aws_access_key_id") {
                    config.aws_access_key = Some(access_key.to_string());
                }

                if let Some(secret_key) = aws.get("aws_secret_access_key") {
                    config.aws_secret_key = Some(secret_key.to_string());
                }
                if let Some(region) = aws.get("region") {
                    config.aws_region = Some(region.to_string());
                }
            }
        }

        if let Ok(mut file) = File::open(dirs::home_dir().unwrap().join(".config/tts.toml")) {
            let mut content = String::new();
            file.read_to_string(&mut content).is_ok();

            match toml::from_str::<Config>(&content) {
                Ok(mut cfg) => {
                    if cfg.voice.is_some() {
                        config.voice = cfg.voice.take();
                    }
                    if cfg.speak_rate.is_some() {
                        config.speak_rate = cfg.speak_rate.take();
                    }
                    if cfg.aws_access_key.is_some() {
                        config.aws_access_key = cfg.aws_access_key.take();
                    }
                    if cfg.aws_secret_key.is_some() {
                        config.aws_secret_key = cfg.aws_secret_key.take();
                    }
                    if cfg.aws_region.is_some() {
                        config.aws_region = cfg.aws_region.take();
                    }
                }
                Err(e) => panic!(e),
            }
        }

        let value = config
            .clone()
            .aws_access_key
            .unwrap_or_else(|| panic!("Missing access key"));

        env::set_var("AWS_ACCESS_KEY_ID", value);

        let value = config
            .clone()
            .aws_secret_key
            .unwrap_or_else(|| panic!("Missing secret key"));
        env::set_var("AWS_SECRET_ACCESS_KEY", value);

        config
    }
}
