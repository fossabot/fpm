use crate::utils::HasElements;

fn construct_fpm_ui(lib: &fpm::Library) -> String {
    let lang = match lib.config.package.language {
        Some(ref lang) => {
            realm_lang::Language::from_2_letter_code(lang).unwrap_or(realm_lang::Language::English)
        }
        None => realm_lang::Language::English,
    };

    let primary_lang = match lib.config.package.translation_of.as_ref() {
        Some(ref package) => match package.language {
            Some(ref lang) => realm_lang::Language::from_2_letter_code(lang)
                .unwrap_or(realm_lang::Language::English),
            None => lang,
        },
        None => lang,
    };

    let current_document_last_modified_on = futures::executor::block_on(
        fpm::utils::get_current_document_last_modified_on(&lib.config, lib.document_id.as_str()),
    );

    format!(
        indoc::indoc! {"
            -- ui-data ui:
            current-language: {current_language}
            document: {document}
            language-detail-page-body: {language_detail_page_body}
            language-detail-page: {language_detail_page}
            language: {language}
            last-modified-on: {last_modified_on}
            missing: {missing}
            never-marked: {never_marked}
            never-synced: {never_synced}
            other-available-languages: {other_available_languages}
            out-dated-body: {out_dated_body}
            out-dated-heading: {out_dated_heading}
            out-dated: {out_dated}
            show-latest-version: {show_latest_version}
            show-outdated-version: {show_outdated_version}
            show-translation-status: {show_translation_status}
            show-unapproved-version: {show_unapproved_version}
            status: {status}
            total-number-of-documents: {total_number_of_documents}
            translation-not-available: {translation_not_available}
            unapproved-heading: {unapproved_heading}
            upto-date: {upto_date}
            welcome-fpm-page-subtitle: {welcome_fpm_page_subtitle}
            welcome-fpm-page: {welcome_fpm_page}
        "},
        current_language = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "current-language",
            &current_document_last_modified_on
        ),
        document = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "document",
            &current_document_last_modified_on
        ),
        language = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "language",
            &current_document_last_modified_on
        ),
        language_detail_page = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "language-detail-page",
            &current_document_last_modified_on
        ),
        language_detail_page_body = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "language-detail-page-body",
            &current_document_last_modified_on
        ),
        last_modified_on = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "last-modified-on",
            &current_document_last_modified_on
        ),
        never_synced = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "never-synced",
            &current_document_last_modified_on
        ),
        missing = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "missing",
            &current_document_last_modified_on
        ),
        never_marked = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "never-marked",
            &current_document_last_modified_on
        ),
        other_available_languages = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "other-available-languages",
            &current_document_last_modified_on
        ),
        out_dated = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "out-dated",
            &current_document_last_modified_on
        ),
        out_dated_body = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "out-dated-body",
            &current_document_last_modified_on
        ),
        out_dated_heading = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "out-dated-heading",
            &current_document_last_modified_on
        ),
        show_latest_version = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "show-latest-version",
            &current_document_last_modified_on
        ),
        show_outdated_version = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "show-outdated-version",
            &current_document_last_modified_on
        ),
        show_translation_status = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "show-translation-status",
            &current_document_last_modified_on
        ),
        show_unapproved_version = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "show-unapproved-version",
            &current_document_last_modified_on
        ),
        status = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "status",
            &current_document_last_modified_on
        ),
        total_number_of_documents = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "total-number-of-documents",
            &current_document_last_modified_on
        ),
        translation_not_available = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "translation-not-available",
            &current_document_last_modified_on
        ),
        unapproved_heading = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "unapproved-heading",
            &current_document_last_modified_on
        ),
        upto_date = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "upto-date",
            &current_document_last_modified_on
        ),
        welcome_fpm_page = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "welcome-fpm-page",
            &current_document_last_modified_on
        ),
        welcome_fpm_page_subtitle = fpm::i18n::translation::search(
            &lang,
            &primary_lang,
            "welcome-fpm-page-subtitle",
            &current_document_last_modified_on
        ),
    )
}

