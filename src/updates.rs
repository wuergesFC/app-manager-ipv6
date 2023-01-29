use std::collections::HashMap;

use crate::composegenerator::{
    v3::update::update_container as update_container_v3,
    v4::update::update_container as update_container_v4, AppYmlFile,
};
use crate::github::get_repo_path;
use crate::hosted_git::check_updates;
use anyhow::{bail, Result};

pub async fn update_app(
    app: &mut AppYmlFile,
    include_pre: &Option<bool>,
) -> Result<HashMap<String, String>> {
    let mut updated_services = HashMap::new();
    let docker = bollard::Docker::connect_with_local_defaults().unwrap();
    match app {
        AppYmlFile::V4(app) => {
            let update_containers = app
                .metadata
                .update_containers
                .clone()
                .unwrap_or_else(|| vec!["main".to_string(), "web".to_string()]);
            let latest_tag = check_updates(&app.metadata, include_pre, None).await;
            if let Err(error) = latest_tag {
                if format!("{}", error) == "No update found" {
                    return Ok(HashMap::new());
                } else {
                    bail!("Failed to get latest release: {}", error);
                }
            }
            let latest_tag = latest_tag.unwrap();

            for (name, service) in app.services.iter_mut() {
                if !update_containers.contains(name) {
                    continue;
                }
                let original_img = service.image.clone();
                let update_result = update_container_v4(service, &latest_tag, &docker).await;
                if let Err(error) = update_result {
                    bail!(error);
                }
                if original_img != service.image {
                    updated_services.insert(original_img, service.image.clone());
                }
            }
            app.metadata.version = latest_tag;
            Ok(updated_services)
        }
        AppYmlFile::V3(app) => {
            let update_containers = vec!["main", "web"];
            let repo = match &app.metadata.repo {
                crate::composegenerator::v3::types::RepoDefinition::RepoUrl(url) => {
                    get_repo_path(url)
                }
                crate::composegenerator::v3::types::RepoDefinition::MultiRepo(map) => {
                    get_repo_path(map.values().next().unwrap())
                }
            };
            if repo.is_none() {
                bail!("Could not parse repo path");
            }
            let current_version = app
                .metadata
                .version
                .strip_prefix('v')
                .unwrap_or(&app.metadata.version);
            let current_version = semver::Version::parse(current_version);
            if current_version.is_err() {
                bail!("Could not parse current version");
            }
            let current_version = current_version.unwrap();
            let (owner, repo) = repo.unwrap();
            let include_pre = include_pre.unwrap_or_else(|| !current_version.pre.is_empty());
            let latest_tag =
                crate::github::check_updates(&owner, &repo, &current_version, include_pre).await;
            if let Err(error) = latest_tag {
                bail!("Failed to get latest release: {}", error);
            }
            let latest_tag = latest_tag.unwrap();

            for service in app.containers.iter_mut() {
                if !update_containers.contains(&service.name.as_str()) {
                    continue;
                }
                let original_img = service.image.clone();
                let update_result = update_container_v3(service, &latest_tag, &docker).await;
                if let Err(error) = update_result {
                    bail!("{}", error);
                }
                if original_img != service.image {
                    updated_services.insert(original_img, service.image.clone());
                }
            }
            app.metadata.version = latest_tag;
            Ok(updated_services)
        }
    }
}
