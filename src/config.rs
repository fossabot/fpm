use std::convert::TryInto;

use lazy_static::__Deref;

/// `Config` struct keeps track of a few configuration parameters that is shared with the entire
/// program. It is constructed from the content of `FPM.ftd` file for the package.
///
/// `Config` is created using `Config::read()` method, and should be constructed only once in the
/// `main()` and passed everywhere.
#[derive(Debug, Clone)]
pub struct Config {
    pub package: fpm::Package,
    /// `root` is the package root folder, this is the folder where `FPM.ftd` file is stored.
    ///
    /// Technically the rest of the program can simply call `std::env::current_dir()` and that
    /// is guaranteed to be same as `Config.root`, but `Config.root` is camino path, instead of
    /// std::path::Path, so we can treat `root` as a handy helper.
    ///
    /// A utility that returns camino version of `current_dir()` may be used in future.
    pub root: camino::Utf8PathBuf,

    /// Keeps a track of the package root for a particular config. For a dep2 of a dep1,
    /// this could point to the <original_root>/.packages/
    /// whereas the project root could be at <original_root>/.packages/<dep1_root>
    pub packages_root: camino::Utf8PathBuf,
    /// `original_directory` is the directory from which the `fpm` command was invoked
    ///
    /// During the execution of `fpm`, we change the directory to the package root so the program
    /// can be written with the assumption that they are running from package `root`.
    ///
    /// When printing filenames for users consumption we want to print the paths relative to the
    /// `original_directory`, so we keep track of the original directory.
    pub original_directory: camino::Utf8PathBuf,
}

