use std::{fs, path::PathBuf};

use anyhow::{Ok, Result};
use futures_util::{future::join_all, stream::StreamExt};
use log::{debug, warn, info};
use reqwest::Url;
use tokio::{fs::File, io::AsyncWriteExt};

/// Downloads files to a specified directory concurrently.
///
/// The urls and corresponding filenames must be in the correct order.
/// ## Example
/// The following can be executed within a non-async function.
/// ```
/// # use anyhow::Ok;
/// let rt = Runtime::new()?;
/// rt.block_on(download_files(&dir, urls_and_filenames))
/// # Ok(())
/// ```
pub(crate) async fn download_files(urls_dirs_filenames: Vec<DownloadTask>) -> Result<()> {
    let client = reqwest::Client::new();
    let mut download_handles = vec![];

    for task in urls_dirs_filenames {
        fs::create_dir_all(&task.dest_dir)?;

        let filepath = task.dest_dir.join(&task.filename);
        if filepath.exists() {
            info!("Skipped downloading file (already exists) {:?}.", filepath);
            continue
        };
        debug!("Downloading file {} from: {}", &task.filename, task.url);
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let mut file = File::create(filepath).await?;
            let mut stream = client.get(task.url).send().await?.bytes_stream();
            while let Some(result) = stream.next().await {
                let chunk = result?;
                file.write_all(&chunk).await?;
            }
            file.flush().await?;
            Ok(())
        });
        download_handles.push(handle);
    }
    join_all(download_handles).await;
    Ok(())
}

/// Details of a file to be downloaded and stored locally.
///
/// Used for coordinating concurrent downloads.
pub struct DownloadTask {
    pub url: Url,
    /// Directory that the file will be created in.
    pub dest_dir: PathBuf,
    /// Name of the file.
    pub filename: String,
}
