use bollard::{image::CreateImageOptions, Docker};

use super::types::Container;
use anyhow::{bail, Result};
use cached::proc_macro::cached;
use futures_util::stream::TryStreamExt;

#[cached(
    key = "String",
    convert = r##"{ format!("{}", container) }"##,
    result = true
)]
pub async fn get_hash(container: &str, docker: &Docker) -> Result<String> {
    tracing::info!("Pulling {}...", container);
    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: container,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;
    let hash = docker.inspect_image(container).await?;
    let Some(digests) = hash.repo_digests else {
        bail!("No digest found for {}", container);
    };
    let Some(result) = digests.first() else {
        bail!("No digest found for {}", container);
    };

    Ok(result.to_owned().split('@').last().unwrap().to_owned())
}

/// Updates a single container
/// Returns false when the to_version is already the latest version,
/// true when an update was available
pub async fn update_container(
    container: &mut Container,
    to_version: &String,
    docker: &Docker,
) -> Result<()> {
    let image = &container.image;
    let Some(image_without_tag) = image.split(':').next() else {
        bail!("Image {} does not contain a tag", image);
    };
    let mut new_tag = image_without_tag.to_owned() + ":" + to_version;
    let new_hash = get_hash(&new_tag, docker).await;
    let hash: String;
    if let Ok(new_image) = new_hash {
        hash = new_image;
    } else {
        new_tag = image_without_tag.to_owned() + ":v" + to_version;
        let new_image = get_hash(&new_tag, docker).await?;
        hash = new_image;
    }
    container.image = new_tag + "@" + &hash;
    Ok(())
}
