use std::error::Error;
use std::fs;
use std::io;
use zip::ZipArchive;

pub fn download_file(url: &str, file_name: &str) -> Result<(), Box<dyn Error>> {
    let data_dir = "Data";
    fs::create_dir_all(data_dir)?;

    let response = reqwest::blocking::get(url)?;

    let file_path = format!("{}/{}", data_dir, file_name);
    let context = response.bytes()?;
    fs::write(&file_path, &context)?;

    println!("Saved to: {}", file_path);
    Ok(())
}

pub fn extract_zip(file_path: &str) -> Result<(), Box<dyn Error>> {
    let base_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => return Err(Box::new(e)),
    };
    let file_path = base_dir.join(file_path);

    let out_dir = base_dir.join("Data");

    let zip_file = fs::File::open(&file_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    let mut some_files_failed = false;

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                some_files_failed = true;
                eprintln!("Failed to access file at index {}: {}", i, e);
                continue;
            }
        };

        let out_path = out_dir.join(file.mangled_name());

        if let Err(e) = fs::create_dir_all(out_path.parent().unwrap()) {
            some_files_failed = true;
            eprintln!(
                "Failed to create directory for {}: {}",
                out_path.display(),
                e
            );
            continue;
        }

        if let Err(e) = std::io::copy(&mut file, &mut fs::File::create(&out_path)?) {
            some_files_failed = true;
            eprintln!("Failed to extract file {}: {}", out_path.display(), e);
            continue;
        }
    }

    if some_files_failed {
        Err(Box::new(io::Error::other(
            "Some files failed to extract. Check logs for details.",
        )))
    } else {
        Ok(())
    }
}
