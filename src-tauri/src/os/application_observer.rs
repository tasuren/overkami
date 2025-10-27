use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, atomic},
};

use smallvec::SmallVec;
use tauri::async_runtime::{Mutex, Sender};
use uuid::Uuid;

use crate::os::application_monitor::{ApplicationProcess, get_application_processes};

pub type AppPid = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationEvent {
    Added(AppPid),
    Removed(AppPid),
}

pub type ListenerTx = Sender<ApplicationEvent>;
pub type WallpaperTxMap = HashMap<Uuid, ListenerTx>;
pub type ApplicationListeners = HashMap<String, WallpaperTxMap>;

static APPLICATION_LISTENERS: LazyLock<Mutex<ApplicationListeners>> =
    LazyLock::new(Default::default);
static OBSERVER_STARTED: atomic::AtomicBool = atomic::AtomicBool::new(false);

/// Registers a listener for application launch or dead events.
/// To unlisten, simply drop the [`Receiver`](tauri::async_runtime::Receiver).
pub async fn listen_application(tx: ListenerTx, target_app_path: String, wallpaper_id: Uuid) {
    if !OBSERVER_STARTED.load(atomic::Ordering::Relaxed) {
        OBSERVER_STARTED.store(true, atomic::Ordering::Relaxed);

        tauri::async_runtime::spawn(observe_applications());
    }

    let mut listeners = APPLICATION_LISTENERS.lock().await;

    if let Some(tx_map) = listeners.get_mut(&target_app_path) {
        tx_map.insert(wallpaper_id, tx);
    } else {
        listeners.insert(target_app_path, HashMap::from([(wallpaper_id, tx)]));
    };
}

pub async fn unlisten_application(target_app_path: &str, wallpaper_id: Uuid) -> Option<ListenerTx> {
    let mut listeners = APPLICATION_LISTENERS.lock().await;

    if let Some(tx_map) = listeners.get_mut(target_app_path) {
        let tx = tx_map.remove(&wallpaper_id);

        if tx_map.is_empty() {
            listeners.remove(target_app_path);
        }

        tx
    } else {
        None
    }
}

/// Starts observing applications and sends diffs to the provided channel.
async fn observe_applications() {
    let mut previous_apps = HashSet::<ApplicationProcess>::new();

    loop {
        let current: HashSet<_> = {
            let listeners = APPLICATION_LISTENERS.lock().await;
            get_application_processes(|path| {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    listeners.contains_key(name)
                } else {
                    false
                }
            })
        };

        // Calculate the difference.
        let added = current
            .difference(&previous_apps)
            .cloned()
            .collect::<Vec<ApplicationProcess>>();
        let removed = previous_apps
            .difference(&current)
            .cloned()
            .collect::<Vec<ApplicationProcess>>();
        previous_apps = current;

        // Send events to listeners.
        async fn send_event(event: ApplicationEvent, process: &ApplicationProcess) {
            let mut listeners = APPLICATION_LISTENERS.lock().await;
            let Some(name) = process.path.file_name().and_then(|n| n.to_str()) else {
                return;
            };
            let Some(tx_map) = listeners.get_mut(name) else {
                return;
            };

            let mut remove = SmallVec::<[Uuid; 3]>::new();
            for (key, tx) in tx_map.iter() {
                if tx.send(event).await.is_err() {
                    remove.push(*key);
                };
            }

            // Remove listeners that is dead.
            for key in remove.iter() {
                tx_map.remove(key);
            }
        }

        for process in added {
            send_event(ApplicationEvent::Added(process.pid), &process).await;
        }

        for process in removed {
            send_event(ApplicationEvent::Removed(process.pid), &process).await;
        }

        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
    }
}
