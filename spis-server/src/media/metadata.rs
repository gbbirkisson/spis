use chrono::{DateTime, Utc};
use color_eyre::{eyre::eyre, Result};
use exif::{In, Tag, Value};

#[derive(Debug)]
pub struct MediaProcessedExif {
    pub(crate) orientation: MediaProcessedOrientation,
    pub(crate) taken: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct MediaProcessedOrientation {
    pub(crate) rotation: i32,
    pub(crate) mirrored: bool,
}

impl MediaProcessedOrientation {
    fn new(rotation: i32, mirrored: bool) -> Self {
        Self { rotation, mirrored }
    }
}

pub fn image_exif_read(bytes: &[u8]) -> Result<MediaProcessedExif> {
    let mut exif_buf_reader = std::io::Cursor::new(bytes);
    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut exif_buf_reader)?;

    // for f in exif.fields() {
    //     println!(
    //         "{} {} {}",
    //         f.tag,
    //         f.ifd_num,
    //         f.display_value().with_unit(&exif)
    //     );
    // }

    let orientation = match exif_get_u32(&exif, Tag::Orientation) {
        // http://sylvana.net/jpegcrop/exif_orientation.html
        Ok(1) => MediaProcessedOrientation::new(0, false),
        Ok(2) => MediaProcessedOrientation::new(0, true),
        Ok(3) => MediaProcessedOrientation::new(180, false),
        Ok(4) => MediaProcessedOrientation::new(180, true),
        Ok(5) => MediaProcessedOrientation::new(90, true),
        Ok(6) => MediaProcessedOrientation::new(90, false),
        Ok(7) => MediaProcessedOrientation::new(270, true),
        Ok(8) => MediaProcessedOrientation::new(270, false),
        _ => MediaProcessedOrientation::new(0, false),
    };

    let timestamp_tz = exif_get_str(&exif, Tag::OffsetTimeOriginal);
    let taken = match exif_get_str(&exif, Tag::DateTimeOriginal) {
        Ok(time) => {
            let time = time.to_owned() + " " + timestamp_tz.unwrap_or("+02:00"); // TODO: Make configurable
            match DateTime::parse_from_str(&time, "%Y:%m:%d %H:%M:%S %z") {
                Ok(time) => Some(time.with_timezone(&Utc)),
                Err(e) => {
                    tracing::warn!("Failed parsing time '{}': {}", time, e);
                    None
                }
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
