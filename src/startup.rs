use std::error::Error;

use crate::downloader::Downloader;

pub fn run(
    ws_url: String,
    http_url: String,
    buffer_size: usize,
) -> Result<Downloader, Box<dyn Error>> {
    let downloader = Downloader::new(ws_url, http_url, buffer_size);
    Ok(downloader)
}
