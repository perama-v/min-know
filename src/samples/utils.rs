use std::{fs, path::PathBuf};

use anyhow::{Ok, Result};
use futures_util::{future::join_all, stream::StreamExt};
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
pub async fn download_files(dest_dir: &PathBuf, urls_and_filenames: Vec<(Url, &str)>) -> Result<()> {
    let client = reqwest::Client::new();
    fs::create_dir_all(&dest_dir)?;

    let mut download_handles = vec![];

    for (url, filename) in urls_and_filenames {
        let filepath = dest_dir.join(filename);
        println!("Downloading file {} from: {}", filename, url);
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let mut file = File::create(filepath).await?;
            let mut stream = client.get(url).send().await?.bytes_stream();
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