use crate::db;
use crate::media::MediaProcessor;
use crate::media::{self, ProcessedMedia};
use async_cron_scheduler::{Job, Scheduler};
use chrono::Local;
use chrono::{Duration, Utc};
use color_eyre::Result;
use notify::{
    event::{AccessKind, EventKind, RemoveKind},
    Config, Error, Event, RecommendedWatcher, Watcher,
};
use rayon::prelude::*;
use sqlx::{Pool, Sqlite};
use std::{collections::HashSet, path::PathBuf};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;
use uuid::Uuid;
use walkdir::WalkDir;

pub fn setup_filewatcher(
    file_sender: Sender<(Option<Uuid>, PathBuf)>,
) -> Result<RecommendedWatcher> {
    let watcher = RecommendedWatcher::new(
        move |event: Result<Event, Error>| {
            if let Ok(event) = event {
                match event.kind {
                    EventKind::Access(kind) => match kind {
                        AccessKind::Close(_) => {
                            for path in event.paths {
                                file_sender.blocking_send((None, path)).unwrap();
                            }
                        }
                        _ => {}
                    },
                    EventKind::Remove(kind) => match kind {
                        RemoveKind::File => {
                            // TODO: Handle file deletions?
                        }
                        _ => {}
                    },
                    _ => {}
                };
            }
        },
        Config::default(),
    )?;
    Ok(watcher)
}

pub fn setup_filewalker(
    mut job_reciever: Receiver<()>,
    file_sender: Sender<(Option<Uuid>, PathBuf)>,
    media_dir: PathBuf,
    pool: Pool<Sqlite>,
) -> Result<()> {
    tracing::debug!("Setup file walker");

    // Get absolute path to media dir
    let media_root = std::fs::canonicalize(&media_dir)?;

    tokio::spawn(async move {
        while let Some(_) = job_reciever.recv().await {
            let start_time = Utc::now();
            tracing::info!("Media processing started");

            tracing::debug!("Mark entire database as unwalked");
            let mark = db::media_mark_unwalked(&pool).await;
            if mark.is_err() {
                tracing::error!("Failed marking media as unwalked: {:?}", &mark);
            }

            let uuid_map = db::media_hashmap(&pool)
                .await
                .expect("Failed to fetch all entries");

            tracing::debug!("Setup walk thread");
            let (tx, rx) = tokio::sync::oneshot::channel();
            let file_sender = file_sender.clone();
            let media_root = media_root.clone();

            tracing::debug!("Start walk thread");
            tokio::task::spawn_blocking(move || {
                for entry in WalkDir::new(&media_root) {
                    match entry {
                        Ok(entry) => {
                            let path = entry.into_path();
                            let uuid = uuid_map.get(path.to_str().unwrap()).cloned();
                            if let Err(error) = file_sender.blocking_send((uuid, path)) {
                                tracing::error!("Walk failed to send to channel: {}", error)
                            }
                        }
                        Err(error) => tracing::error!("Walk failed: {}", error),
                    }
                }
                tx.send(()).expect("Failed to trigger processing finish");
            });

            tracing::debug!("Wait for walk to finish");
            rx.await.unwrap();

            tracing::info!("Processing done, waiting for grace period");
            sleep(tokio::time::Duration::from_secs(10)).await;

            tracing::debug!("Update missing field in DB");
            if mark.is_ok() {
                db::media_mark_missing(&pool)
                    .await
                    .expect("Failed to mark missing");
            }

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

            let end_time = Utc::now();
            let diff = end_time - start_time;
            if diff > Duration::hours(1) {
                let hours = diff.num_hours();
                let minutes = (diff - Duration::hours(hours)).num_minutes();
                tracing::info!(
                    "Media processing ended after {} hours and {} minutes",
                    hours,
                    minutes,
                );
            } else if diff > Duration::minutes(1) {
                let minutes = diff.num_minutes();
                let seconds = (diff - Duration::minutes(minutes)).num_seconds();
                tracing::info!(
                    "Media processing ended after {} minutes and {} seconds",
                    minutes,
                    seconds,
                )
            } else {
                tracing::info!(
                    "Media processing ended after {} seconds",
                    diff.num_seconds()
                )
            }
        }
        panic!("job_channel closed");
    });

    tracing::debug!("Setup file done");
    Ok(())
}