impl Config {
    /// `build_dir` is where the static built files are stored. `fpm build` command creates this
    /// folder and stores its output here.
    pub fn build_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".build")
    }

    /// history of a fpm package is stored in `.history` folder.
    ///
    /// Current design is wrong, we should move this helper to `fpm::Package` maybe.
    ///
    /// History of a package is considered part of the package, and when a package is downloaded we
    /// have to chose if we want to download its history as well. For now we do not. Eventually in
    /// we will be able to say download the history also for some package.
    ///
    /// ```ftd
    /// -- ftp.dependency: django
    ///  with-history: true
    /// ```
    ///     
    /// `.history` file is created or updated by `fpm sync` command only, no one else should edit
    /// anything in it.
    pub fn history_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".history")
    }

    /// every package's `.history` contains a file `.latest.ftd`. It looks a bit linke this:
    ///
    /// ```ftd
    /// -- import: fpm
    ///
    /// -- fpm.snapshot: FPM.ftd
    /// timestamp: 1638706756293421000
    ///
    /// -- fpm.snapshot: blog.ftd
    /// timestamp: 1638706756293421000
    /// ```
    ///
    /// One `fpm.snapshot` for every file that is currently part of the package.
    pub fn latest_ftd(&self) -> camino::Utf8PathBuf {
        self.root.join(".history/.latest.ftd")
    }

    /// track_dir returns the directory where track files are stored. Tracking information as well
    /// is considered part of a package, but it is not downloaded when a package is downloaded as
    /// a dependency of another package.
    pub fn track_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".tracks")
    }

    /// `is_translation_package()` is a helper to tell you if the current package is a translation
    /// of another package. We may delete this helper soon.
    pub fn is_translation_package(&self) -> bool {
        self.package.translation_of.is_some()
    }

    /// original_path() returns the path of the original package if the current package is a
    /// translation package. it returns the path in `.packages` folder where the
    pub fn original_path(&self) -> fpm::Result<camino::Utf8PathBuf> {
        let o = match self.package.translation_of.as_ref() {
            Some(ref o) => o,
            None => {
                return Err(fpm::Error::UsageError {
                    message: "This package is not a translation package".to_string(),
                });
            }
        };
        match &o.fpm_path {
            Some(fpm_path) => Ok(fpm_path
                .parent()
                .expect("Expect fpm_path parent. Panic!")
                .to_owned()),
            _ => {
                return Err(fpm::Error::UsageError {
                    message: format!("Unable to find `fpm_path` of the package {}", o.name),
                })
            }
        }
    }

    /*/// aliases() returns the list of the available aliases at the package level.
    pub fn aliases(&self) -> fpm::Result<std::collections::BTreeMap<&str, &fpm::Package>> {
        let mut resp = std::collections::BTreeMap::new();
        self.package
            .dependencies
            .iter()
            .filter(|d| d.alias.is_some())
            .for_each(|d| {
                resp.insert(d.alias.as_ref().unwrap().as_str(), &d.package);
            });
        Ok(resp)
    }*/

    /// `get_font_style()` returns the HTML style tag which includes all the fonts used by any
    /// ftd document. Currently this function does not check for fonts in package dependencies
    /// nor it tries to avoid fonts that are configured but not needed in current document.
    pub fn get_font_style(&self) -> String {
        use itertools::Itertools;
        // TODO: accept list of actual fonts used in the current document. each document accepts
        //       a different list of fonts and only fonts used by a given document should be
        //       included in the HTML produced by that font
        // TODO: fetch fonts from package dependencies as well (ideally this function should fail
        //       if one of the fonts used by any ftd document is not found

        let generated_style = self
            .package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec()
            .iter()
            .fold(self.package.get_font_html(), |accumulator, dep| {
                format!(
                    "{pre}\n{new}",
                    pre = accumulator,
                    new = dep.package.get_font_html()
                )
            });
        return match generated_style.trim().is_empty() {
            false => format!("<style>{}</style>", generated_style),
            _ => "".to_string(),
        };
    }

    /// `read()` is the way to read a Config.
    pub async fn read() -> fpm::Result<Config> {
        let original_directory: camino::Utf8PathBuf =
            std::env::current_dir()?.canonicalize()?.try_into()?;
        let root = match find_root_for_file(&original_directory, "FPM.ftd") {
            Some(fpm_ftd_root) => fpm_ftd_root,
            None => {
                // Look for FPM manifest
                match find_root_for_file(&original_directory, "FPM.manifest.ftd") {
                    Some(fpm_manifest_path) => {
                        let doc =
                            tokio::fs::read_to_string(fpm_manifest_path.join("FPM.manifest.ftd"));
                        let lib = fpm::FPMLibrary::default();
                        match ftd::p2::Document::from("FPM.manifest", doc.await?.as_str(), &lib) {
                            Ok(fpm_manifest_processed) => {
                                let k: String =
                                    fpm_manifest_processed.get("FPM.manifest#package-root")?;
                                let new_package_root = k
                                    .as_str()
                                    .split('/')
                                    .fold(fpm_manifest_path, |accumulator, part| {
                                        accumulator.join(part)
                                    });
                                if new_package_root.join("FPM.ftd").exists() {
                                    new_package_root
                                } else {
                                    return Err(fpm::Error::PackageError {
                                        message: "Can't find FPM.ftd. The path specified in FPM.manifest.ftd doesn't contain the FPM.ftd file".to_string(),
                                    });
                                }
                            }
                            Err(e) => {
                                return Err(fpm::Error::PackageError {
                                    message: format!("failed to parse FPM.manifest.ftd: {:?}", &e),
                                });
                            }
                        }
                    }
                    None => {
                        return Err(fpm::Error::UsageError {
                            message:
                                "FPM.ftd or FPM.manifest.ftd not found in any parent directory"
                                    .to_string(),
                        });
                    }
                }
            }
        };
        let b = {
            let doc = tokio::fs::read_to_string(root.join("FPM.ftd"));
            let lib = fpm::FPMLibrary::default();
            match ftd::p2::Document::from("FPM", doc.await?.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::PackageError {
                        message: format!("failed to parse FPM.ftd 3: {:?}", &e),
                    });
                }
            }
        };

        let mut deps = {
            let temp_deps: Vec<fpm::dependency::DependencyTemp> = b.get("fpm#dependency")?;
            temp_deps
                .into_iter()
                .map(|v| v.into_dependency())
                .collect::<Vec<fpm::Result<fpm::Dependency>>>()
                .into_iter()
                .collect::<fpm::Result<Vec<fpm::Dependency>>>()?
        };

        let mut package = {
            let temp_package: Option<PackageTemp> = b.get("fpm#package")?;
            let mut package = match temp_package {
                Some(v) => v.into_package(),
                None => {
                    return Err(fpm::Error::PackageError {
                        message: "FPM.ftd does not contain package definition".to_string(),
                    })
                }
            };

            if package.name != fpm::PACKAGE_INFO_INTERFACE
                && !deps.iter().any(|dep| {
                    dep.implements
                        .contains(&fpm::PACKAGE_INFO_INTERFACE.to_string())
                })
            {
                deps.push(fpm::Dependency {
                    package: fpm::Package::new(fpm::PACKAGE_INFO_INTERFACE),
                    version: None,
                    notes: None,
                    alias: None,
                    implements: Vec::new(),
                });
            };

            package.dependencies = deps;

            let auto_imports: Vec<String> = b.get("fpm#auto-import")?;
            // let mut aliases = std::collections::HashMap::<String, String>::new();
            let auto_import = auto_imports
                .iter()
                .map(|f| fpm::AutoImport::from_string(f.as_str()))
                .collect();
            package.auto_import = auto_import;

            package.ignored_paths = b.get::<Vec<String>>("fpm#ignore")?;
            package.fonts = b.get("fpm#font")?;
            package
        };

        fpm::utils::validate_zip_url(&package)?;

        fpm::dependency::ensure(&root, &mut package)?;
        if package.import_auto_imports_from_original {
            if let Some(ref original_package) = *package.translation_of {
                if !package.auto_import.is_empty() {
                    return Err(fpm::Error::PackageError {
                        message: format!("Can't use `inherit-auto-imports-from-original` along with auto-imports defined for the translation package. Either set `inherit-auto-imports-from-original` to false or remove `fpm.auto-import` from the {package_name}/FPM.ftd file", package_name=package.name.as_str()),
                    });
                } else {
                    package.auto_import = original_package.auto_import.clone()
                }
            }
        }
        Ok(Config {
            package,
            packages_root: root.clone().join(".packages"),
            root,
            original_directory,
        })
    }
}

