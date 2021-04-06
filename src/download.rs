use std::{any::Any, fs::{File, read_dir, read_to_string, remove_dir, remove_file}, io::{self, Write}, path::Path, process::exit};
use serde::{Serialize, Deserialize};
use serde_json::{Value, from_str};
use zip::ZipArchive;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    authors: Vec<String>,
    title: String,
    description: String,
    update_url: String
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadList {
    #[serde(alias = "$schema")]
    schema: String,
    file_info: FileInfo
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Playerarray {
    steamid: Value,
    attributes: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct List {
    #[serde(alias = "$schema")]
    schema: String,
    file_info: FileInfo,
    players: Vec<Playerarray>
}


pub struct Downloader { 
    client: reqwest::Client
}

/* supposed to download/unpack and parse the lists */
impl Downloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new()
        }
    }


    /* downloads the zip and writes it to a .zip via rust I/O */
    pub async fn download_lists(&mut self, url: String) -> Result<(), Box<dyn std::error::Error>> {

        let resp = self.client.get(url).send().await?;
        let bytes = resp.bytes().await?;

        let mut file = File::create("lists/misc/lists.zip").unwrap();
        file.write_all(&bytes).unwrap();

        Downloader::unzip(self).await?;

        Ok(())
    }

    /* unzips the file using the zip crate */
    pub async fn unzip(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open("lists/misc/lists.zip").unwrap();
        let mut archive = ZipArchive::new(file).unwrap();

        /* iterate over all fils in archive */
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            let path = match file.enclosed_name() {
                Some(path ) => "lists/misc/".to_owned() + path.to_str().unwrap(),
                None => exit(-1),
            };

            let mut out = File::create(&path).unwrap();
            io::copy(&mut file, &mut out).unwrap();
        }
        
        /* delete zip again */
        remove_file("lists/misc/lists.zip").unwrap();

        Downloader::parse_lists(self);

        Ok(())
    }

    /* parses json into custom struct */
    pub async fn parse_lists(&mut self) -> Result<Vec<List>, Box<dyn std::error::Error>> {
        let mut lists: Vec<List> = vec![];
        for file in read_dir("lists/misc/").unwrap() {
            let file = file.unwrap();
            let content = read_to_string("lists/misc/".to_owned() + file.file_name().to_str().unwrap()).unwrap();

            let dlist: DownloadList = serde_json::from_str(&content).unwrap();

            /* download actual list */
            let resp = self.client.get(dlist.file_info.update_url).send().await?;
            let bytes = &resp.bytes().await?;

            let fname = "lists/".to_owned() + &dlist.file_info.title + ".json";
            
            if fname.contains("Moeb") {
                continue;
            }

            let mut file = File::create(&fname).unwrap();
            file.write_all(&bytes).unwrap();   
            let filecontent = read_to_string(&fname).unwrap();
            let list: List = serde_json::from_str(&filecontent).unwrap();

            lists.push(list);
        }
        Ok(lists)
    }

}