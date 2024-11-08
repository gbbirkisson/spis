use crate::db;
use crate::media::MediaProcessor;
use crate::media::{self, ProcessedMedia};
use crate::prelude::*;
use async_cron_scheduler::{Job, Scheduler};
use chrono::Local;
use chrono::{DateTime, Duration, Utc};
use color_eyre::eyre::Context;
use color_eyre::Result;
use notify::event::ModifyKind;
use notify::{
    event::{AccessKind, CreateKind, EventKind},
    Config, Error, Event, RecommendedWatcher, Watcher,
};
use rayon::prelude::*;
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::{collections::HashSet, path::PathBuf};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::sleep;
use uuid::Uuid;
use walkdir::WalkDir;

const DEBOUNCE_SECONDS: u64 = 5;
pub const NOTHING: () = ();
pub const JOB_TRIGGER: () = ();

pub type Nothing = ();
pub type JobTrigger = ();
pub type File = (Option<Uuid>, PathBuf);

pub fn setup_filewatcher(file_sender: Sender<File>) -> Result<RecommendedWatcher> {
    // Setup debouncer channel
    let (debouncer_sender, mut debouncer_receiver): (
        Sender<Option<PathBuf>>,
        Receiver<Option<PathBuf>>,
    ) = channel(1);

    // Trigger debouncer every DEBOUNCE_SECONDS / 2
    let debouncer_sender_trigger = debouncer_sender.clone();
    tokio::spawn(async move {
        loop {
            if let Err(error) = debouncer_sender_trigger.send(None).await {
                tracing::error!("Failed to send trigger to debounce channel: {:?}", error);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(DEBOUNCE_SECONDS / 2)).await;
        }
    });

    // Debouncer logic
    tokio::spawn(async move {
        let mut debounced_values: HashMap<PathBuf, DateTime<Utc>> = HashMap::new();

        while let Some(optional_path) = debouncer_receiver.recv().await {
            if let Some(path) = optional_path {
                debounced_values.insert(path, Utc::now());
            } else {
                let now = Utc::now();
                let retain = |_: &PathBuf, time: &mut DateTime<Utc>| {
                    now - *time
                        < Duration::seconds(
                            DEBOUNCE_SECONDS
                                .try_into()
                                .expect("Convert DEBOUNCE_SECONDS should not fail"),
                        )
                };
                for (path, time) in &mut debounced_values {
                    if !retain(path, time) {
                        tracing::debug!("Triggering processing for: {:?}", path);
                        if let Err(error) = file_sender
                            .send((None, path.clone()))
                            .await
                            .wrap_err("send failed")
                        {
                            tracing::error!("Failed to send file to channel: {:?}", error);
                        }
                    }
                }
                debounced_values.retain(retain);
            }
        }
    });

    // Setup file watcher
    let file_watcher = RecommendedWatcher::new(
        move |event: Result<Event, Error>| {
            if let Ok(event) = event {
                tracing::trace!("Got file event: {:?}", event);
                let trigger_processing = match event.kind {
                    EventKind::Create(CreateKind::File)
                    | EventKind::Access(AccessKind::Close(_))
                    | EventKind::Modify(ModifyKind::Name(_)) => true,
                    // EventKind::Remove(RemoveKind::File) => {
                    //     // TODO: Handle file deletions?
                    // }
                    _ => false,
                };
                if trigger_processing {
                    for path in event.paths {
                        if let Err(error) = debouncer_sender
                            .blocking_send(Some(path))
                            .wrap_err("blocking_send failed")
                        {
                            tracing::error!("Failed to send file to debounce channel: {:?}", error);
                        }
                    }
                }
            }
        },
        Config::default(),
    )?;

    Ok(file_watcher)
}

