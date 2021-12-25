pub(crate) enum TranslatedDocument {
    Missing {
        original: fpm::File,
    },
    NeverMarked {
        original: fpm::File,   // main
        translated: fpm::File, // fallback
    },
    Outdated {
        original: fpm::File,   // fallback
        translated: fpm::File, // main
        last_marked_on: u128,
        original_latest: u128,
        translated_latest: u128,
    },
    UptoDate {
        translated: fpm::File,
    },
}

#[derive(Debug, Default)]
pub struct TranslationData {
    pub diff: Option<String>,
    pub last_marked_on: Option<u128>,
    pub original_latest: Option<u128>,
    pub translated_latest: Option<u128>,
}

impl TranslatedDocument {
    pub async fn html(&self, config: &fpm::Config, base_url: Option<&str>) -> fpm::Result<()> {
        // handle the message
        // render with-fallback or with-message
        let message = fpm::get_messages(self, config)?;
        let (main, fallback, translated_data) = match self {
            TranslatedDocument::Missing { original } => {
                (original, None, TranslationData::default())
            }
            TranslatedDocument::NeverMarked {
                original,
                translated,
            } => (original, Some(translated), TranslationData::default()),
            TranslatedDocument::Outdated {
                original,
                translated,
                last_marked_on,
                original_latest,
                translated_latest,
            } => {
                // Gets the diff on original file between last_marked_on and original_latest timestamp
                let diff = get_diff(config, original, last_marked_on, original_latest).await?;
                let translated_data = TranslationData {
                    diff: Some(diff),
                    last_marked_on: Some(*last_marked_on),
                    original_latest: Some(*original_latest),
                    translated_latest: Some(*translated_latest),
                };

                (translated, Some(original), translated_data)
            }
            TranslatedDocument::UptoDate { translated, .. } => {
                (translated, None, TranslationData::default())
            }
        };
        fpm::process_file(
            config,
            main,
            fallback,
            Some(message.as_str()),
            translated_data,
            base_url,
        )
        .await?;
        return Ok(());

        /// Gets the diff on original file between last_marked_on and original_latest timestamp
        async fn get_diff(
            config: &fpm::Config,
            original: &fpm::File,
            last_marked_on: &u128,
            original_latest: &u128,
        ) -> fpm::Result<String> {
            let last_marked_on_path = fpm::utils::history_path(
                original.get_id().as_str(),
                config.original_path()?.as_str(),
                last_marked_on,
            );
            let last_marked_on_data = tokio::fs::read_to_string(last_marked_on_path).await?;
            let original_latest_path = fpm::utils::history_path(
                original.get_id().as_str(),
                config.original_path()?.as_str(),
                original_latest,
            );
            let original_latest_data = tokio::fs::read_to_string(original_latest_path).await?;

            let patch = diffy::create_patch(&last_marked_on_data, &original_latest_data);
            Ok(patch.to_string().replace("---", "\\---"))
        }
    }

    pub async fn get_translated_document(
        config: &fpm::Config,
        original_documents: std::collections::BTreeMap<String, fpm::File>,
        translated_documents: std::collections::BTreeMap<String, fpm::File>,
    ) -> fpm::Result<std::collections::BTreeMap<String, TranslatedDocument>> {
        let original_snapshots =
            fpm::snapshot::get_latest_snapshots(&config.original_path()?).await?;
        let mut translation_status = std::collections::BTreeMap::new();
        for (file, timestamp) in original_snapshots {
            let original_document =
                if let Some(original_document) = original_documents.get(file.as_str()) {
                    original_document
                } else if file.eq("README.md") {
                    original_documents
                        .get("index.md")
                        .ok_or(fpm::Error::PackageError {
                            message: format!("Could not find `{}`", file),
                        })?
                } else {
                    return Err(fpm::Error::PackageError {
                        message: format!("Could not find `{}`", file),
                    });
                };
            if !translated_documents.contains_key(&file) {
                translation_status.insert(
                    file,
                    TranslatedDocument::Missing {
                        original: original_document.clone(),
                    },
                );
                continue;
            }
            let translated_document = translated_documents.get(file.as_str()).unwrap();
            let track_path = fpm::utils::track_path(file.as_str(), config.root.as_str());
            if !track_path.exists() {
                translation_status.insert(
                    file,
                    TranslatedDocument::NeverMarked {
                        original: original_document.clone(),
                        translated: translated_document.clone(),
                    },
                );
                continue;
            }
            let tracks = fpm::tracker::get_tracks(config.root.as_str(), &track_path)?;
            if let Some(fpm::Track {
                last_merged_version: Some(last_merged_version),
                self_timestamp,
                ..
            }) = tracks.get(&file)
            {
                if last_merged_version < &timestamp {
                    translation_status.insert(
                        file,
                        TranslatedDocument::Outdated {
                            original: original_document.clone(),
                            translated: translated_document.clone(),
                            last_marked_on: *last_merged_version,
                            original_latest: timestamp,
                            translated_latest: *self_timestamp,
                        },
                    );
                    continue;
                }
                translation_status.insert(
                    file,
                    TranslatedDocument::UptoDate {
                        translated: translated_document.clone(),
                    },
                );
            } else {
                translation_status.insert(
                    file,
                    TranslatedDocument::NeverMarked {
                        original: original_document.clone(),
                        translated: translated_document.clone(),
                    },
                );
            }
        }
        Ok(translation_status)
    }
}