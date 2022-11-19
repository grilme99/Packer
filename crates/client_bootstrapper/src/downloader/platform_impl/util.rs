//! Collection of shared utilities between OS-specific downloader implementations.

use std::{
    fs,
    io::{Read, Seek},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{bail, Context};
use futures::future;
use reqwest::{
    header::{HeaderValue, CONTENT_LENGTH, RANGE},
    Client, StatusCode,
};
use zip::ZipArchive;

/// Download a client files and write them to path asynchronously.
pub async fn download_file(
    client: &Client,
    url: &str,
    write_to: &Path,
    target_concurrent_downloads: u32,
) -> anyhow::Result<PathBuf> {
    let hash = sha256::digest(url);
    log::debug!("Downloading {url} ({hash})");

    // Get the content length so we can download the file in parallel chunks
    let response = client
        .head(url)
        .send()
        .await
        .context(format!("Failed to make HEAD reqwest to {url}"))?;

    let content_length = response
        .headers()
        .get(CONTENT_LENGTH)
        .context("HEAD response does not include content length")?
        .to_str()
        .context("Failed to convert content length to string slice")?;

    let content_length =
        u64::from_str(content_length).context("Failed to convert string slice to u64")?;

    log::debug!("Content length for {url}: {content_length}");

    // Start downloading chunks
    log::debug!("Downloading file at {url}");

    let buffer_size = content_length.div_floor(target_concurrent_downloads as u64);
    let range_iter = PartialRangeIter::new(0, content_length - 1, buffer_size)
        .context("Failed to make range iter")?;

    log::debug!("Download chunks for {url}: {}", range_iter.clone().count());

    let mut download_tasks = Vec::new();
    for range in range_iter {
        download_tasks.push(download_partial_chunk(client, url, range));
    }

    let downloaded_chunks = future::try_join_all(download_tasks)
        .await
        .context("Failed to download {url}")?;

    log::debug!("Downloaded file at {url}");

    // Join all downloaded chunks into one byte array and write to path
    let mut file_bytes = Vec::new();
    for mut chunk in downloaded_chunks {
        file_bytes.append(&mut chunk);
    }

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

    log::debug!("Parsed ZIP archive for {url}");
    extract_archive(&mut archive, write_to).context("Failed to extract archive")?;

    Ok(path)
}

/// Download a partial file chunk from the CDN in parallel to speed up download
async fn download_partial_chunk(
    client: &Client,
    url: &str,
    range: HeaderValue,
) -> anyhow::Result<Vec<u8>> {
    log::trace!("Range {range:?} ({url})");

    let response = client
        .get(url)
        .header(RANGE, &range)
        .send()
        .await
        .context("Request for range {range:?} at {url} failed")?;

    let status = response.status();
    if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
        bail!("Got unexpected response from CDN ({url} {range:?}): {status}");
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to get bytes from CDN response {url} ({range:?})")?
        .to_vec();

    Ok(bytes)
}

/// Modified from https://github.com/zip-rs/zip/blob/5737927dbbd15a8b648c315f2f8e2a39cdc1a430/examples/extract.rs
fn extract_archive<T: Read + Seek>(
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

/// https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html?highlight=range#make-a-partial-download-with-http-range-headers
#[derive(Debug, Clone)]
struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u64,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u64) -> anyhow::Result<Self> {
        if buffer_size == 0 {
            bail!("Expected a value greater than 0 for buffer_size, got {buffer_size}.");
        }

        Ok(PartialRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for PartialRangeIter {
    type Item = HeaderValue;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size, self.end - self.start + 1);
            Some(
                HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1))
                    .expect("string provided by format!"),
            )
        }
    }
}
