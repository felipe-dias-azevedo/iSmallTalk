use crate::app::main_window::MainWindow;
use gtk::gio::ApplicationFlags;
use gtk::glib::clone;
use gtk::prelude::*;

pub struct MainApplication {
    app: gtk::Application,
}

impl MainApplication {
    pub fn new() -> Self {
        MainApplication {
            app: gtk::Application::new(
                Some("com.felipe.iSmallTalk"),
                ApplicationFlags::HANDLES_OPEN,
            ),
        }
    }

    pub fn on_activate(&self, main_window: &MainWindow) {
        self.app
            .connect_activate(clone!(@strong main_window.window as window => move |app| {
                if let Some(existing_window) = app.active_window() {
                    existing_window.present();
                } else {
                    window.set_application(Some(app));
                    app.add_window(&window);
                }
            }));
    }
}
