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