pub fn setup_filewalker(
    pool: Pool<Sqlite>,
    media_dir: PathBuf,
    file_sender: Sender<File>,
    follow_symlinks: bool,
) -> Result<Sender<JobTrigger>> {
    tracing::debug!("Setup file walker");

    let (job_sender, mut job_receiver) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        while job_receiver.recv().await.is_some() {
            let time_start = Utc::now();
            tracing::info!("Media processing started");

            tracing::debug!("Mark entire database as unwalked");
            let db_mark_missing = match db::media_mark_unwalked(&pool).await {
                Ok(()) => true,
                Err(error) => {
                    tracing::error!("Failed marking media as unwalked: {:?}", error);
                    false
                }
            };

            let old_uuids = match db::media_hashmap(&pool).await {
                Ok(map) => map,
                Err(error) => {
                    tracing::error!("Failed to get old entries: {:?}", error);
                    HashMap::with_capacity(0)
                }
            };

            tracing::debug!("Setup walk thread");
            let (walk_finished_sender, walk_finished_receiver) = tokio::sync::oneshot::channel();

            let file_sender = file_sender.clone();
            let media_dir = media_dir.clone();

            tracing::debug!("Start walk thread");
            tokio::task::spawn_blocking(move || {
                walk_dir(&old_uuids, &media_dir, &file_sender, follow_symlinks);
                if let Err(error) = walk_finished_sender.send(NOTHING) {
                    tracing::error!("Failed to trigger processing finish: {:?}", error);
                };
            });

            tracing::debug!("Wait for walk to finish");
            if let Err(error) = walk_finished_receiver.await {
                tracing::error!("Failed to trigger processing finish: {:?}", error);
            };

            tracing::info!("Walking done, waiting for grace period");
            sleep(tokio::time::Duration::from_secs(10)).await;

            tracing::debug!("Update missing field in DB");
            if db_mark_missing {
                if let Err(error) = db::media_mark_missing(&pool).await {
                    tracing::error!("Failed marking media as walked: {:?}", error);
                }
            }

            // Print counts
            if let Ok(counts) = db::media_count(&pool).await {
                tracing::info!("DB counts total:    {}", counts.count);
                if let Some(c) = counts.walked {
                    tracing::info!("DB counts walked:   {}", c);
                }
                if let Some(c) = counts.favorite {
                    tracing::info!("DB counts favorite: {}", c);
                }
                if let Some(c) = counts.archived {
                    tracing::info!("DB counts archived: {}", c);
                }
                if let Some(c) = counts.missing {
                    tracing::info!("DB counts missing:  {}", c);
                }
            }

            // Print duration
            let time_end = Utc::now();
            let time_diff = time_end - time_start;
            if time_diff > Duration::hours(1) {
                let hours = time_diff.num_hours();
                let minutes = (time_diff - Duration::hours(hours)).num_minutes();
                tracing::info!(
                    "Media processing ended after {} hours and {} minutes",
                    hours,
                    minutes,
                );
            } else if time_diff > Duration::minutes(1) {
                let minutes = time_diff.num_minutes();
                let seconds = (time_diff - Duration::minutes(minutes)).num_seconds();
                tracing::info!(
                    "Media processing ended after {} minutes and {} seconds",
                    minutes,
                    seconds,
                );
            } else {
                tracing::info!(
                    "Media processing ended after {} seconds",
                    time_diff.num_seconds()
                );
            }
        }
        tracing::warn!("job_channel closed");
    });

    tracing::debug!("Setup file done");
    Ok(job_sender)
}

#[allow(clippy::cognitive_complexity)]
fn walk_dir(
    old_uuids: &HashMap<String, Uuid>,
    media_dir: &PathBuf,
    file_sender: &Sender<(Option<Uuid>, PathBuf)>,
    follow_symlinks: bool,
) {
    let mut count = 0;
    for entry in WalkDir::new(media_dir).follow_links(follow_symlinks) {
        count += 1;
        if count % 1000 == 0 {
            tracing::info!("Walked {} files so far ...", count);
        }
        match entry.wrap_err("Failed to walk") {
            Ok(entry) => {
                let path = entry.into_path();
                let path_string: String = W(&path).into();
                let uuid = old_uuids.get(&path_string).copied();
                if let Err(error) = file_sender
                    .blocking_send((uuid, path))
                    .wrap_err("blocking_send failed")
                {
                    tracing::error!("Walk failed to send to channel: {:?}", error);
                }
            }
            Err(error) => tracing::error!("Walk failed: {:?}", error),
        }
    }
    tracing::info!("Walked {} files in total", count);
}

