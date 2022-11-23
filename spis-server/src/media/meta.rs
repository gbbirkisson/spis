use chrono::{DateTime, Utc};
use exif::{In, Tag, Value};
use eyre::{eyre, Result};

pub(crate) struct MediaProcessedOrientation(pub(crate) i32, pub(crate) bool);

pub(crate) struct MediaProcessedExif {
    pub(crate) orientation: MediaProcessedOrientation,
    pub(crate) taken: Option<DateTime<Utc>>,
}

pub(crate) fn image_exif_read(bytes: &[u8]) -> Result<MediaProcessedExif> {
    let mut exif_buf_reader = std::io::Cursor::new(bytes);
    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut exif_buf_reader)?;

    let orientation = match exif_get_u32(&exif, Tag::Orientation) {
        // http://sylvana.net/jpegcrop/exif_orientation.html
        Ok(1) => MediaProcessedOrientation(0, false),
        Ok(2) => MediaProcessedOrientation(0, true),
        Ok(3) => MediaProcessedOrientation(180, false),
        Ok(4) => MediaProcessedOrientation(180, true),
        Ok(5) => MediaProcessedOrientation(90, true),
        Ok(6) => MediaProcessedOrientation(90, false),
        Ok(7) => MediaProcessedOrientation(270, true),
        Ok(8) => MediaProcessedOrientation(270, false),
        _ => MediaProcessedOrientation(0, false),
    };

    let timestamp_tz = exif_get_str(&exif, Tag::OffsetTimeOriginal);
    let taken = match exif_get_str(&exif, Tag::DateTimeOriginal) {
        Ok(time) => {
            let pair = match timestamp_tz {
                Ok(tz) => (time.to_string() + tz, "%Y:%m:%d %H:%M:%S %z"),
                _ => (time.to_string(), "%Y:%m:%d %H:%M:%S"),
            };
            match DateTime::parse_from_str(&pair.0, pair.1) {
                Ok(time) => Some(time.with_timezone(&Utc)),
                _ => None,
            }
        }
        _ => None,
    };

    Ok(MediaProcessedExif { orientation, taken })
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
