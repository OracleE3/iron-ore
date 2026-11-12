// mod server_manager {
use rusdantic::prelude::*;
use strum_macros::EnumString;

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
