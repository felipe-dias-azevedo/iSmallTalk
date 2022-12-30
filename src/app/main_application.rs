use crate::app::main_window::MainWindow;
use gtk::gio::{ApplicationFlags, Notification, Cancellable, NotificationPriority};
use gtk::glib::clone;
use gtk::prelude::*;

static APP_ID: Option<&str> = Some("test.iSmallTalk");

pub struct MainApplication {
    app: gtk::Application
}

impl MainApplication {
    pub fn new() -> Self {
        let main_application = MainApplication {
            app: gtk::Application::new(
                APP_ID,
                ApplicationFlags::HANDLES_OPEN,
            )
        };
        
        main_application.app.register(Cancellable::NONE).expect("Register failed");
        //main_application.app.activate();

        main_application
    }

    pub fn send_notification(&self, title: &str, text: &str) {
        let notification = Notification::new(title);
        notification.set_priority(NotificationPriority::Low);
        notification.set_body(Some(text));
        
        self.app.send_notification(APP_ID, &notification);
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

    pub fn start(&self) {
        self.app.run();
    }
}
