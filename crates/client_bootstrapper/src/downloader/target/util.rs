//! Collection of shared utilities between OS-specific downloader implementations.

use std::{
    fs,
    io::{Read, Seek},
    path::{Path, PathBuf},
};

use anyhow::Context;
use reqwest::Client;
use zip::ZipArchive;

/// Download a client files and write them to path asynchronously.
pub async fn download_file(client: &Client, url: &str, write_to: &Path) -> anyhow::Result<PathBuf> {
    let hash = sha256::digest(url);
    log::debug!("Downloading {url} ({hash})");

    // Download file from Roblox CDN
    let file_bytes = client
        .get(url)
        .send()
        .await
        .context(format!("Failed to request client file: {url}"))?
        .bytes()
        .await
        .context(format!("Failed to parse response to bytes for file: {url}"))?;

    log::debug!("Writing ZIP {url} ({hash}) to path");

    // Write the ZIP file to path before extracting it.
    // TODO: Work out how to skip this redundant step. Writing to disk wastes time if we can just
    //  immediately extract the ZIP.
    let path = write_to.join(format!("{hash}.zip"));
    fs::write(&path, file_bytes).context("Failed to write RobloxPlayer ZIP to path")?;

    log::debug!("Wrote ZIP {url} ({hash}) to path. Now extracting.");

    // Extract the ZIP.
    // TODO: Make ZIP extraction async for Windows because there's a lot of files to extract.
    let file = fs::File::open(&path).context(format!("Failed to read path into file: {path:?}"))?;
    let mut archive =
        ZipArchive::new(file).context(format!("Failed to create archive for path {path:?}"))?;

    extract_archive(&mut archive, write_to).context("Failed to extract archive")?;

    Ok(path)
}

/// Modified from https://github.com/zip-rs/zip/blob/5737927dbbd15a8b648c315f2f8e2a39cdc1a430/examples/extract.rs
pub fn extract_archive<T: Read + Seek>(
    archive: &mut ZipArchive<T>,
    write_to: &Path,
) -> anyhow::Result<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = write_to.join(match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        });

        if (*file.name()).ends_with('/') {
            log::trace!("File {i} extracted to \"{outpath:?}\"");
            fs::create_dir_all(&outpath)
                .context(format!("Failed to create directory {outpath:?}"))?;
        } else {
            log::trace!(
                "File {i} extracted to \"{outpath:?}\" ({} bytes)",
                file.size()
            );

            if let Some(path) = outpath.parent() {
                if !path.exists() {
                    fs::create_dir_all(&path)
                        .context(format!("Failed to create directory {path:?}"))?;
                }
            }

            let mut outfile = fs::File::create(&outpath)
                .context(format!("Failed to create file at {outpath:?}"))?;

            std::io::copy(&mut file, &mut outfile).context(format!(
                "Failed to copy file contents into writer for {outpath:?}"
            ))?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
}
