use super::images::crop;
use chrono::{DateTime, Utc};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use image::io::Reader;
use image::DynamicImage;
use std::io::{Cursor, Read};
use subprocess::{Exec, Redirection};

#[derive(Clone)]
pub struct VideoProcessor {}

impl VideoProcessor {
    pub fn new() -> Result<Self> {
        which::which("ffprobe").map_err(|_| eyre!("ffprobe not installed"))?;
        which::which("ffmpeg").map_err(|_| eyre!("ffmpeg not installed"))?;
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
            .capture()?
            .stdout_str()
            .trim()
            .replace('z', "Z");
        Ok(DateTime::parse_from_rfc3339(&timestamp)
            .wrap_err("failed to parse video timestamp")?
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
            .stream_stdout()?;

        let mut buffer = Vec::new();
        img.read_to_end(&mut buffer)?;

        let mut img =
            Reader::with_format(Cursor::new(&buffer), image::ImageFormat::WebP).decode()?;

        img = crop(img);
        img = img.thumbnail(size, size);

        Ok(img)
    }
}
