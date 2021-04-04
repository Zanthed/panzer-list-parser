use std::{fs::create_dir, path::Path, process::exit};

mod download;
mod handler;

use download::Downloader;
use handler::Handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("lists").is_dir() {
        create_dir("lists").expect("Unable to create lists directory");
        exit(-1);
    }

    let mut downloader = Downloader::new();

    
    downloader.download_lists("https://github.com/incontestableness/TF2BD-Community-Lists/raw/main/All.zip".to_string()).await?;
    downloader.unzip().await?;
    downloader.parse_lists().await?;
    

    Ok(())
}