pub(crate) fn get(lib: &fpm::Library) -> String {
    let mut fpm_base = format!(
        indoc::indoc! {"
            {fpm_base}
            {fpm_ui}
            -- string document-id: {document_id}
            -- string translation-status-url: {home_url}
            -- string title: {title}
            -- string package-name: {package_name}
            -- optional string package-zip:
            -- string home-url: {home_url}
        "},
        fpm_base = fpm::fpm_ftd(),
        fpm_ui = construct_fpm_ui(lib),
        document_id = lib.document_id,
        title = lib.config.package.name,
        package_name = lib.config.package.name,
        home_url = format!("//{}", lib.config.package.name),
    );

    if lib.config.package.translation_of.is_some() {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- is-translation-package: true
            "},
            fpm_base = fpm_base,
        );
    }

    if lib.config.package.translations.has_elements() {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- has-translations: true
            "},
            fpm_base = fpm_base,
        );
    }

    if let Some(ref zip) = lib.config.package.zip {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- package-zip: {package_zip}
            "},
            fpm_base = fpm_base,
            package_zip = zip,
        );
    }

    if let Some(ref diff) = lib.translated_data.diff {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- diff: 
                
                {diff}    
            "},
            fpm_base = fpm_base,
            diff = diff,
        );
    }

    if let Some(ref status) = lib.translated_data.status {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- translation-status: 
                
                {translation_status}    
            "},
            fpm_base = fpm_base,
            translation_status = status,
        );
    }

    if lib.config.package.translation_of.is_some() || lib.config.package.translations.has_elements()
    {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- translation-status-url: //{package_name}/FPM/translation-status    
            "},
            fpm_base = fpm_base,
            package_name = lib.config.package.name,
        );
    }

    if let Ok(number_of_documents) =
        futures::executor::block_on(fpm::utils::get_number_of_documents(&lib.config))
    {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- number-of-documents: {number_of_documents}    
            "},
            fpm_base = fpm_base,
            number_of_documents = number_of_documents,
        );
    }

    if let Some(last_modified_on) =
        futures::executor::block_on(fpm::utils::get_last_modified_on(&lib.config.root))
    {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- last-modified-on: {last_modified_on}    
            "},
            fpm_base = fpm_base,
            last_modified_on = last_modified_on,
        );
    }

    if let Some(last_modified_on) = futures::executor::block_on(
        fpm::utils::get_current_document_last_modified_on(&lib.config, lib.document_id.as_str()),
    ) {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- current-document-last-modified-on: {last_modified_on}    
            "},
            fpm_base = fpm_base,
            last_modified_on = last_modified_on,
        );
    }

    if let Some(ref language) = lib.config.package.language {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- language: {language}     
            "},
            fpm_base = fpm_base,
            language = fpm::utils::language_to_human(language),
        );
    }

    if let Some(ref last_marked_on) = lib.translated_data.last_marked_on {
        let rfc3339 = fpm::utils::nanos_to_rfc3339(last_marked_on);
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- last-marked-on: {last_marked_on}
                -- last-marked-on-rfc3339: {rfc3339}    
            "},
            fpm_base = fpm_base,
            last_marked_on = last_marked_on,
            rfc3339 = rfc3339,
        );
    }
    if let Some(ref original_latest) = lib.translated_data.original_latest {
        let rfc3339 = fpm::utils::nanos_to_rfc3339(original_latest);
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- original-latest: {original_latest}
                -- original-latest-rfc3339: {rfc3339}    
            "},
            fpm_base = fpm_base,
            original_latest = original_latest,
            rfc3339 = rfc3339,
        );
    }
    if let Some(ref translated_latest) = lib.translated_data.translated_latest {
        let rfc3339 = fpm::utils::nanos_to_rfc3339(translated_latest);
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                        
                -- translated-latest: {translated_latest}
                -- translated-latest-rfc3339: {rfc3339}    
            "},
            fpm_base = fpm_base,
            translated_latest = translated_latest,
            rfc3339 = rfc3339,
        );
    }
    if let Some((filename, content)) = lib.markdown.as_ref() {
        fpm_base = format!(
            indoc::indoc! {"
                {fpm_base}
                
                -- string markdown-filename: {filename}                        
                -- string markdown-content:
    
                {content}
            "},
            fpm_base = fpm_base,
            filename = filename,
            content = content,
        );
    }

    if let Ok(original_path) = lib.config.original_path() {
        if let Ok(original_snapshots) =
            futures::executor::block_on(fpm::snapshot::get_latest_snapshots(&original_path))
        {
            if let Ok(translation_status) =
                fpm::commands::translation_status::get_translation_status(
                    &original_snapshots,
                    &lib.config.root,
                )
            {
                let mut never_marked_files = "".to_string();
                let mut missing_files = "".to_string();
                let mut outdated_files = "".to_string();
                let mut upto_date_files = "".to_string();
                let mut translation_status_list = "".to_string();

                for (file, status) in translation_status.iter() {
                    translation_status_list = format!(
                        indoc::indoc! {"
                            {list}
                            
                            -- status:
                            file: {file}
                            status: {status}                                    
                        "},
                        list = translation_status_list,
                        file = file,
                        status = status.as_str()
                    );

                    match status {
                        fpm::commands::translation_status::TranslationStatus::Missing => {
                            missing_files = format!(
                                indoc::indoc! {"
                                    {list}
                                    
                                    -- missing-files: {file}                                            
                                "},
                                list = missing_files,
                                file = file,
                            );
                        }
                        fpm::commands::translation_status::TranslationStatus::NeverMarked => {
                            never_marked_files = format!(
                                indoc::indoc! {"
                                            {list}
                                            
                                            -- never-marked-files: {file}
                                            
                                        "},
                                list = never_marked_files,
                                file = file,
                            );
                        }
                        fpm::commands::translation_status::TranslationStatus::Outdated => {
                            outdated_files = format!(
                                indoc::indoc! {"
                                            {list}
                                            
                                            -- outdated-files: {file}
                                            
                                        "},
                                list = outdated_files,
                                file = file,
                            );
                        }
                        fpm::commands::translation_status::TranslationStatus::UptoDate => {
                            upto_date_files = format!(
                                indoc::indoc! {"
                                            {list}
                                            
                                            -- upto-date-files: {file}
                                            
                                        "},
                                list = upto_date_files,
                                file = file,
                            );
                        }
                    }
                }

                fpm_base = format!(
                    indoc::indoc! {"
                                {fpm_base}
                                
                                -- record status-data:
                                string file:
                                string status:
                                
                                -- status-data list status:
        
                                {translation_status_list}

                                -- string list missing-files:
                                
                                {missing_files}

                                -- string list never-marked-files:
                                
                                {never_marked_files}

                                -- string list outdated-files:
                                
                                {outdated_files}

                                -- string list upto-date-files:
                                
                                {upto_date_files}

                                
                            "},
                    fpm_base = fpm_base,
                    translation_status_list = translation_status_list,
                    missing_files = missing_files,
                    never_marked_files = never_marked_files,
                    outdated_files = outdated_files,
                    upto_date_files = upto_date_files
                );
            }
        }
    }

    if lib.config.package.translations.has_elements() {
        let mut translation_status_list = "".to_string();
        for translation in lib.config.package.translations.iter() {
            if let Some(ref status) = translation.translation_status_summary {
                if let Some(ref language) = translation.language {
                    let url = format!("https://{}/FPM/translation-status/", translation.name);
                    let status = {
                        let mut status_data = format!(
                            indoc::indoc! {"
                                        -- status:
                                        language: {language}
                                        url: {url}
                                        never-marked: {never_marked}
                                        missing: {missing}
                                        out-dated: {out_dated}
                                        upto-date: {upto_date}
                                    "},
                            language = language,
                            url = url,
                            never_marked = status.never_marked,
                            missing = status.missing,
                            out_dated = status.out_dated,
                            upto_date = status.upto_date
                        );
                        if let Some(ref last_modified_on) = status.last_modified_on {
                            status_data = format!(
                                indoc::indoc! {"
                                            {status}last-modified-on: {last_modified_on}
                                        "},
                                status = status_data,
                                last_modified_on = last_modified_on
                            );
                        }
                        status_data
                    };
                    translation_status_list = format!(
                        indoc::indoc! {"
                                    {list}
                                    
                                    {status}
                                    
                                "},
                        list = translation_status_list,
                        status = status
                    );
                }
            }
        }

        fpm_base = format!(
            indoc::indoc! {"
                        {fpm_base}
                        
                        -- record status-data:
                        string language:
                        string url:
                        integer never-marked:
                        integer missing:
                        integer out-dated:
                        integer upto-date:
                        optional string last-modified-on:
                        
                        -- status-data list status:

                        {translation_status_list}
                        
                    "},
            fpm_base = fpm_base,
            translation_status_list = translation_status_list
        );
    }

    let other_language_packages =
        if let Some(translation_of) = lib.config.package.translation_of.as_ref() {
            let mut other_language_packages = translation_of
                .translations
                .iter()
                .collect::<Vec<&fpm::Package>>();
            other_language_packages.insert(0, translation_of);
            other_language_packages
        } else {
            lib.config
                .package
                .translations
                .iter()
                .collect::<Vec<&fpm::Package>>()
        };

    if other_language_packages.has_elements() {
        let mut languages = "".to_string();
        let doc_id = if lib.document_id.eq("index.ftd") {
            "".to_string()
        } else {
            lib.document_id.replace(".ftd", "/")
        };

        for lang_package in other_language_packages {
            let language = if let Some(ref lang) = lang_package.language {
                fpm::utils::language_to_human(lang)
            } else {
                continue;
            };

            let domain = if lang_package.name.ends_with('/') {
                format!("https://{}{}", lang_package.name, doc_id)
            } else {
                format!("https://{}/{}", lang_package.name, doc_id)
            };

            languages = format!(
                indoc::indoc! {"
                    {languages}
                    - {language}
                      url: {domain}
                "},
                languages = languages,
                domain = domain,
                language = language,
            );
        }

        if !languages.trim().is_empty() {
            fpm_base = format!(
                indoc::indoc! {"
                        {fpm_base}
                        
                        -- language-toc:
                        $processor$: toc

                        {languages}

                        "},
                fpm_base = fpm_base,
                languages = languages,
            );
        }
    }

    fpm_base
}