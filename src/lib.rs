// mod server_manager {
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use lenient_semver::parse;
use regex::Regex;
use reqwest::Error;
use ripunzip::{
    FilenameFilter, NullProgressReporter, UnzipEngine, UnzipOptions, UnzipProgressReporter,
};
use rusdantic::prelude::*;
use semver::Version;
use std::{
    collections::HashMap,
    fmt::Write,
    io::{Cursor, Read},
    path::PathBuf,
    sync::RwLock,
};
use strum_macros::{Display, EnumString};

#[derive(Debug, Display, PartialEq, EnumString)]
pub enum ServerPlatform {
    #[strum(serialize = "Windows")]
    Windows,
    #[strum(serialize = "Linux")]
    Linux,
    #[strum(serialize = "PreviewWindows")]
    WindowsPreview,
    #[strum(serialize = "PreviewLinux")]
    LinuxPreview,
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

pub fn load_version_list_from_github(
    url: &str,
    platform: ServerPlatform,
) -> Result<HashMap<String, String>, Error> {
    let version_url_re = Regex::new(r"\* \[(?<version>(\d+\.){3}\d+)\]\((?<url>[\w:/\\\.-]+)\)")
        .expect("Failed to compile regex");
    let platform_section_start: String = format!(
        "{}-RELEASES:START",
        platform.to_string().to_ascii_uppercase()
    );
    let platform_section_end: String =
        format!("{}-RELEASES:END", platform.to_string().to_ascii_uppercase());
    let mut versions: HashMap<String, String> = HashMap::new();
    let releases_list: String = reqwest::blocking::get(url)?
        .text()
        .expect(format!("Failed to get payload from community source '{}'", url).as_str());
    for l in releases_list
        .split("\n")
        .skip_while(|l| !l.contains(platform_section_start.as_str()))
    {
        if l.is_empty() {
            continue;
        } else if l.contains(platform_section_start.as_str()) {
            continue;
        } else if l.contains(platform_section_end.as_str()) {
            break;
        }
        if let Some(captures) = version_url_re.captures(l) {
            // let ver: Version = parse(&captures["version"]).expect("Failed to parse server version");
            versions.insert(captures["version"].to_string(), captures["url"].to_string());
        } else {
            println!("Line '{}' does not match regex '{:?}'", l, version_url_re);
            continue;
        }
    }
    println!(
        "Got versions: {}",
        serde_json::to_string_pretty(&versions).expect("Failed to serialize versions")
    );
    Ok(versions)
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

////////////////////////////////////////////////////////////////////////////////////////////////
// Stolen from https://github.com/GoogleChrome/ripunzip/blob/main/src/main.rs#L209
pub struct ProgressDisplayer(ProgressBar);

impl ProgressDisplayer {
    fn new() -> Self {
        Self(ProgressBar::new(0))
    }
}

impl UnzipProgressReporter for ProgressDisplayer {
    fn extraction_starting(&self, display_name: &str) {
        self.0.set_message(format!("Extracting {display_name}"))
    }

    fn total_bytes_expected(&self, expected: u64) {
        self.0.set_length(expected);
        self.0.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})\n{msg}")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#-"));
    }

    fn bytes_extracted(&self, count: u64) {
        self.0.inc(count)
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////

pub fn ripunzip_uri(
    uri: String,
    output_dir: String,
    silent: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let engine = UnzipEngine::for_uri(uri.as_str(), None, || {
        println!("Failed to unzip URI");
    })
    .expect("Failed to create unzip engine");
    let progress_reporter: Box<dyn UnzipProgressReporter + Sync> = if silent {
        Box::new(NullProgressReporter)
    } else {
        Box::new(ProgressDisplayer::new())
    };
    let opts = UnzipOptions {
        output_directory: Some(PathBuf::from(output_dir)),
        password: None,
        single_threaded: true,
        filename_filter: Some(Box::new(BedrockServerFilesFilter {})),
        progress_reporter: progress_reporter,
    };
    engine.unzip(opts)?;
    Ok(())
}
