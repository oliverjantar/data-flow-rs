use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek};
use std::thread;

fn main() -> io::Result<()> {
    let file = File::open("./data/2")?;
    let file_size = file.metadata()?.len();
    let middle = file_size / 2;

    let file1 = file.try_clone()?;
    let handle1 = thread::spawn(move || {
        let mut reader = BufReader::new(&file1);
        reader.seek(io::SeekFrom::Start(0)).unwrap();
        for line in reader.lines().take_while(|line| {
            let pos = line
                .as_ref()
                .map(|s| s.as_bytes().len() as u64 + 1)
                .unwrap_or(0);
            pos <= middle
        }) {
            let line = line.unwrap();
            println!("Thread 1: {}", line);
        }
    });

    let handle2 = thread::spawn(move || {
        let mut reader = BufReader::new(&file);
        reader.seek(io::SeekFrom::Start(middle)).unwrap();
        for line in reader.lines() {
            let line = line.unwrap();
            println!("Thread 2: {}", line);
        }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    Ok(())
}

/*

use anyhow::{Context, Result};
use async_std::{
    fs,
    path::{Path, PathBuf},
};
use futures::stream::{Stream, StreamExt};
use std::ffi::OsStr;

async fn read_dir_filtered<P: AsRef<Path>>(
    ending: &'static str,
    path: P,
) -> Result<std::pin::Pin<Box<dyn Stream<Item = PathBuf>>>> {
    let dir_stream = fs::read_dir(&path)
        .await
        .with_context(|| format!("failed to read: {:?}", path.as_ref()))?;
    Ok(Box::pin(dir_stream.filter_map(move |d| async move {
        match d {
            Ok(rr) if path_has_ending(rr.path(), ending) => Some(rr.path().to_path_buf()),
            Ok(_) | Err(_) => None,
        }
    })))
}

fn path_has_ending(p: impl AsRef<Path>, ending: &str) -> bool {
    match p.as_ref().extension().and_then(OsStr::to_str) {
        Some(ext) => ending == ext,
        _ => false,
    }
}

*/
