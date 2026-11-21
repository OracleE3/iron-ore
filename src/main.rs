use regex::Regex;

use server_manager::{ripunzip_uri, DownloadLink, VersionPayload};

fn main() {
    let bedrock_server_url: &str =
        "https://net-secondary.web.minecraft-services.net/api/v1.0/download/links";
    let body: String = reqwest::blocking::get(bedrock_server_url)
        .unwrap()
        .text()
        .expect("Failed to get payload from Minecraft server API");
    println!("body = {body:?}");
    let payload: VersionPayload = rusdantic::from_json(body.as_str())
        .expect("Failed to deserialize Minecraft server versions payload");
    let links: Vec<DownloadLink> = payload.result.links;
    println!("Links: {:?}", links);
    let mut correct_download: Option<String> = None;
    for link in links {
        if !link.download_type.starts_with("serverBedrock") {
            continue;
        }
        if link
            .download_type
            .replace("serverBedrock", "")
            .to_lowercase()
            == std::env::consts::OS
        {
            correct_download = Some(link.download_url.clone());
        }
    }
    println!("Current OS: {:?}", std::env::consts::OS);
    println!("Correct download: {:?}", correct_download);
    let latest_download_re = Regex::new(r"https?://[a-zA-Z0-9\-\./]+/bin-(?<os>\w+)/bedrock-server-(?<version>[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+).zip").unwrap();
    let captures = latest_download_re
        .captures(body.as_str())
        .expect("Can't parse download link!");
    let version =
        lenient_semver::parse(&captures["version"]).expect("Failed to parse server version");
    println!("Version: {:?}", version);
    let correct_dl_str = correct_download.unwrap();
    let server_zip_filename = correct_dl_str[correct_dl_str
        .rfind("/")
        .expect("Can't find '/' in download str")..]
        .to_string();
    let server_zip: bytes::Bytes = reqwest::blocking::get(correct_dl_str.clone())
        .expect("Failed to get payload from Minecraft server API")
        .bytes()
        .expect("Failed to get bytes from Minecraft server download");
    // .collect::<Result<Vec<u8>>>()
    // .expect("Failed to collect bytes from payload into Vec")
    std::fs::write(format!("/tmp/{}", server_zip_filename), server_zip.as_ref())
        .expect("Failed to write bytes to file");
    // unzip_payload(server_zip.as_ref()).expect("Failed to unzip server");
    ripunzip_uri(correct_dl_str, false).expect("Failed to unzip from network stream");
}
