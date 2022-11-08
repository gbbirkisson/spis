use async_channel::{Receiver, Sender};
use std::path::PathBuf;
use walkdir::WalkDir;

trait HasExt {
    fn has_ext(&self, ext: &[String]) -> bool;
}

impl HasExt for walkdir::DirEntry {
    fn has_ext(&self, ext: &[String]) -> bool {
        match self.file_name().to_str() {
            None => (),
            Some(name) => {
                for e in ext {
                    if name.ends_with(e) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub fn start_walker(ext: Vec<String>) -> (Sender<PathBuf>, Receiver<walkdir::DirEntry>) {
    let (path_s, path_r): (Sender<PathBuf>, Receiver<PathBuf>) = async_channel::bounded(1);
    let (entry_s, entry_r) = async_channel::unbounded();

    tokio::spawn(async move {
        let ext = ext;
        loop {
            match path_r.recv().await {
                Ok(path) => {
                    tracing::info!("Walking path {}", path.to_string_lossy());

                    let walk = WalkDir::new(path)
                        .into_iter()
                        .filter_map(|r| r.ok())
                        .filter(|e| e.has_ext(&ext));

                    for entry in walk {
                        tracing::info!("Sending entry to channel {:?}", entry);
                        entry_s
                            .send(entry)
                            .await
                            .expect("unbounded channel should not fill up")
                    }
                }
                Err(e) => {
                    tracing::warn!("Error on walker PathBuf channel {}", e);
                }
            }
        }
    });

    (path_s, entry_r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ctor::ctor]
    fn setup() {
        tracing_subscriber::fmt::init();
    }

    #[tokio::test]
    async fn img_walker_walk() {
        let (path_s, entry_r) = start_walker(vec!["png".to_string()]);
        let path = std::fs::canonicalize("..").unwrap();
        path_s.send(path).await.unwrap();
        let entry = entry_r.recv().await.unwrap();
        tracing::info!("{:?}", entry);
        assert_eq!(entry.depth(), 1)
    }
}
