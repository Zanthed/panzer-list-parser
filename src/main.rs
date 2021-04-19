use core::panic;
use std::{fs::{create_dir_all}, path::Path, io::ErrorKind};

mod download;
mod handler;

use download::Downloader;
use handler::Handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("lists/misc").is_dir() {
        println!("Creating directories lists/misc");
        let dir = create_dir_all("lists/misc").unwrap_or_else(|e| {
            if e.kind() == ErrorKind::PermissionDenied {
                panic!("Failed to create directories list/misc. Permission denied.\n{:?}", e)
            } else {
                panic!("Unknown error creating directories lists/misc.\nError: {:?}", e)
            }
        });
    }
    else if Path::new("lists/misc").is_dir() {
        println!("lists/misc already exists, continuing.")
    }

    let mut downloader = Downloader::new();
 
    downloader.download_lists("https://github.com/incontestableness/TF2BD-Community-Lists/raw/main/All.zip".to_string()).await?;

    Ok(())
}
