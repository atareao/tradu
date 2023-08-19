mod deepl;

use deepl::DeepL;
use tokio;
use std::str::FromStr;
use tracing_subscriber::{EnvFilter,
    layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{debug, error};
use std::{process, path::PathBuf};
use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    text: String,

    #[arg(short, long)]
    source_lang: Option<String>,

    #[arg(short, long)]
    dest_lang: Option<String>,
}

#[tokio::main]
async fn main() {
    let config = match get_config().await{
        Some(path) => path,
        None => {
            let mut path = std::env::current_dir().unwrap();
            path.push("tradu.yml");
            DeepL::write_default(&path).await;
            path
        }
    };
    let deepl = DeepL::read_content(&config).await;
    if deepl.get_auth_key() == ""{
        println!("Error: Auth Key in {} is empty", &config.to_str().unwrap());
        process::exit(1);

    }
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(deepl.get_log_level()).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let source_lang = match args.source_lang{
        Some(value) => value,
        None => deepl.get_source_lang().to_string(),
    };
    let target_lang = match args.dest_lang{
        Some(value) => value,
        None => deepl.get_target_lang().to_string(),
    };
    match deepl.translate(&args.text, &source_lang, &target_lang).await{
        Ok(response) => {
            println!("{}", response)
        },
        Err(e) => {
            error!("{}", e);
        }
    }
}

async fn get_config() -> Option<PathBuf>{
    let mut current_path = std::env::current_dir().unwrap();
    current_path.push("tradu.yml");
    debug!("Current path: {}", current_path.display());
    if(tokio::fs::metadata(&current_path)).await.is_ok(){
        return Some(current_path);
    }
    let mut exe_path = std::env::current_exe().unwrap();
    exe_path.push("tradu.yml");
    debug!("Exe path: {}", exe_path.display());
    if(tokio::fs::metadata(&exe_path)).await.is_ok(){
        return Some(exe_path);
    }
    let mut home_path = dirs::home_dir().unwrap();
    debug!("Home path: {}", home_path.display());
    home_path.push(".tradu.yml");
    if(tokio::fs::metadata(&home_path)).await.is_ok(){
        return Some(home_path);
    }
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("tradu.yml");
    debug!("Config path: {}", config_path.display());
    if(tokio::fs::metadata(&config_path)).await.is_ok(){
        return Some(config_path);
    }
    let mut config_folder = dirs::config_dir().unwrap();
    config_folder.push("tradu");
    config_folder.push("tradu.yml");
    debug!("Config folder: {}", config_folder.display());
    if(tokio::fs::metadata(&config_folder)).await.is_ok(){
        return Some(config_folder);
    }
    None
}
