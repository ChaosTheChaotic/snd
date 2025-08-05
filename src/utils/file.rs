use crate::utils::fileparse::fpre;
use dirs::download_dir;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rand::Rng;
use std::{
    env::temp_dir,
    ffi::OsStr,
    fs::{remove_file, File},
    path::{Path, PathBuf},
};
use tar::{Archive, Builder};

pub fn downloadfc(full_path: &Path) -> (File, PathBuf) {
    let fname = full_path.file_name().unwrap_or_else(|| OsStr::new("file"));
    let dld = download_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut candidate = dld.join(fname);
    let mut count = 0;

    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return (file, candidate),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                count += 1;

                let stem = full_path.file_stem().unwrap_or_else(|| OsStr::new("file"));
                let extension = full_path.extension();

                let mut new_fname = std::ffi::OsString::with_capacity(32);
                new_fname.push(stem);
                new_fname.push("_");
                new_fname.push(count.to_string());

                if let Some(ext) = extension {
                    if !ext.is_empty() {
                        new_fname.push(".");
                        new_fname.push(ext);
                    }
                }

                candidate = dld.join(new_fname);

                if count > 10000 {
                    panic!("Failed to create file after 10000 attempts");
                }
            }
            Err(e) => panic!("Failed to create file: {}", e),
        }
    }
}

// This function creates a tar file but does not remove it. Removing it should be handled by any
// code that calls this
pub fn tarify(fpath: String) -> PathBuf {
    let dir_name = Path::new(&fpath)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("temp_dir");

    let random_bytes: [u8; 64] = rand::rng().random();
    let random_suffix = hex::encode(random_bytes);

    let tarfpth = temp_dir().join(format!("{}_??{}.tar.gz", dir_name, random_suffix));
    let tarfp = File::create(&tarfpth).expect("Failed to create temp file");
    let enc = GzEncoder::new(tarfp, Compression::default());
    let mut tar = Builder::new(enc);
    tar.append_dir_all("", &fpath)
        .expect("Failed to add directory to archive");
    tar.finish()
        .expect("Failed to finish writing to the archive");
    tarfpth
}

pub fn untarify(saved_path: &Path) -> std::io::Result<()> {
    let file = File::open(saved_path)?;
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);

    let sname = fpre(saved_path)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let fnameo = sname.split("_??").next().unwrap_or(&sname);

    let dl_dir = download_dir().unwrap_or_else(|| PathBuf::from("."));
    let sfpth = dl_dir.join(&fnameo);

    if !sfpth.exists() {
        std::fs::create_dir_all(&sfpth)?;
    }

    archive.unpack(&sfpth)?;
    remove_file(saved_path)?;

    Ok(())
}
