use chksum::prelude::*;
use image::io::Reader as ImageReader;
use std::{
    collections::HashMap,
    fs::{read_dir, File},
    path::Path,
};
use walkdir::{DirEntry, WalkDir};

use chksum::prelude::HashAlgorithm;
use model::Image;

static STATE_FILE_NAME: &str = "state.json";
static THUMBNAIL_DIR: &str = "thumbnails";
static THUMBNAIL_FORMAT: &str = "webp";
static THUMBNAIL_HEIGHT: u32 = 200;
static HASH_ALGO: HashAlgorithm = HashAlgorithm::MD5;

pub struct State {
    state_dir: String,
    image_dir: String,
    image_dir_hash: String,
    images: HashMap<String, Image>,
}

impl State {
    pub fn load(state_dir: &str, image_dir: &str) -> Self {
        // TODO: Try to load old state

        let mut res = Self {
            state_dir: state_dir.to_string(),
            image_dir: image_dir.to_string(),
            image_dir_hash: "".to_string(),
            images: HashMap::new(),
        };
        res.reload().unwrap(); // TODO: Handle error

        res
    }

    fn reload(&mut self) -> Result<()> {
        let img_dir_path = Path::new(&self.image_dir);

        let hash_digest = read_dir(img_dir_path)?.chksum(HASH_ALGO)?; // TODO: Handle
        let hash = format!("{:x}", hash_digest);

        tracing::info!("Current image dir hash: {}", self.image_dir_hash);
        if self.image_dir_hash == hash {
            tracing::info!("No change detected");
            return Ok(());
        }

        tracing::info!("Changes detected");
        self.image_dir_hash = hash;
        tracing::info!("New image dir hash: {}", self.image_dir_hash);

        let mut thumbnail_dir = "".to_owned();
        thumbnail_dir.push_str(&self.state_dir);
        thumbnail_dir.push_str("/");
        thumbnail_dir.push_str(THUMBNAIL_DIR);

        self.images = walk(&self.image_dir, &thumbnail_dir)?; // TODO: Handle

        Ok(())
    }

    pub fn images(&self) -> &HashMap<String, Image> {
        &self.images
    }
}

fn walk(image_dir: &str, thumbnail_dir: &str) -> Result<HashMap<String, Image>> {
    let mut res = HashMap::new();
    tracing::info!("Start to walk dir: {}", image_dir);
    let walk = WalkDir::new(image_dir).into_iter();

    let thumbnail_path = Path::new(thumbnail_dir);

    let mut count = 0;
    for entry in walk.filter_map(|e| e.ok()).filter(walk_filter) {
        let mut image_file = File::open(entry.path())?; // TODO: Handle
        let image_hash_digest = image_file.chksum(HASH_ALGO)?; // TODO: Handle
        let image_hash = format!("{:x}", image_hash_digest);

        let mut image_thumbnail_path = thumbnail_path.join(image_hash.clone());
        image_thumbnail_path.set_extension(THUMBNAIL_FORMAT);
        if !image_thumbnail_path.exists() {
            tracing::info!("Creating thumbnail: {:?}", image_thumbnail_path);
            let mut image = ImageReader::open(entry.path()).unwrap().decode().unwrap(); // TODO
            image = image.thumbnail(THUMBNAIL_HEIGHT * 2, THUMBNAIL_HEIGHT);
            image.save(image_thumbnail_path.clone()).unwrap(); // TODO
        }

        res.insert(
            image_hash.clone(),
            Image {
                hash: image_hash,
                thumbnail: image_thumbnail_path
                    .into_os_string()
                    .into_string()
                    .unwrap()
                    .replace("dev", ""),
                path: entry
                    .path()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .replace("dev", ""),
            },
        );

        count += 1;
        if count % 10 == 0 {
            tracing::info!("Added {} images", count)
        }
    }
    tracing::info!("Scan done. Total image count: {}", count);

    Ok(res)
}

fn walk_filter(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|f| f.ends_with("jpg"))
        .unwrap_or(false)
}
