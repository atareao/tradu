use hyper::Request;
use serde::{Serialize, Deserialize};
use tokio::fs::read_to_string;
use std::{process, path::PathBuf};
use serde_json::json;
use tracing::{info, debug, error};
use std::error::Error;
use std::fmt;

use hyper::Client;

#[derive(Debug)]
pub struct DeepLError{
    message: String,
}

impl DeepLError{
    pub fn new(message: &str) -> Self{
        Self {
            message: message.into()
        }
    }
}


// Display implementation is required for std::error::Error.
impl fmt::Display for DeepLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for DeepLError {} // Defaul


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeepL{
    #[serde(default = "get_default_log_level")]
    log_level: String,
    #[serde(default = "get_default_base_url")]
    base_url: String,
    #[serde(default = "get_default_endpoint")]
    endpoint: String,
    #[serde(default = "get_default_auth_key")]
    auth_key: String,
    #[serde(default = "get_default_source_lang")]
    source_lang: String,
    #[serde(default = "get_default_target_lang")]
    target_lang: String,
    #[serde(default = "get_default_split_sentences")]
    split_sentences: String,
    #[serde(default = "get_default_preserve_formatting")]
    preserve_formatting: bool,
    #[serde(default = "get_default_formality")]
    formality: String,


}

fn get_default_log_level() -> String{
    "info".to_string()
}

fn get_default_base_url() -> String {
    "api-free.deepl.com".to_string()
}

fn get_default_endpoint() -> String {
    "v2/translate".to_string()
}

fn get_default_auth_key() -> String {
    "".to_string()
}

fn get_default_source_lang() -> String {
    "ES".to_string()
}

fn get_default_target_lang() -> String {
    "EN".to_string()
}

fn get_default_split_sentences() -> String{
    "1".to_string()
}

fn get_default_preserve_formatting() -> bool{
    false
}

fn get_default_formality() -> String{
    "default".to_string()
}


impl DeepL{

    pub fn get_log_level(&self) -> &str {
        &self.log_level
    }

    pub fn get_auth_key(&self) -> &str{
        &self.auth_key
    }

    pub fn get_source_lang(&self) -> &str{
        &self.source_lang
    }

    pub fn get_target_lang(&self) -> &str{
        &self.target_lang
    }

    pub async fn save(&self, path: &PathBuf){
        let _ = tokio::fs::write(
            path,
            serde_yaml::to_string(&self).unwrap().as_bytes()
        ).await;
    }

    pub async fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String, Box<dyn Error + 'static>>{
        let url = format!("https://{}/{}", self.base_url, self.endpoint);
        debug!("Url: {}", url);
        debug!("Text: {}", text);
        debug!("Source lang: {}", source_lang);
        debug!("Target lang: {}", target_lang);
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build();
        let client = Client::builder().build(https);
        let body = serde_json::to_string(&json!({
            "source_lang": source_lang,
            "target_lang": target_lang,
            "split_sentences": self.split_sentences,
            "preserve_formatting": self.preserve_formatting,
            "formality": self.formality,
            "text": [ text ]
        })).unwrap();
        let request = Request::builder()
            .method("POST")
            .uri(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("DeepL-Auth-Key {}", self.auth_key))
            .body(hyper::Body::from(body))
            .unwrap();
        match client.request(request).await{
            Ok(resp) => {
                let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
                // Convert the body bytes to utf-8
                let body = String::from_utf8(body_bytes.to_vec()).unwrap();
                debug!("{}", &body);
                let data: serde_json::Value = serde_json::from_str(&body).unwrap();
                if data.get("error").is_some(){
                    let error = data.get("error").unwrap();
                    let message = error.get("message").unwrap().as_str().unwrap();
                    Err(Box::new(DeepLError::new(message)))
                }else{
                    let translation = data.get("translations")
                        .unwrap()
                        .as_array()
                        .unwrap()[0]
                        .get("text")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    Ok(translation.to_string())
                }
            },
            Err(e) => {
                error!("{}", e);
                Err(Box::new(e))
            }
        }
    }

    fn default() -> Self{
        Self{
            log_level: get_default_log_level(),
            base_url: get_default_base_url(),
            endpoint: get_default_endpoint(),
            auth_key: get_default_auth_key(),
            source_lang: get_default_source_lang(),
            target_lang: get_default_target_lang(),
            split_sentences: get_default_split_sentences(),
            preserve_formatting: get_default_preserve_formatting(),
            formality: get_default_formality(),
        }
    }

    pub async fn write_default(file: &PathBuf){
        let default = Self::default();
        let _ = tokio::fs::write(
            file,
            serde_yaml::to_string(&default).unwrap().as_bytes()).await;
    }

    pub async fn read_content(file: &PathBuf) -> DeepL{
        info!("File to read: {:?}", file);
        let content = match read_to_string(file)
            .await {
                Ok(value) => {
                    debug!("Content read: {}", value);
                    value
                },
                Err(e) => {
                    error!("Error: {}", e);
                    println!("Error with config file `config.yml`: {}",
                        e.to_string());
                    process::exit(1);
                }
            };
        match serde_yaml::from_str(&content){
            Ok(configuration) => configuration,
            Err(e) => {
                error!("Error: {}", e);
                println!("Error with config file `config.yml`: {}",
                    e.to_string());
                process::exit(1);
            }
        }
    }
}