struct RecieverIterator<T> {
    reciever: Receiver<T>,
}

impl<T> Iterator for RecieverIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.reciever.blocking_recv()
    }
}

pub fn setup_media_processing(
    file_reciever: Receiver<(Option<Uuid>, PathBuf)>,
    media_sender: Sender<ProcessedMedia>,
    thumb_dir: PathBuf,
) -> Result<()> {
    tracing::debug!("Setup media processing");

    // Convert reciever into iterator so we can parallelize with rayon
    let file_iterator = RecieverIterator {
        reciever: file_reciever,
    };

    let media_processor = MediaProcessor::new(thumb_dir);

    // Spawn a new thread that processes files
    tokio::task::spawn_blocking(move || {
        // TODO: Will collecting to a HashSet cause a memory leak?
        let _res: HashSet<()> = file_iterator
            .par_bridge()
            .filter(|(_, path)| {
                // Filter out hidden files
                !path
                    .components()
                    .into_iter()
                    .any(|c| c.as_os_str().to_str().unwrap().chars().nth(0).unwrap() == '.')
            })
            .filter_map(|(uuid, path)| {
                // Filter out files with no extension
                match path
                    .extension()
                    .map(|ext| ext.to_str().unwrap().to_lowercase())
                {
                    Some(ext) => Some((uuid, path, ext)),
                    None => None,
                }
            })
            .filter_map(|(uuid, path, ext)| match ext.as_str() {
                // Map extensions to media type
                "jpg" | "jpeg" => Some((uuid, path, media::ProcessedMediaType::Image)),
                "mov" | "mp4" => Some((uuid, path, media::ProcessedMediaType::Video)),
                _ => None,
            })
            .map(|(uuid, path, media_type)| {
                match media_processor.process(uuid, path, media_type) {
                    Ok(media) => {
                        if let Err(error) = media_sender.blocking_send(media) {
                            tracing::error!("Failed to send media to channel: {}", error);
                        };
                    }
                    Err(error) => tracing::error!("Failed processing media: {}", error),
                };

                // Return empty object to be collected
                ()
            })
            .collect();
        panic!("file_channel was closed");
    });

    tracing::debug!("Setup media processing done");
    Ok(())
}

pub fn setup_db_store(pool: Pool<Sqlite>, media_reciever: Receiver<ProcessedMedia>) -> Result<()> {
    tracing::debug!("Setup db store");

    tokio::spawn(async move {
        tracing::info!("Starting db processing component");

        let mut media_reciever = media_reciever;
        while let Some(media) = media_reciever.recv().await {
            if let Err(error) = db::media_insert(&pool, media).await {
                tracing::error!("Failed inserting media to db: {}", error);
            }
        }
        panic!("media_channel was closed");
    });

    tracing::debug!("Setup db store done");
    Ok(())
}

pub fn setup_cron(job_sender: Sender<()>, schedule: String) -> Result<()> {
    tracing::debug!("Setup cron job");

    tokio::spawn(async move {
        tracing::info!("Added processing schedule: {}", schedule);
        let (mut scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
        let job = Job::cron(&schedule).unwrap();
        scheduler.insert(job, move |_| {
            tracing::info!("Triggering cron job: {}", schedule);
            let job_sender = job_sender.clone();
            tokio::spawn(async move {
                if let Err(error) = job_sender.send(()).await {
                    tracing::error!("Failed triggering cron job: {}", error);
                }
            });
        });
        sched_service.await;
    });

    tracing::debug!("Setup cron job done");
    Ok(())
}
