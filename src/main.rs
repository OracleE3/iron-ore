use regex::Regex;

use server_manager::{DownloadLink, VersionPayload};

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
    // let latest_download_re = Regex::new(r"https?://[a-zA-Z0-9\-\./]+/bin-(?<os>\w+)/bedrock-server-(?<version>[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+).zip").unwrap();
    // let Some(results) = latest_download_re.captures(body.as_str()) else {
    //     println!("Can't find download link!");
    //     return;
    // };
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
}
