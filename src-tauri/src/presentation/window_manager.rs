use std::ops::Deref;

struct MainThreadSafe<T>(T);

unsafe impl<T> Send for MainThreadSafe<T> {}

impl<T> Deref for MainThreadSafe<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

pub struct WindowManager {
    windows: smallvec::SmallVec<[tauri::Window; 6]>,
}

impl WindowManager {
    pub fn new(app: &mut tauri::App) -> Self {
        let windows = Default::default();

        for (i, monitor) in app.available_monitors().unwrap().into_iter().enumerate() {
            let tauri::PhysicalSize { width, height } = monitor.size();
            let window = tauri::WebviewWindow::builder(
                app,
                format!("wallpaper-{i}"),
                tauri::WebviewUrl::App("nothing.html".into()),
            )
            .inner_size(*width as _, *height as _)
            .closable(false)
            .always_on_top(true)
            .transparent(true)
            .title_bar_style(tauri::TitleBarStyle::Transparent)
            .hidden_title(true)
            .decorations(false)
            .skip_taskbar(true)
            .visible_on_all_workspaces(true)
            .build()
            .unwrap();
            window.set_ignore_cursor_events(true).unwrap();
        }

        Self { windows }
    }
}
