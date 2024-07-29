use super::images::crop;
use chrono::{DateTime, Utc};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use image::DynamicImage;
use image::ImageReader;
use std::io::{Cursor, Read};
use subprocess::{Exec, Redirection};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct VideoProcessor {}

impl VideoProcessor {
    pub fn new() -> Result<Self> {
        which::which("ffprobe").wrap_err("ffprobe not installed")?;
        which::which("ffmpeg").wrap_err("ffmpeg not installed")?;
        Ok(Self {})
    }

    pub fn get_timestamp(&self, file: &str) -> Result<DateTime<Utc>> {
        let timestamp = Exec::cmd("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-select_streams")
            .arg("v:0")
            .arg("-show_entries")
            .arg("stream_tags=creation_time")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1")
            .arg(file)
            .stdout(Redirection::Pipe)
            .capture()
            .wrap_err("Failed to execute ffprobe")?
            .stdout_str()
            .trim()
            .to_string();

        if timestamp.is_empty() {
            return Err(eyre!("No creation_time tag in video"));
        }

        let timestamp_modified = timestamp.replace('z', "Z");
        Ok(DateTime::parse_from_rfc3339(&timestamp_modified)
            .wrap_err(format!("Failed to parse video timestamp:{:?}", &timestamp))?
            .with_timezone(&Utc))
    }

    pub fn get_thumbnail(&self, file: &str, size: u32) -> Result<DynamicImage> {
        let mut img = Exec::cmd("ffmpeg")
            .arg("-v")
            .arg("quiet")
            .arg("-ss")
            .arg("00:00:00.00")
            .arg("-i")
            .arg(file)
            .arg("-vf")
            .arg(format!(
                "scale={size}:{size}:force_original_aspect_ratio=increase"
            ))
            .arg("-vframes")
            .arg("1")
            .arg("-f")
            .arg("webp")
            .arg("pipe:1")
            .stream_stdout()
            .wrap_err("Failed to execute ffmpeg")?;

        let mut buffer = Vec::new();
        img.read_to_end(&mut buffer)
            .wrap_err("Failed to read thumbnail into buffer")?;

        let mut img = ImageReader::with_format(Cursor::new(&buffer), image::ImageFormat::WebP)
            .decode()
            .wrap_err("Failed to decode image")?;

        img = crop(img);
        img = img.thumbnail(size, size);

        Ok(img)
    }
}
