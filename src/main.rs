mod dupe_scanner;
use clap::Parser;
use dupe_scanner::DupeScanner;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let start_dir = Path::new(&args.directory);
    let mut dupe_scanner = DupeScanner::from_path(start_dir, args.ignore_symlinks);
    dupe_scanner.find_dupes()?;
    Ok(())
}

/// Find duplicate files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to recursively search
    #[arg(index = 1, default_value_t = String::from("."))]
    directory: String,

    /// Ignore Symlinks
    #[arg(short, long)]
    ignore_symlinks: bool,
}
