use crate::models::MediaFile;
use crate::scanner::{is_media_file, create_media_file};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

/// 文件变更事件
#[derive(Debug)]
pub enum FileChangeEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

/// 文件监控器
pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    watched_paths: Vec<PathBuf>,
    event_tx: Option<mpsc::Sender<FileChangeEvent>>,
    media_files: Arc<Mutex<HashMap<PathBuf, MediaFile>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            watched_paths: Vec::new(),
            event_tx: None,
            media_files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 启动文件监控
    pub async fn start_watching(
        &mut self,
        paths: Vec<PathBuf>,
        on_change: impl Fn(FileChangeEvent) + Send + 'static,
    ) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
        self.event_tx = Some(tx.clone());

        // 创建文件监控器
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) => {
                            for path in event.paths {
                                if is_media_file(&path) {
                                    let _ = tx.blocking_send(FileChangeEvent::Created(path));
                                }
                            }
                        }
                        EventKind::Modify(_) => {
                            for path in event.paths {
                                if is_media_file(&path) {
                                    let _ = tx.blocking_send(FileChangeEvent::Modified(path));
                                }
                            }
                        }
                        EventKind::Remove(_) => {
                            for path in event.paths {
                                let _ = tx.blocking_send(FileChangeEvent::Deleted(path));
                            }
                        }
                        _ => {}
                    }
                }
            },
            notify::Config::default(),
        )?;

        // 监控指定目录
        for path in &paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                info!("Started watching directory: {:?}", path);
            } else {
                warn!("Path does not exist, skipping: {:?}", path);
            }
        }

        self.watcher = Some(watcher);
        self.watched_paths = paths;

        // 处理文件变更事件
        let media_files = self.media_files.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match &event {
                    FileChangeEvent::Created(path) => {
                        info!("New media file detected: {:?}", path);
                        if let Ok(media_file) = create_media_file(path).await {
                            let mut files = media_files.lock().await;
                            files.insert(path.clone(), media_file);
                        }
                    }
                    FileChangeEvent::Modified(path) => {
                        info!("Media file modified: {:?}", path);
                        if let Ok(media_file) = create_media_file(path).await {
                            let mut files = media_files.lock().await;
                            files.insert(path.clone(), media_file);
                        }
                    }
                    FileChangeEvent::Deleted(path) => {
                        info!("Media file deleted: {:?}", path);
                        let mut files = media_files.lock().await;
                        files.remove(path);
                    }
                }

                // 调用回调函数
                on_change(event);
            }
        });

        Ok(())
    }

    /// 停止文件监控
    pub fn stop_watching(&mut self) {
        self.watcher = None;
        self.event_tx = None;
        info!("File watcher stopped");
    }

    /// 获取当前监控的媒体文件
    pub async fn get_media_files(&self) -> Vec<MediaFile> {
        let files = self.media_files.lock().await;
        files.values().cloned().collect()
    }

    /// 添加监控路径
    pub async fn add_watch_path(&mut self, path: PathBuf) -> anyhow::Result<()> {
        if let Some(ref mut watcher) = self.watcher {
            if path.exists() {
                watcher.watch(&path, RecursiveMode::Recursive)?;
                self.watched_paths.push(path.clone());
                info!("Added watch path: {:?}", path);
            } else {
                warn!("Path does not exist: {:?}", path);
            }
        }
        Ok(())
    }

    /// 移除监控路径
    pub async fn remove_watch_path(&mut self, path: &Path) -> anyhow::Result<()> {
        if let Some(ref mut watcher) = self.watcher {
            watcher.unwatch(path)?;
            self.watched_paths.retain(|p| p != path);
            info!("Removed watch path: {:?}", path);
        }
        Ok(())
    }

    /// 获取监控的路径列表
    pub fn get_watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.stop_watching();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_watcher() {
        let dir = tempdir().unwrap();
        let mut watcher = FileWatcher::new();

        let (tx, _rx) = mpsc::channel(10);
        let tx_clone = tx.clone();

        watcher
            .start_watching(vec![dir.path().to_path_buf()], move |event| {
                let _ = tx_clone.blocking_send(event);
            })
            .await
            .unwrap();

        assert_eq!(watcher.get_watched_paths().len(), 1);
    }
}
