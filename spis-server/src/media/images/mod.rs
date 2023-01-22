use chrono::{DateTime, Utc};
use color_eyre::{eyre::eyre, Result};
use exif::{Exif, In, Tag, Value};
use image::DynamicImage;
use std::{fs, path::Path};

pub struct ImageProcessor {
    exif: Exif,
    image: DynamicImage,
}

impl ImageProcessor {
    pub fn new(path: &Path) -> Result<Self> {
        let media_bytes = fs::read(path)?;
        let mut exif_buf_reader = std::io::Cursor::new(media_bytes);
        let exif_reader = exif::Reader::new();
        let exif = exif_reader.read_from_container(&mut exif_buf_reader)?;
        let image = image::open(path)?;
        Ok(Self { exif, image })
    }

    pub fn get_timestamp(&self) -> Result<DateTime<Utc>> {
        let timestamp_tz = exif_get_str(&self.exif, Tag::OffsetTimeOriginal);
        let mut timestamp = exif_get_str(&self.exif, Tag::DateTimeOriginal)?
            .to_string()
            .replace("-", ":");
        timestamp.push_str(" ");
        timestamp.push_str(timestamp_tz.unwrap_or("+02:00")); // TODO: Make configurable
        Ok(DateTime::parse_from_str(&timestamp, "%Y:%m:%d %H:%M:%S %z")?.with_timezone(&Utc))
    }

    pub fn get_thumbnail(&self, size: u32) -> Result<DynamicImage> {
        let mut image = self.image.thumbnail(size, size);
        image = match exif_get_u32(&self.exif, Tag::Orientation) {
            // http://sylvana.net/jpegcrop/exif_orientation.html
            Ok(1) => image,
            Ok(2) => image.flipv(),
            Ok(3) => image.rotate180(),
            Ok(4) => image.rotate180().flipv(),
            Ok(5) => image.rotate90().flipv(),
            Ok(6) => image.rotate90(),
            Ok(7) => image.rotate270().flipv(),
            Ok(8) => image.rotate270(),
            _ => image,
        };
        Ok(crop(image))
    }
}

fn exif_get_u32(exif: &exif::Exif, tag: Tag) -> Result<u32> {
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
                let bytes = bytes.get(0).ok_or(eyre!("Something is wrong"))?;
                Ok(std::str::from_utf8(bytes)?)
            }
            _ => Err(eyre!("Not Ascii value")),
        },
        None => Err(eyre!("Value not found")),
    }
}

pub fn crop(mut image: DynamicImage) -> DynamicImage {
    let image_height = image.height();
    let image_width = image.width();
    match image_height > image_width {
        true => image.crop(
            0,
            (image_height - image_width) / 2,
            image_width,
            image_width,
        ),
        false => image.crop(
            (image_width - image_height) / 2,
            0,
            image_height,
            image_height,
        ),
    }
}
