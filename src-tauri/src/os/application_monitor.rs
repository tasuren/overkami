use std::{ffi::OsStr, path::PathBuf};

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

pub struct ApplicationMonitor {
    system: System,
}

impl ApplicationMonitor {
    pub fn new() -> Self {
        Self {
            system: get_system(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            get_process_specifics(),
        );
    }

    pub fn get_application_processes(&self) -> impl Iterator<Item = ApplicationProcess> + '_ {
        self.system
            .processes()
            .iter()
            .filter_map(|(pid, process)| ApplicationProcess::from_sysinfo(pid.as_u32(), process))
    }

    pub fn get_application_process(&self, pid: u32) -> Option<ApplicationProcess> {
        self.system
            .process(sysinfo::Pid::from_u32(pid))
            .and_then(|process| ApplicationProcess::from_sysinfo(pid, process))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationProcess {
    pub pid: u32,
    pub name: Option<String>,
    /// Executable path. For macOS, this can be the `.app` bundle path.
    pub path: PathBuf,
}

impl ApplicationProcess {
    pub fn new(pid: u32, name: Option<String>, executable_path: PathBuf) -> Self {
        let path = if cfg!(target_os = "macos") {
            utils::get_application_path_by_exe(&executable_path).unwrap_or(executable_path)
        } else {
            executable_path
        };

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

mod utils {
    #[cfg(target_os = "macos")]
    use std::path::{Path, PathBuf};

    #[cfg(target_os = "macos")]
    pub fn get_application_path_by_exe(exe: impl AsRef<Path>) -> Option<PathBuf> {
        for ancestor in exe.as_ref().ancestors() {
            if ancestor.extension().is_some_and(|ext| ext == "app") {
                return Some(ancestor.to_owned());
            }
        }

        None
    }
}
