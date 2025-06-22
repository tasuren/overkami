use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{atomic, LazyLock},
};

use smallvec::{smallvec, SmallVec};
use tauri::async_runtime::{Mutex, Sender};
use uuid::Uuid;

use crate::os::application_monitor::{get_application_processes, ApplicationProcess};

pub type AppPid = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationEvent {
    Added(AppPid),
    Removed(AppPid),
}

pub type ListenerTx = Sender<ApplicationEvent>;
pub type ApplicationListeners = HashMap<PathBuf, SmallVec<[(ListenerTx, Uuid); 4]>>;

static APPLICATION_LISTENERS: LazyLock<Mutex<ApplicationListeners>> =
    LazyLock::new(Default::default);
static OBSERVER_STARTED: atomic::AtomicBool = atomic::AtomicBool::new(false);

/// Registers a listener for application launch or dead events.
/// To unlisten, simply drop the [`Receiver`](tauri::async_runtime::Receiver).
pub async fn listen_application(tx: ListenerTx, target_app_path: PathBuf, wallpaper_id: Uuid) {
    if !OBSERVER_STARTED.load(atomic::Ordering::Relaxed) {
        OBSERVER_STARTED.store(true, atomic::Ordering::Relaxed);

        tauri::async_runtime::spawn(observe_applications());
    }

    let mut listeners = APPLICATION_LISTENERS.lock().await;

    if let Some(listeners) = listeners.get_mut(&target_app_path) {
        listeners.push((tx, wallpaper_id));
    } else {
        listeners.insert(target_app_path, smallvec![(tx, wallpaper_id)]);
    };
}

pub async fn unlisten_application(target_app_path: &PathBuf, wallpaper_id: Uuid) {
    let mut listeners = APPLICATION_LISTENERS.lock().await;

    if let Some(target_app_listeners) = listeners.get_mut(target_app_path) {
        let index = target_app_listeners
            .iter()
            .position(|(_, id)| *id == wallpaper_id);

        if let Some(index) = index {
            let _ = target_app_listeners.remove(index);

            if target_app_listeners.is_empty() {
                listeners.remove(target_app_path);
            }
        }
    }
}

/// Starts observing applications and sends diffs to the provided channel.
async fn observe_applications() {
    let mut previous_apps = HashSet::<ApplicationProcess>::new();

    loop {
        let current: HashSet<_> = {
            let listeners = APPLICATION_LISTENERS.lock().await;
            get_application_processes(|path| listeners.contains_key(path))
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
            let Some(listeners) = listeners.get_mut(&process.path) else {
                return;
            };

            let mut remove = SmallVec::<[usize; 3]>::new();
            for (i, (listener, _)) in listeners.iter().enumerate() {
                if listener.send(event).await.is_err() {
                    remove.push(i);
                };
            }

            // Remove listeners that is dead.
            for &i in remove.iter().rev() {
                listeners.remove(i);
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
