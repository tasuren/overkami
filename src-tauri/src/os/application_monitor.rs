use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use sysinfo::{ProcessRefreshKind, RefreshKind, System};

fn get_process_specifics() -> ProcessRefreshKind {
    sysinfo::ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::Always)
}

fn get_system_specifics() -> RefreshKind {
    RefreshKind::nothing().with_processes(get_process_specifics())
}

fn get_system() -> System {
    System::new_with_specifics(get_system_specifics())
}

static SYSTEM: LazyLock<Mutex<System>> = LazyLock::new(|| Mutex::new(get_system()));

pub fn refresh_blocking() {
    SYSTEM.lock().unwrap().refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        get_process_specifics(),
    );
}

pub fn get_application_processes<T>(mut filter: impl FnMut(&Path) -> bool) -> T
where
    T: FromIterator<ApplicationProcess>,
{
    SYSTEM
        .lock()
        .unwrap()
        .processes()
        .iter()
        .filter_map(|(pid, process)| ApplicationProcess::from_sysinfo(pid.as_u32(), process))
        .filter(|app| filter(app.path.as_ref()))
        .collect()
}

pub fn get_application_process(pid: u32) -> Option<ApplicationProcess> {
    SYSTEM
        .lock()
        .unwrap()
        .process(sysinfo::Pid::from_u32(pid))
        .and_then(|process| ApplicationProcess::from_sysinfo(pid, process))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationProcess {
    pub pid: u32,
    pub name: Option<String>,
    /// Executable path. For macOS, this can be the `.app` bundle path.
    pub path: PathBuf,
}

impl ApplicationProcess {
    pub fn new(pid: u32, name: Option<String>, path: PathBuf) -> Self {
        Self { pid, name, path }
    }
}

impl ApplicationProcess {
    fn from_sysinfo(pid: u32, process: &sysinfo::Process) -> Option<Self> {
        let path = process.exe()?;
        let name = process
            .name()
            .to_str()
            .or_else(|| path.file_name().and_then(OsStr::to_str))
            .map(|name| name.to_owned());

        Some(Self::new(pid, name, path.to_owned()))
    }
}

pub mod auto_refresh {
    use std::{
        sync::{atomic, OnceLock},
        thread::JoinHandle,
    };

    static STOP_AUTO_REFRESH: atomic::AtomicBool = atomic::AtomicBool::new(false);
    static AUTO_REFRESH_TASK: OnceLock<JoinHandle<()>> = OnceLock::new();

    pub fn start() -> anyhow::Result<()> {
        if AUTO_REFRESH_TASK.get().is_some() {
            anyhow::bail!("Auto-refresh task is already running");
        }

        let handle = std::thread::spawn(|| loop {
            if STOP_AUTO_REFRESH.load(atomic::Ordering::Relaxed) {
                break;
            }

            super::refresh_blocking();

            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        });

        AUTO_REFRESH_TASK.set(handle).unwrap();

        Ok(())
    }
}
