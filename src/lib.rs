// mod server_manager {
use ripunzip::{FilenameFilter, NullProgressReporter, UnzipEngine, UnzipOptions};
use rusdantic::prelude::*;
use std::{
    io::{Cursor, Read},
    path::PathBuf,
};
use strum_macros::EnumString;
use zip::ZipArchive;

#[derive(Debug, PartialEq, EnumString)]
pub enum ServerPlatform {
    Windows(String),
    Linux(String),
    WindowsPreview(String),
    LinuxPreview(String),
}

#[derive(Rusdantic, Debug)]
pub struct DownloadLink {
    #[rusdantic(
        length(min = 3, max = 50),
        validation_alias = "downloadType",
        serialization_alias = "downloadType"
    )]
    pub download_type: String,
    #[rusdantic(
        length(min = 3, max = 200),
        validation_alias = "downloadUrl",
        serialization_alias = "downloadUrl"
    )]
    pub download_url: String,
}

#[derive(Rusdantic, Debug)]
pub struct VersionResult {
    #[rusdantic(nested)]
    pub links: Vec<DownloadLink>,
}

#[derive(Rusdantic, Debug)]
pub struct VersionPayload {
    #[rusdantic(nested)]
    pub result: VersionResult,
}

// pub fn unzip_payload(zip_bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//     let cursor = Cursor::new(zip_bytes);
//     let mut archive = ZipArchive::new(cursor)?;
//     println!("Zip offset: {:?}", archive.offset());
//     println!("Archive Files: {:?}", archive.file_names());

//     for i in 0..archive.len() {
//         let mut file = archive.by_index(i)?;
//         println!("Filename: {:?}", file.name());

//         let mut contents = Vec::new();
//         file.read_to_end(&mut contents)?;

//         println!("Extracted {:?} byte", contents.len());
//     }
//     Ok(())
// }

struct BedrockServerFilesFilter {}

impl BedrockServerFilesFilter {
    const FILES_TO_IGNORE: [&str; 4] = [
        "allowlist.json",
        "permissions.json",
        "server.properties",
        "profanity_filter.wlist",
    ];
}

impl FilenameFilter for BedrockServerFilesFilter {
    fn should_unzip(&self, filename: &str) -> bool {
        !BedrockServerFilesFilter::FILES_TO_IGNORE.contains(&filename)
    }
}

pub fn ripunzip_uri(uri: String) -> Result<(), Box<dyn std::error::Error>> {
    let engine = UnzipEngine::for_uri(uri.as_str(), None, || {
        println!("Failed to unzip URI");
    })
    .expect("Failed to create unzip engine");
    let opts = UnzipOptions {
        output_directory: Some(PathBuf::from("/tmp/bedrock-server-ripunzip/")),
        password: None,
        single_threaded: true,
        filename_filter: Some(Box::new(BedrockServerFilesFilter {})),
        progress_reporter: Box::new(NullProgressReporter),
    };
    engine.unzip(opts)?;
    Ok(())
}
