use std::{fs::{File, remove_file}, io::{self, Write}, path::Path, process::exit};

use zip::ZipArchive;

pub struct List {
    
}


pub struct Downloader { 
    client: reqwest::Client,
    lists: Vec<List>
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            lists: vec![],
        }
    }


    /* downloads the zip and writes it to a .zip via rust I/O */
    pub async fn download_lists(&mut self, url: String) -> Result<(), reqwest::Error> {

        let resp = self.client.get(url).send().await?;
        let bytes = resp.bytes().await?;

        if Path::new("lists.zip").is_dir() {
            remove_file("lists.zip").unwrap();
        }

        let mut file = File::create("lists/lists.zip").unwrap();
        file.write_all(&bytes).unwrap();

        Ok(())
    }

    /* unzips the file using the zip crate */
    pub async fn unzip(&mut self) -> Result<(), std::io::Error> {
        let file = File::open("lists/lists.zip").unwrap();
        let mut archive = ZipArchive::new(file).unwrap();

        println!("{:?}", archive);

        /* iterate over all fils in archive */
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            let path = match file.enclosed_name() {
                Some(path ) => path.to_owned(),
                None => exit(-1),
            };

            let mut out = File::create(&path).unwrap();
            io::copy(&mut file, &mut out).unwrap();

        }

        Ok(())
    }
}