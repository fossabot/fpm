pub async fn rm(config: &fpm::Config, file: &str, cr: Option<&str>) -> fpm::Result<()> {
    if let Some(cr) = cr {
        let cr = cr.parse::<usize>()?;
        cr_rm(config, file, cr).await
    } else {
        simple_rm(config, file).await
    }
}

async fn simple_rm(config: &fpm::Config, file: &str) -> fpm::Result<()> {
    use itertools::Itertools;

    let mut workspace = config.get_workspace_map().await?;

    if !config
        .get_non_deleted_latest_file_paths()
        .await?
        .iter()
        .any(|(v, _)| v.eq(file))
    {
        return Err(fpm::Error::UsageError {
            message: format!("{} doesn't exists in latest. If added in workspace use `fpm revert {}` command instead", file, file),
        });
    }

    if let Some(workspace_entry) = workspace.get_mut(file) {
        workspace_entry.set_deleted();
        let path = config.root.join(&workspace_entry.filename);
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
    } else {
        return Err(fpm::Error::UsageError {
            message: format!("{} is not present in workspace", file),
        });
    }

    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;

    Ok(())
}

async fn cr_rm(config: &fpm::Config, file: &str, cr: usize) -> fpm::Result<()> {
    use itertools::Itertools;

    let remote_manifest = config.get_remote_manifest(false).await?;
    let file_edit = if let Some(file_edit) = remote_manifest.get(file) {
        file_edit
    } else {
        return Err(fpm::Error::UsageError {
            message: format!("{} is not present in remote manifest.", file,),
        });
    };

    // create delete entry
    let mut deleted_files = fpm::cr::get_deleted_files(config, cr).await?;
    if deleted_files
        .iter()
        .map(|v| &v.filename)
        .contains(&file.to_string())
    {
        return fpm::usage_error(format!("{} is already deleted in CR#{}", file, cr));
    }
    deleted_files.push(fpm::cr::CRDeleted::new(file, file_edit.version));
    fpm::cr::create_deleted_files(config, cr, deleted_files.as_slice()).await?;

    // create workspace entry
    let mut workspace = config.get_workspace_map().await?;
    let deleted_file_path = &config.cr_deleted_file_path(cr);
    if !workspace.contains_key(config.path_without_root(deleted_file_path)?.as_str()) {
        workspace.insert(
            config.path_without_root(deleted_file_path)?,
            fpm::workspace::WorkspaceEntry {
                filename: config.path_without_root(deleted_file_path)?,
                deleted: None,
                version: None,
                cr: Some(cr),
            },
        );
        config
            .write_workspace(workspace.into_values().collect_vec().as_slice())
            .await?;
    }

    Ok(())
}
