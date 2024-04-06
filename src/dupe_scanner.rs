use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    io::{BufReader, Read},
    path::Path,
    sync::{Arc, RwLock},
};
use threadpool::ThreadPool;

type SeenFiles = Arc<RwLock<HashMap<Vec<u8>, String>>>;
pub struct DupeScanner {
    start_dir: String,
    worker_pool: ThreadPool,
    seen_files: SeenFiles,
    ignore_symlinks: bool,
}

impl Default for DupeScanner {
    fn default() -> Self {
        Self {
            start_dir: String::from("."),
            worker_pool: Default::default(),
            seen_files: Default::default(),
            ignore_symlinks: true,
        }
    }
}

impl DupeScanner {
    pub fn new(start_directory: &str, worker_pool_size: usize, ignore_symlinks: bool) -> Self {
        DupeScanner {
            start_dir: String::from(start_directory),
            worker_pool: ThreadPool::new(worker_pool_size),
            seen_files: Arc::new(RwLock::new(HashMap::new())),
            ignore_symlinks,
        }
    }

    pub fn from_path(path: &Path, ignore_symlinks: bool) -> Self {
        DupeScanner {
            start_dir: String::from(path.to_str().unwrap()),
            worker_pool: Default::default(),
            seen_files: Default::default(),
            ignore_symlinks,
        }
    }

    pub fn find_dupes(&mut self) -> std::io::Result<()> {
        let start_dir = Path::new(&self.start_dir).to_owned();
        self.scan_directory(&start_dir)?;
        self.worker_pool.join();
        Ok(())
    }

    fn scan_directory(&mut self, path: &Path) -> std::io::Result<()> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if !path.is_dir() {
                    let seen_copy = self.seen_files.clone();
                    // Ignore symlinks if needed
                    if self.ignore_symlinks && path.is_symlink() {
                        continue;
                    }
                    self.worker_pool.execute(move || {
                        handle_file(seen_copy, path.as_path());
                    });
                } else {
                    self.scan_directory(&entry.path())?;
                }
            }
        } else {
            // Ignore symlinks if needed
            if self.ignore_symlinks && path.is_symlink() {
                return Ok(());
            }
            let copy = self.seen_files.clone();
            let path_copy = path.to_owned();
            self.worker_pool.execute(move || {
                handle_file(copy, &path_copy);
            });
        }
        Ok(())
    }
}

fn handle_file(seen_files: SeenFiles, path: &Path) {
    let hash = hash_file(path);
    let seen_file;
    {
        if let Some(s) = seen_files.read().unwrap().get(&hash) {
            seen_file = Some(s.clone());
        } else {
            seen_file = None;
        }
    }

    if let Some(x) = seen_file {
        println!("{} = {}", path.to_str().unwrap(), x);
    } else {
        seen_files
            .write()
            .unwrap()
            .insert(hash, String::from(path.to_str().unwrap()));
    }
}

fn hash_file(path: &Path) -> Vec<u8> {
    const BUFFER_SIZE: usize = 4096;
    let mut hasher = Sha1::new();
    let file = std::fs::File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        if let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            } else {
                hasher.update(&buffer[..bytes_read]);
            }
        }
    }
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;
    #[test]
    fn test_hasher_hello() {
        let result = hash_file(Path::new("test/helloworld.txt"));
        assert_eq!(result[..], hex!("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"));
    }

    #[test]
    fn test_hasher_odyssey() {
        let result = hash_file(Path::new("test/odyssey.mb.txt"));
        assert_eq!(result[..], hex!("84d81cb70dfc52a964e3c6f38d753533e134a9e8"));
    }
    #[test]
    fn test_dupes() {
        let mut correct = HashMap::new();
        correct.insert(
            hex!("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed").to_vec(),
            "test/test2.txt".to_string(),
        );
        correct.insert(
            hex!("84d81cb70dfc52a964e3c6f38d753533e134a9e8").to_vec(),
            "test/odyssey2.txt".to_string(),
        );
        correct.insert(
            hex!("b444ac06613fc8d63795be9ad0beaf55011936ac").to_vec(),
            "test/test1.txt".to_string(),
        );

        let mut dupe_scanner = DupeScanner::new("test/", 8, true);
        dupe_scanner.find_dupes().unwrap();
        for (hash, _) in correct {
            assert!(dupe_scanner.seen_files.read().unwrap().get(&hash).is_some());
        }
    }
}