pub fn setup_media_processing(
    thumb_dir: PathBuf,
    allow_no_exif: bool,
    force_processing: bool,
) -> Result<(Sender<File>, Receiver<ProcessedMedia>)> {
    tracing::debug!("Setup media processing");

    let (file_sender, file_receiver): (Sender<File>, Receiver<File>) =
        tokio::sync::mpsc::channel(rayon::current_num_threads());
    let (media_sender, media_receiver) = tokio::sync::mpsc::channel(rayon::current_num_threads());

    let media_processor = MediaProcessor::new(thumb_dir, force_processing);

    tokio::task::spawn_blocking(move || {
        // TODO: Will collecting to a HashSet cause a memory leak?
        let _res: HashSet<Nothing> = W(file_receiver)
            .into_iter()
            .par_bridge()
            .filter(|(_, path)| {
                // Filter out hidden files
                !path
                    .components()
                    .any(|c| String::from(W(c)).starts_with('.'))
            })
            .filter_map(|(uuid, path)| {
                // Filter out files with no extension
                path.extension()
                    .map(|ext| String::from(W(ext)).to_lowercase())
                    .map(|ext| (uuid, path, ext))
            })
            .filter_map(|(uuid, path, ext)| match ext.as_str() {
                // Map extensions to media type
                "avif" | "jpg" | "jpeg" | "png" | "apng" | "gif" | "webp" | "tif" | "tiff"
                | "bmp" => Some((uuid, path, media::ProcessedMediaType::Image)),
                "mov" | "mp4" => Some((uuid, path, media::ProcessedMediaType::Video)),
                _ => None,
            })
            .map(|(uuid, path, media_type)| {
                match media_processor.process(uuid, &path, media_type, allow_no_exif) {
                    Ok(media) => {
                        if let Err(error) = media_sender
                            .blocking_send(media)
                            .wrap_err("blocking_send failed")
                        {
                            tracing::error!("Failed to send media to channel: {:?}", error);
                        };
                    }
                    Err(error) => {
                        tracing::error!("Failed processing media: {:?} {:?}", &path, error);
                    }
                };
            })
            .collect();
        tracing::warn!("file_channel was closed");
    });

    tracing::debug!("Setup media processing done");
    Ok((file_sender, media_receiver))
}

pub fn setup_db_store(pool: Pool<Sqlite>, media_receiver: Receiver<ProcessedMedia>) -> Result<()> {
    tracing::debug!("Setup db store");

    tokio::spawn(async move {
        tracing::info!("Starting db processing component");

        let mut media_receiver = media_receiver;
        while let Some(media) = media_receiver.recv().await {
            if let Err(error) = db::media_insert(&pool, media).await {
                tracing::error!("Failed inserting media to db: {:?}", error);
            }
        }
        tracing::warn!("media_channel was closed");
    });

    tracing::debug!("Setup db store done");
    Ok(())
}

pub fn setup_cron(job_sender: Sender<()>, schedule: &str) -> Result<()> {
    tracing::debug!("Setup cron job");

    let schedule: String = schedule.to_string();
    let job = Job::cron(&schedule)?;

    tokio::spawn(async move {
        tracing::info!("Added processing schedule: {}", schedule);
        let (mut scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
        scheduler
            .insert(job, move |_| {
                tracing::info!("Triggering cron job: {}", schedule);
                let job_sender = job_sender.clone();
                tokio::spawn(async move {
                    if let Err(error) = job_sender.send(JOB_TRIGGER).await.wrap_err("send failed") {
                        tracing::error!("Failed triggering cron job: {:?}", error);
                    }
                });
            })
            .await;
        sched_service.await;
    });

    tracing::debug!("Setup cron job done");
    Ok(())
}