/// `find_root_for_file()` starts with the given path, which is the current directory where the
/// application started in, and goes up till it finds a folder that contains `FPM.ftd` file.
pub(crate) fn find_root_for_file(
    dir: &camino::Utf8Path,
    file_name: &str,
) -> Option<camino::Utf8PathBuf> {
    if dir.join(file_name).exists() {
        Some(dir.into())
    } else {
        if let Some(p) = dir.parent() {
            return find_root_for_file(p, file_name);
        };
        None
    }
}

/// PackageTemp is a struct that is used for mapping the `fpm.package` data in FPM.ftd file. It is
/// not used elsewhere in program, it is immediately converted to `fpm::Package` struct during
/// deserialization process
#[derive(serde::Deserialize, Debug, Clone)]
pub(crate) struct PackageTemp {
    pub name: String,
    #[serde(rename = "translation-of")]
    pub translation_of: Option<String>,
    #[serde(rename = "translation")]
    pub translations: Vec<String>,
    #[serde(rename = "language")]
    pub language: Option<String>,
    pub about: Option<String>,
    pub zip: Option<String>,
    #[serde(rename = "canonical-url")]
    pub canonical_url: Option<String>,
    #[serde(rename = "inherit-auto-imports-from-original")]
    pub import_auto_imports_from_original: bool,
}

