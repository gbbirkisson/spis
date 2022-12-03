use super::images::crop;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Oslo;
use color_eyre::{eyre::eyre, Result};
use image::io::Reader;
use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::io::{Cursor, Read};
use std::sync::Arc;
use subprocess::{Exec, Redirection};

const FONT_BYTES: &[u8; 781460] = include_bytes!("Kelvinch-1GY8j.ttf");
const WHITE: Rgba<u8> = Rgba([200u8, 200u8, 200u8, 50u8]);

#[derive(Clone)]
pub struct VideoProcessor<'a> {
    font: Arc<Font<'a>>,
}

impl<'a> VideoProcessor<'a> {
    pub fn new() -> Result<Self> {
        which::which("ffprobe").map_err(|_| eyre!("ffprobe not installed"))?;
        which::which("ffmpeg").map_err(|_| eyre!("ffmpeg not installed"))?;
        let font = Font::try_from_bytes(FONT_BYTES).ok_or(eyre!("Unable to load font"))?;
        Ok(Self {
            font: Arc::new(font),
        })
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
            .stdout_str();
        let timestamp = timestamp
            .split_once('.')
            .ok_or(eyre!("Failed to split timestamp"))?
            .0;

        let parsed_time = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S")?;
        let parsed_time = Oslo.from_local_datetime(&parsed_time).unwrap();
        Ok(parsed_time.with_timezone(&Utc))
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

        let height = 150.0;
        let scale = Scale {
            x: height,
            y: height,
        };

        draw_text_mut(
            &mut img,
            WHITE,
            (size as f32 / 2.0 - (height / 4.0)) as i32,
            (size as f32 / 2.0 - (height / 2.0)) as i32,
            scale,
            &self.font,
            "⏵",
        );
        Ok(img)
    }
}
