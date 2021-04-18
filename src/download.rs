use std::{any::Any, fs::{File, read_dir, read_to_string, remove_dir, remove_file}, io::{self, Write, Read, ErrorKind}, path::Path, str};
use serde::{Serialize, Deserialize};
use serde_json::{Value, from_str};
use zip::ZipArchive;
use sha2::{Sha256, Digest};

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
    // Pretty sure this puts all the bytes into memory but since it's ~1.6kb who cares.
    pub async fn download_lists(&mut self, url: String) -> Result<(), Box<dyn std::error::Error>> {

        let resp = self.client.get(url).send().await?;
        let bytes = resp.bytes().await?;

        let mut file = File::create("lists/misc/lists.zip").unwrap_or_else(|e| {
            if e.kind() == ErrorKind::PermissionDenied {
                panic!("Failed to create lists.zip in lists/misc. Permission denied.\n{:?}", e)
            } else {
                panic!("Unexpected error creating lists.zip in lists/misc.\nError: {:?}", e)
            }
        });

        file.write_all(&bytes).unwrap_or_else(|e| {
            if e.kind() == ErrorKind::PermissionDenied {
                panic!("Failed to write data to lists.zip. Permission denied.\n{:?}", e)
            } else if e.kind() == ErrorKind::UnexpectedEof {
                panic!("Failed to write data to lists.zip. Unexpected end of file.\n{:?}", e)
            } else if e.kind() == ErrorKind::NotFound {
                panic!("Failed to write data to lists.zip. File not found.\n{:?}", e)
            } else {
                panic!("Unexpected error writing data to lists.zip.\nError: {:?}", e)
            }
        });

        println!("Attempting to calculate SHA256 sum of lists.zip.");

        let mut hasher = Sha256::new();
        let file = File::open("lists/misc/lists.zip");

        let file = match file {
            Ok(mut file) => { 
                println!("Opened lists.zip. Copying data to hasher."); 
                io::copy(&mut file, &mut hasher)?;
                let hash = hasher.finalize();
                println!("SHA256 sum of your lists.zip: {:x}", hash);
                println!("As of 18 April 2021, lists.zip's SHA256 sum is 9ceed66481e62ddb8c07e183416a2c77b615f16cdbffa338fbd1281f89d5e1bb");
            }
            Err(e) => { println!("Failed opening lists.zip. Ignoring. \nError: {:?}", e) }
        };

        Downloader::unzip(self).await?;

        Ok(())
    }

    /* unzips the file using the zip crate */
    pub async fn unzip(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Opening lists.zip for reading.");
        let file = File::open("lists/misc/lists.zip").unwrap_or_else(|e| {
            if e.kind() == ErrorKind::NotFound {
                panic!("Failed to open lists.zip. File not found.\n{:?}", e)
            } else if e.kind() == ErrorKind::PermissionDenied {
                panic!("Failed to open lists.zip. Permission denied.\n{:?}", e)
            } else {
                panic!("Unexpected error opening lists.zip.\nERror: {:?}", e)
            }
        });

        let mut archive = ZipArchive::new(file)?;

        /* iterate over all files in archive */
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap_or_else(|e| {
                panic!("ZIP failure.\nError: {:?}", e)
            });

            let path = match file.enclosed_name() {
                Some(path ) => "lists/misc/".to_owned() + path.to_str().unwrap(),
                None => { panic!("Failed: no data.") }
            };

            /* Creates json files and copies data to them. */
            let mut out = File::create(&path).unwrap_or_else(|e| {
                if e.kind() == ErrorKind::PermissionDenied {
                    panic!("Unable to create {:?}. Permission denied.\n{:?}", path, e)
                } else {
                    panic!("Unexpected error creating {:?}.\nError: {:?}", path, e)
                }
            });

            io::copy(&mut file, &mut out).unwrap_or_else(|e| {
                if e.kind() == ErrorKind::PermissionDenied {
                    panic!("Unable to copy data to {:?}. Permission denied.\n{:?}", out, e)
                } else if e.kind() == ErrorKind::Interrupted {
                    println!("Data copy operation was interrupted. Retrying.");
                    io::copy(&mut file, &mut out).unwrap_or_else(|e| {
                        panic!("Data copy operation failed after retry. Quitting.\nError: {:?}", e)
                    })
                } else {
                    panic!("Unexpected error during data copy operation.\nFile attempted wrote: {:?}\nError: {:?}", out, e)
                }
            });
        }

        /* Attempts to delete zip */
        if remove_file("lists/misc/lists.zip").is_err() {
            println!("Failed to delete lists.zip. Ignoring.")
        }
        else {
            println!("Removed lists.zip successfully.")
        }

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

            let mut file = File::create(&fname).unwrap_or_else(|e| {
                if e.kind() == ErrorKind::PermissionDenied {
                    panic!("Failed to create {:?}. Permission denied.\n{:?}", fname, e)
                } else { panic!("Unexpected error creating {:?}.\nError: {:?}", fname, e) }
            });
            file.write_all(&bytes).unwrap_or_else(|e| {
                if e.kind() == ErrorKind::PermissionDenied {
                    panic!("Failed to write data. Permission denied.\n{:?}", e)
                } else if e.kind() == ErrorKind::UnexpectedEof {
                    panic!("Failed to write data. Unexpected end of file.\n{:?}", e)
                } else if e.kind() == ErrorKind::NotFound {
                    panic!("Failed to write data. File not found.\n{:?}", e)
                } else {
                    panic!("Unexpected error writing data.\nError: {:?}", e)
                }
            });
            let filecontent = read_to_string(&fname).unwrap();
            let list: List = serde_json::from_str(&filecontent).unwrap();

            lists.push(list);
        }
        Ok(lists)
    }
}