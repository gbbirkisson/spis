use chrono::{DateTime, Utc};
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use exif::{Exif, In, Tag, Value};
use image::DynamicImage;
use std::{fs, path::Path};
use subprocess::{Exec, Redirection};

pub struct ImageProcessor {
    exif: Option<Exif>,
    image: DynamicImage,
}

impl ImageProcessor {
    pub fn new(path: &Path) -> Result<Self> {
        let media_bytes = fs::read(path)?;
        let mut exif_buf_reader = std::io::Cursor::new(media_bytes);
        let exif_reader = exif::Reader::new();

        let exif = exif_reader.read_from_container(&mut exif_buf_reader).ok();

        let is_heif = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| matches!(ext, "heic" | "heif"));

        let image = if is_heif {
            open_heif_image(path).wrap_err("Failed to open HEIF image via heif-convert")?
        } else {
            image::open(path).wrap_err("Failed to open image")?
        };

        Ok(Self { exif, image })
    }

    pub fn get_timestamp(&self) -> Result<DateTime<Utc>> {
        let exif = self.exif.as_ref().ok_or_else(|| eyre!("no exif data"))?;

        let timestamp = exif_get_str(exif, Tag::DateTimeOriginal)
            .or_else(|_| exif_get_str(exif, Tag::DateTime))
            .wrap_err("Failed to get DateTime/DateTimeOriginal tag from exif data")?;
        let timestamp_tz = exif_get_str(exif, Tag::OffsetTimeOriginal);

        let mut timestamp_modified = timestamp.to_string().replace('-', ":");
        timestamp_modified.push(' ');
        match &timestamp_tz {
            Ok(tz) => timestamp_modified.push_str(tz.as_ref()),
            Err(_) => timestamp_modified.push_str("+02:00"), // TODO: Make configurable
        }

        let timestamp = DateTime::parse_from_str(&timestamp_modified, "%Y:%m:%d %H:%M:%S %z")
            .wrap_err(format!(
                "Failed to parse timestamp:{:?} tz:{:?}",
                &timestamp, &timestamp_tz
            ))?
            .with_timezone(&Utc);

        Ok(timestamp)
    }

    pub fn get_thumbnail(&self, size: u32) -> Result<DynamicImage> {
        let mut image = self.image.thumbnail(size, size);
        if let Some(exif) = &self.exif {
            image = match exif_get_u32(exif, Tag::Orientation) {
                // http://sylvana.net/jpegcrop/exif_orientation.html
                // Ok(1) => image,
                Ok(2) => image.flipv(),
                Ok(3) => image.rotate180(),
                Ok(4) => image.rotate180().flipv(),
                Ok(5) => image.rotate90().flipv(),
                Ok(6) => image.rotate90(),
                Ok(7) => image.rotate270().flipv(),
                Ok(8) => image.rotate270(),
                _ => image,
            };
        }
        Ok(crop(image))
    }
}

fn exif_get_u32(exif: &exif::Exif, tag: Tag) -> Result<u32> {
    #[allow(clippy::option_if_let_else)]
    match exif.get_field(tag, In::PRIMARY) {
        Some(field) => match field.value.get_uint(0) {
            Some(v) => Ok(v),
            _ => Err(eyre!("Failed getting number")),
        },
        None => Err(eyre!("Value not found")),
    }
}

fn exif_get_str(exif: &exif::Exif, tag: Tag) -> Result<&str> {
    match exif.get_field(tag, In::PRIMARY) {
        Some(field) => match &field.value {
            Value::Ascii(bytes) => {
                let bytes = bytes.first().ok_or_else(|| eyre!("Something is wrong"))?;
                Ok(std::str::from_utf8(bytes)?)
            }
            _ => Err(eyre!("Not Ascii value")),
        },
        None => Err(eyre!("Value not found")),
    }
}

#[must_use]
pub fn crop(mut image: DynamicImage) -> DynamicImage {
    let image_height = image.height();
    let image_width = image.width();
    if image_height > image_width {
        image.crop(
            0,
            (image_height - image_width) / 2,
            image_width,
            image_width,
        )
    } else {
        image.crop(
            (image_width - image_height) / 2,
            0,
            image_height,
            image_height,
        )
    }
}

fn open_heif_image(path: &Path) -> Result<DynamicImage> {
    let tmp = std::env::temp_dir().join(format!("spis-heif-{}.jpg", std::process::id()));

    let result = Exec::cmd("heif-convert")
        .arg("--quiet")
        .arg(path.as_os_str())
        .arg(&tmp)
        .stderr(Redirection::Pipe)
        .capture()
        .wrap_err("Failed to execute heif-convert")?;

    if !result.success() {
        let _ = fs::remove_file(&tmp);
        return Err(eyre!("heif-convert failed: {}", result.stderr_str().trim()));
    }

    let image = image::open(&tmp).wrap_err("Failed to open converted HEIF image");
    let _ = fs::remove_file(&tmp);
    image
}
