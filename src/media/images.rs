use chrono::{DateTime, Utc};
use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use exif::{Exif, In, Rational, Tag, Value};
use image::DynamicImage;
use std::{fs, path::Path};

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
        let image = image::open(path).wrap_err("Failed to open image")?;

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

    pub fn get_gps_coordinates(&self) -> Result<(f64, f64)> {
        let exif = self.exif.as_ref().ok_or_else(|| eyre!("No EXIF data"))?;

        let latitude = exif_get_gps_coordinate(exif, Tag::GPSLatitude, Tag::GPSLatitudeRef)
            .wrap_err("Failed to get GPS latitude")?;
        let longitude = exif_get_gps_coordinate(exif, Tag::GPSLongitude, Tag::GPSLongitudeRef)
            .wrap_err("Failed to get GPS longitude")?;

        Ok((latitude, longitude))
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

// Helper function to extract GPS coordinate
fn exif_get_gps_coordinate(exif: &Exif, tag_coord: Tag, tag_ref: Tag) -> Result<f64> {
    let coord_field = exif
        .get_field(tag_coord, In::PRIMARY)
        .ok_or_else(|| eyre!("GPS coordinate not found"))?;
    let ref_field = exif
        .get_field(tag_ref, In::PRIMARY)
        .ok_or_else(|| eyre!("GPS reference not found"))?;

    let coord = if let Value::Rational(ref vec) = coord_field.value {
        vec
    } else {
        return Err(eyre!("Invalid GPS coordinate format"));
    };

    let direction = if let Value::Ascii(ref vec) = ref_field.value {
        let bytes = vec.first().ok_or_else(|| eyre!("Empty GPS reference"))?;
        std::str::from_utf8(bytes)?.trim().to_string()
    } else {
        return Err(eyre!("Invalid GPS reference format"));
    };

    // Convert degrees, minutes, seconds to decimal degrees
    if coord.len() != 3 {
        return Err(eyre!("Invalid GPS coordinate length"));
    }

    let degrees = coord[0].to_f64();
    let minutes = coord[1].to_f64();
    let seconds = coord[2].to_f64();

    let mut decimal = degrees + minutes / 60.0 + seconds / 3600.0;

    if direction == "S" || direction == "W" {
        decimal = -decimal;
    }

    Ok(decimal)
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
