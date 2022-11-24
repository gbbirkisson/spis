use chrono::prelude::*;
use chrono::{DateTime, Utc};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub static THUMBNAIL_FORMAT: &str = "webp";

pub(crate) trait Thumbnail {
    fn get_thumbnail(&self, uuid: &Uuid) -> PathBuf;
}

impl Thumbnail for PathBuf {
    fn get_thumbnail(&self, uuid: &Uuid) -> PathBuf {
        let mut res = self.join(uuid.to_string());
        res.set_extension(THUMBNAIL_FORMAT);
        res
    }
}

trait TimeConv {
    fn into(&self) -> DateTime<Utc>;
}

impl TimeConv for SystemTime {
    fn into(&self) -> DateTime<Utc> {
        let (sec, nsec) = match self.duration_since(UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
            Err(e) => {
                let dur = e.duration();
                let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
                if nsec == 0 {
                    (-sec, 0)
                } else {
                    (-sec - 1, 1_000_000_000 - nsec)
                }
            }
        };
        Utc.timestamp_opt(sec, nsec).unwrap()
    }
}