impl PackageTemp {
    pub fn into_package(self) -> fpm::Package {
        // TODO: change this method to: `validate(self) -> fpm::Result<fpm::Package>` and do all
        //       validations in it. Like a package must not have both translation-of and
        //       `translations` set.
        let translation_of = self.translation_of.as_ref().map(|v| fpm::Package::new(v));
        let translations = self
            .translations
            .clone()
            .into_iter()
            .map(|v| fpm::Package::new(&v))
            .collect::<Vec<fpm::Package>>();

        fpm::Package {
            name: self.name,
            translation_of: Box::new(translation_of),
            translations,
            language: self.language,
            about: self.about,
            zip: self.zip,
            translation_status_summary: None,
            canonical_url: self.canonical_url,
            dependencies: vec![],
            auto_import: vec![],
            fpm_path: None,
            ignored_paths: vec![],
            fonts: vec![],
            import_auto_imports_from_original: self.import_auto_imports_from_original,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub translation_of: Box<Option<Package>>,
    pub translations: Vec<Package>,
    pub language: Option<String>,
    pub about: Option<String>,
    pub zip: Option<String>,
    pub translation_status_summary: Option<fpm::translation::TranslationStatusSummary>,
    pub canonical_url: Option<String>,
    /// `dependencies` keeps track of direct dependencies of a given package. This too should be
    /// moved to `fpm::Package` to support recursive dependencies etc.
    pub dependencies: Vec<fpm::Dependency>,
    /// `auto_import` keeps track of the global auto imports in the package.
    pub auto_import: Vec<fpm::AutoImport>,
    pub fpm_path: Option<camino::Utf8PathBuf>,
    /// `ignored` keeps track of files that are to be ignored by `fpm build`, `fpm sync` etc.
    pub ignored_paths: Vec<String>,
    /// `fonts` keeps track of the fonts used by the package.
    ///
    /// Note that this too is kind of bad design, we will move fonts to `fpm::Package` struct soon.
    pub fonts: Vec<fpm::Font>,

    pub import_auto_imports_from_original: bool,
}

impl Package {
    pub fn new(name: &str) -> fpm::Package {
        fpm::Package {
            name: name.to_string(),
            translation_of: Box::new(None),
            translations: vec![],
            language: None,
            about: None,
            zip: None,
            translation_status_summary: None,
            canonical_url: None,
            dependencies: vec![],
            auto_import: vec![],
            fpm_path: None,
            ignored_paths: vec![],
            fonts: vec![],
            import_auto_imports_from_original: true,
        }
    }

    pub fn get_dependency_for_interface(&self, interface: &str) -> Option<&fpm::Dependency> {
        self.dependencies
            .iter()
            .find(|dep| dep.implements.contains(&interface.to_string()))
    }

    pub fn get_flattened_dependencies(&self) -> Vec<fpm::Dependency> {
        self.dependencies
            .clone()
            .into_iter()
            .fold(&mut vec![], |old_val, dep| {
                old_val.extend(dep.package.get_flattened_dependencies());
                old_val.push(dep);
                old_val
            })
            .to_owned()
    }

    pub fn get_font_html(&self) -> String {
        self.fonts.iter().fold(String::new(), |accumulator, font| {
            format!(
                "{pre}{new}\n",
                pre = accumulator,
                new = font.to_html(self.name.as_str())
            )
        })
    }

    pub fn generate_prefix_string(&self, with_alias: bool) -> Option<String> {
        self.auto_import.iter().fold(None, |pre, ai| {
            let mut import_doc_path = ai.path.clone();
            if !with_alias {
                // Check for the aliases and map them to the full path
                for dependency in &self.dependencies {
                    if let Some(alias) = &dependency.alias {
                        if alias.as_str().eq(ai.path.as_str())
                            || ai.path.starts_with(format!("{}/", alias).as_str())
                        {
                            import_doc_path = ai.path.replacen(
                                dependency.alias.as_ref()?.as_str(),
                                dependency.package.name.as_str(),
                                1,
                            );
                        }
                    }
                }
            }
            Some(format!(
                "{}\n-- import: {}{}",
                pre.unwrap_or_else(|| "".to_string()),
                &import_doc_path,
                match &ai.alias {
                    Some(a) => format!(" as {}", a),
                    None => String::new(),
                }
            ))
        })
    }

    pub fn get_prefixed_body(&self, body: &str, id: &str, with_alias: bool) -> String {
        if id.contains("FPM/") {
            return body.to_string();
        };
        match self.generate_prefix_string(with_alias) {
            Some(s) => format!("{}\n\n{}", s.trim(), body),
            None => body.to_string(),
        }
    }

    pub fn eval_auto_import(&self, name: &str) -> Option<&str> {
        for x in &self.auto_import {
            let matching_string = match &x.alias {
                Some(a) => a.as_str(),
                None => x.path.as_str(),
            };
            if matching_string == name {
                return Some(&x.path);
            };
        }
        None
    }

    pub fn generate_canonical_url(&self, path: &str) -> String {
        match &self.canonical_url {
            Some(url) => {
                // Ignore the FPM document as that path won't exist in the reference website
                if !path.starts_with("-/") {
                    format!(
                        "\n<link rel=\"canonical\" href=\"{canonical_base}{path}\" />",
                        canonical_base = url,
                        path = path
                    )
                } else {
                    "".to_string()
                }
            }
            None => "".to_string(),
        }
    }

    /// aliases() returns the list of the available aliases at the package level.
    pub fn aliases(&self) -> fpm::Result<std::collections::BTreeMap<&str, &fpm::Package>> {
        let mut resp = std::collections::BTreeMap::new();
        for d in &self.dependencies {
            if let Some(a) = &d.alias {
                resp.insert(a.as_str(), &d.package);
            }
            resp.insert(&d.package.name, &d.package);
        }
        Ok(resp)
    }

    pub async fn get_assets_doc(&self, config: &fpm::Config) -> fpm::Result<String> {
        // Virtual document that contains the asset information about the package
        use itertools::Itertools;
        let all_docs = fpm::get_documents(config, self).await?;
        let all_file_names = all_docs
            .iter()
            .filter_map(|file_instance| {
                let file_id = file_instance.get_id();
                if file_id.eq("FPM.ftd") {
                    None
                } else {
                    Some(Path::new(file_id.as_str()))
                }
            })
            .collect::<Vec<Path>>();
        let mut top = dir("root");
        for path in all_file_names.iter() {
            build_tree(&mut top, &path.parts, 0);
        }
        let mut all_extensions: Vec<String> = vec![];
        let (resp1, resp2) = build_record_values(&top, &mut all_extensions);
        let extension_records = all_extensions
            .iter()
            .unique()
            .map(|ext| {
                // if fpm::IMAGE_EXT.contains(&ext.as_str()) {
                //     format!("-- record {ext}-file:\nfpm.image-src {ext}:")
                // } else {
                format!("-- record {ext}-file:\nstring {ext}:")
                // }
            })
            .join("\n");
        let (font_record, fonts) = self
            .fonts
            .iter()
            .unique_by(|font| font.name.as_str())
            .collect_vec()
            .iter()
            .fold(
                (
                    String::from("-- record font:"),
                    String::from("-- font fonts:"),
                ),
                |(record_accumulator, instance_accumulator), font| {
                    (
                        format!(
                            "{pre}\nstring {font_var_name}:",
                            pre = record_accumulator,
                            font_var_name = font.name.as_str(),
                        ),
                        format!(
                            "{pre}\n{font_var_name}: {font_var_val}",
                            pre = instance_accumulator,
                            font_var_name = font.name.as_str(),
                            font_var_val = font.html_name(self.name.as_str())
                        ),
                    )
                },
            );
        return Ok(format!(
            indoc::indoc! {"
                -- import: fpm

                {extension_records}\n
                {resp1}\n
                {resp2}
                {font_record}
                {fonts}
            "},
            extension_records = extension_records,
            resp1 = resp1,
            resp2 = resp2,
            font_record = font_record,
            fonts = fonts
        ));

        #[derive(Debug)]
        struct Path {
            parts: Vec<String>,
        }
        impl Path {
            pub fn new(path: &str) -> Path {
                Path {
                    parts: path.to_string().split('/').map(|s| s.to_string()).collect(),
                }
            }
        }

        #[derive(Debug, Clone)]
        struct Dir {
            name: String,
            full_path: String,
            children: Vec<Dir>,
        }

        fn dir(val: &str) -> Dir {
            Dir::new(val, val)
        }

        impl Dir {
            fn new(name: &str, full_path: &str) -> Dir {
                Dir {
                    name: name.to_string(),
                    full_path: full_path.to_string(),
                    children: Vec::<Dir>::new(),
                }
            }

            fn find_child(&mut self, name: &str) -> Option<&mut Dir> {
                for c in self.children.iter_mut() {
                    if c.name == name {
                        return Some(c);
                    }
                }
                None
            }

            fn add_child<T>(&mut self, leaf: T) -> &mut Self
            where
                T: Into<Dir>,
            {
                self.children.push(leaf.into());
                self
            }

            fn is_leaf(&self) -> bool {
                self.children.is_empty()
            }

            fn full_path_to_key(&self) -> String {
                let path = self.full_path.as_str().trim_start_matches('.').trim();
                path.chars()
                    .map(|x| match x {
                        '/' => '-',
                        '.' => '-',
                        '_' => '-',
                        _ => x,
                    })
                    .collect()
            }
        }

        fn build_tree(node: &mut Dir, parts: &[String], depth: usize) {
            if depth < parts.len() {
                let item = &parts[depth];
                let full_path = &parts[..depth + 1].join("/");

                let dir = match node.find_child(item) {
                    Some(d) => d,
                    None => {
                        let d = Dir::new(item, full_path.as_str());
                        node.add_child(d);
                        match node.find_child(item) {
                            Some(d2) => d2,
                            None => panic!("Got here!"),
                        }
                    }
                };
                build_tree(dir, parts, depth + 1);
            }
        }

        fn build_record_values(node: &Dir, found_extensions: &mut Vec<String>) -> (String, String) {
            let mut resp_records = String::new();
            let mut resp_values = String::new();
            for child_node in node.children.iter() {
                let child_node = child_node.deref();
                let child_full_path = child_node.full_path.clone();
                let child_key = child_node.full_path_to_key();
                if !child_node.is_leaf() {
                    let (new_records, new_values) = child_node.children.iter().fold(
                        (
                            format!(
                                "-- record {key}-folder:",
                                key = child_node.full_path_to_key()
                            ),
                            format!(
                                "-- {key}-folder {key}:",
                                key = child_node.full_path_to_key(),
                            ),
                        ),
                        |(child_record_accumulator, child_value_accumulator), sub_child_ins| {
                            let sub_child_ins = sub_child_ins.deref();
                            let (append_record, append_value) = if sub_child_ins.is_leaf() {
                                let (sub_child_file_name, sub_child_file_ext) =
                                    process_leaf_node(sub_child_ins);
                                if let Some(ext) = sub_child_file_ext {
                                    (
                                        format!("{ext}-file {sub_child_file_name}:"),
                                        format!(
                                            "{sub_child_file_name}: ${sub_child_key}",
                                            sub_child_key = sub_child_ins.full_path_to_key()
                                        ),
                                    )
                                } else {
                                    (
                                        format!("string {sub_child_file_name}:"),
                                        format!(
                                            "{sub_child_file_name}: {sub_child_path}",
                                            sub_child_path = sub_child_ins.full_path
                                        ),
                                    )
                                }
                            } else {
                                (
                                    format!(
                                        "{sub_child_key}-folder {sub_child_name}:",
                                        sub_child_key = sub_child_ins.full_path_to_key(),
                                        sub_child_name = sub_child_ins.name.as_str()
                                    ),
                                    format!(
                                        "{sub_child_name}: ${sub_child_key}",
                                        sub_child_key = sub_child_ins.full_path_to_key(),
                                        sub_child_name = sub_child_ins.name.as_str()
                                    ),
                                )
                            };
                            (
                                format!("{child_record_accumulator}\n{append_record}"),
                                format!("{child_value_accumulator}\n{append_value}"),
                            )
                        },
                    );

                    let (child_records, child_values) =
                        build_record_values(child_node, found_extensions);

                    resp_records = format!("{resp_records}\n{child_records}\n{new_records}");
                    resp_values = format!("{resp_values}\n{child_values}\n{new_values}");
                } else {
                    let (file_name, file_ext) = process_leaf_node(child_node);
                    let new_values = if let Some(ext) = file_ext {
                        found_extensions.push(ext.to_string());
                        // Found the extension. Should be appended to the parent record
                        format!("-- {ext}-file {child_key}:\n{ext}: {child_full_path}",)
                    } else {
                        // Extension not found. Will be appended directly as a string
                        format!("-- string {file_name}: {child_full_path}")
                    };
                    resp_values = format!("{resp_values}\n{new_values}");
                }
            }
            (resp_records, resp_values)
        }

        fn process_leaf_node(node: &Dir) -> (&str, Option<&str>) {
            if let Some((name, ext)) = node.name.as_str().rsplit_once('.') {
                let name = if name.is_empty() { "DOT" } else { name };
                (name, Some(ext))
            } else {
                // File without extension
                (node.name.as_str(), None)
            }
        }
    }
}
