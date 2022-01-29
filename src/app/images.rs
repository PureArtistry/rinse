use std::{
    env::var,
    fs::{self, read_dir},
    path::{Path, PathBuf}
};

use anyhow::{bail, Result};
use eframe::epi::Image;

const NO_ART: &[u8; 250000] = include_bytes!("../../assets/NO_ART");

pub fn get_cover(paths: &(&str, &str)) -> Image {
    gen_image(paths).unwrap_or_else(|_| Image::from_rgba_unmultiplied([250, 250], NO_ART))
}

fn gen_image(paths: &(&str, &str)) -> Result<Image> {
    let image_path = search(paths)?;

    let cache_path = var("XDG_CACHE_HOME").unwrap_or_else(|_| [&var("HOME").unwrap(), ".cache"].join("/"));
    let cache_dir = Path::new(&[&cache_path, "rinse"].join("/")).to_owned();
    if !Path::is_dir(&cache_dir) {
        fs::create_dir(&cache_dir)?
    }

    let cache_name = base64::encode_config(
        image_path.to_str().unwrap().bytes().collect::<Vec<u8>>(),
        base64::URL_SAFE
    );
    let cache_file = [cache_dir.to_str().unwrap(), &*cache_name].join("/");

    match Path::is_file(Path::new(&cache_file)) {
        true => Ok(Image::from_rgba_unmultiplied([250, 250], &fs::read(&cache_file)?)),
        false => {
            let image = image::open(image_path)?;
            let resized = image.resize_to_fill(250, 250, image::imageops::FilterType::Lanczos3);
            let cache_buffer = resized.to_rgba8().into_vec();
            fs::write(cache_file, &cache_buffer)?;
            Ok(Image::from_rgba_unmultiplied([250, 250], &cache_buffer))
        }
    }
}

fn search(paths: &(&str, &str)) -> Result<PathBuf> {
    let (music_dir, filepath) = paths;
    let mut file_dir = filepath.split('/').collect::<Vec<&str>>();
    file_dir.pop();
    let cover_path = file_dir.join("/");
    let dir_contents = read_dir([music_dir.to_string(), cover_path].join("/"))?;
    for x in dir_contents.flatten() {
        match x.file_name().to_ascii_lowercase().to_str().unwrap() {
            "cover.jpeg" | "cover.jpg" | "cover.png" => return Ok(x.path()),
            _ => {}
        }
    }
    bail!("")
}
