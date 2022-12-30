mod app;
mod channel;
mod messaging;
mod networking;

use std::{cell::RefCell, rc::Rc};

use gtk::glib;

use crate::app::main_application::MainApplication;
use crate::app::main_window::MainWindow;
use crate::channel::system_action::SystemAction;
use crate::channel::{listener_thread, receiver_thread};
use crate::messaging::host_messenger::HostMessenger;
use crate::networking::{local_ip, server};
use messaging::messenger::Messenger;

fn main() {
    let (tx_sys, rx_sys) = glib::MainContext::channel::<SystemAction>(glib::PRIORITY_DEFAULT);

    let ip = local_ip::get();
    let (server, port) = server::bind_ip_port(&ip);

    let default_messenger = Messenger::new(ip, port);
    let default_host_messenger: Option<HostMessenger> = None;
    let default_host_messenger = Rc::new(RefCell::new(default_host_messenger));

    let id_messenger = default_messenger.get_id();
    let messenger = Rc::new(RefCell::new(default_messenger));

    let actual_text = Rc::new(RefCell::new(String::from("")));

    gtk::init().expect("GTK failed");

    let app = MainApplication::new();

    let builder_main_window = gtk::Builder::from_file("templates/ismalltalk-main.glade");

    let main_window = MainWindow::new(&builder_main_window, &id_messenger);

    app.on_activate(&main_window);

    main_window.on_delete(&tx_sys);
    main_window.on_about_clicked(&builder_main_window);
    main_window.on_host_change(&tx_sys);
    main_window.on_text_change(&actual_text);
    main_window.on_entry_keyboard_press();
    main_window.on_send_clicked(&id_messenger, &actual_text, &tx_sys);
    main_window.on_add_client_clicked(&tx_sys);

    let _ = listener_thread::start(&id_messenger, &tx_sys, server);

    receiver_thread::start(rx_sys, &main_window, &messenger, &default_host_messenger);

    gtk::main();
}
