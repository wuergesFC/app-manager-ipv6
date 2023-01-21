use std::io::{Read, Write};
use std::path::Path;

use super::tera::convert_app_yml_for_update;
use crate::composegenerator::v4::types::AppYml as AppYmlV4;
use crate::composegenerator::AppYmlFile;
use crate::{composegenerator::load_config, updates::update_app};

use anyhow::{bail, Result};

async fn update_app_yml(path: &Path, include_prerelease: &Option<bool>) -> Result<()> {
    let app_yml = std::fs::File::open(path)?;
    let mut parsed_app_yml = load_config(app_yml)?;
    let changes = update_app(&mut parsed_app_yml, include_prerelease).await?;
    if changes.is_empty() {
        return Ok(());
    }
    match parsed_app_yml {
        crate::composegenerator::AppYmlFile::V4(app_yml) => {
            let writer = std::fs::File::create(path)?;
            serde_yaml::to_writer(writer, &app_yml)?;
        }
        crate::composegenerator::AppYmlFile::V3(app_yml) => {
            let writer = std::fs::File::create(path)?;
            serde_yaml::to_writer(writer, &app_yml)?;
        }
    }
    Ok(())
}

async fn update_app_yml_jinja(path: &Path, include_prerelease: &Option<bool>) -> Result<()> {
    let app_id = path
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let mut app_yml = convert_app_yml_for_update(path, app_id)?;
    let app_definition: AppYmlV4 = serde_yaml::from_str(&app_yml)?;
    let original_version = app_definition.metadata.version.clone();
    let mut app_definition = AppYmlFile::V4(app_definition);
    let replacements = update_app(&mut app_definition, include_prerelease).await?;
    let mut original_app_yml = std::fs::File::open(path)?;
    app_yml = String::new();
    original_app_yml.read_to_string(&mut app_yml)?;
    for (old_img, new_img) in replacements {
        app_yml = app_yml.replace(&old_img, &new_img);
    }
    let AppYmlFile::V4(app_definition) = app_definition else {
        unreachable!();
    };
    app_yml = app_yml.replace(
        &format!("version: {}", original_version),
        &format!("version: {}", app_definition.metadata.version),
    );
    let mut writer = std::fs::File::create(path)?;
    writer.write_all(app_yml.as_bytes())?;
    Ok(())
}

pub async fn update_app_file(path: &Path, include_prerelease: &Option<bool>) -> Result<()> {
    match path
        .extension()
        .expect("File has no extension!")
        .to_str()
        .expect("File extension is not unicode")
    {
        "yml" => update_app_yml(path, include_prerelease).await,
        "jinja" => update_app_yml_jinja(path, include_prerelease).await,
        _ => {
            bail!("App file format not recognized!");
        }
    }
}
