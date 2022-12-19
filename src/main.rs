use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    io::{BufReader, Read},
    path::Path,
};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    let dir = args.nth(1).unwrap_or_else(|| String::from("."));
    scan_directory(Path::new(&dir), &mut HashMap::new())?;
    Ok(())
}

fn scan_directory(path: &Path, seen: &mut HashMap<Vec<u8>, String>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                handle_file(path.as_path(), seen);
            } else {
                scan_directory(&entry.path(), seen)?;
            }
        }
    } else {
        handle_file(path, seen);
    }
    Ok(())
}

fn handle_file(path: &Path, seen: &mut HashMap<Vec<u8>, String>) {
    let hash = hash_file(path);
    if let Some(x) = seen.get(&hash) {
        println!("{} = {}", path.to_str().unwrap(), x);
    } else {
        seen.insert(hash, String::from(path.to_str().unwrap()));
    }
}

const BUFFER_SIZE: usize = 4096;

fn hash_file(path: &Path) -> Vec<u8> {
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
        let mut hashes = HashMap::new();
        scan_directory(Path::new("test"), &mut hashes).unwrap();
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
        assert_eq!(&hashes, &correct);
    }
}
