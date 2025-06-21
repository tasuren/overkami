use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use tauri::async_runtime::Sender;

use crate::os::ApplicationProcess;

pub struct ApplicationDiff {
    pub added: Vec<ApplicationProcess>,
    pub removed: Vec<ApplicationProcess>,
}

// A set of application paths to filter applications.
type ApplicationFilter = Arc<Mutex<HashSet<PathBuf>>>;

pub struct ApplicationObserver {
    sys_updater: JoinHandle<()>,
}

impl ApplicationObserver {
    pub fn start(tx: Sender<ApplicationDiff>, filter: ApplicationFilter) -> Self {
        let sys_updater = std::thread::spawn({
            let filter = filter.clone();

            move || {
                observe_applications(tx, filter);
            }
        });

        Self { sys_updater }
    }

    pub fn stop(self) {
        if let Err(error) = self.sys_updater.join() {
            eprintln!("Application Observer panicked: {:?}", error);
        };
    }
}

/// Starts observing applications and sends diffs to the provided channel.
fn observe_applications(tx: Sender<ApplicationDiff>, filter: ApplicationFilter) {
    let mut monitor = crate::os::ApplicationMonitor::new();

    let mut previous_apps = HashSet::<ApplicationProcess>::new();

    loop {
        monitor.refresh();

        let filter = filter.lock().unwrap();
        let current = monitor
            .get_application_processes()
            .filter(|process| filter.contains(process.path.as_path()))
            .collect::<HashSet<_>>();

        // Calculate the difference
        let added = current
            .difference(&previous_apps)
            .cloned()
            .collect::<Vec<ApplicationProcess>>();
        let removed = previous_apps
            .difference(&current)
            .cloned()
            .collect::<Vec<ApplicationProcess>>();
        previous_apps = current;

        tx.blocking_send(ApplicationDiff { added, removed })
            .unwrap();
    }
}
