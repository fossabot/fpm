#[derive(serde::Deserialize, Debug, Clone)]
pub struct Track {
    pub filename: String,
    pub package: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "other-timestamp")]
    pub other_timestamp: Option<u128>,
    #[serde(rename = "self-timestamp")]
    pub self_timestamp: u128,
    #[serde(rename = "last-merged-version")]
    pub last_merged_version: Option<u128>,
}

pub(crate) fn get_tracks(
    base_path: &str,
    path: &camino::Utf8PathBuf,
) -> fpm::Result<std::collections::BTreeMap<String, Track>> {
    let mut tracks = std::collections::BTreeMap::new();
    if !path.exists() {
        return Ok(tracks);
    }

    let lib = fpm::FPMLibrary::default();
    let doc = std::fs::read_to_string(path)?;
    let b = match fpm::doc::parse_ftd(base_path, doc.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", base_path, &e);
            todo!();
        }
    };
    let track_list: Vec<Track> = b.get("fpm#track")?;
    for track in track_list {
        tracks.insert(track.filename.to_string(), track);
    }
    Ok(tracks)
}
