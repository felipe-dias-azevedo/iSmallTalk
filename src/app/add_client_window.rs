use crate::channel::system_action::SystemAction;
use crate::networking::client;
use gtk::glib::{clone, Sender};
use gtk::prelude::*;

const ADD_CLIENT_WINDOW: &'static str = include_str!("./templates/ismalltalk-addclient.glade");

pub struct AddClientWindow {
    window: gtk::Window,
    addclient_entry: gtk::Entry,
    addclient_button: gtk::Button,
    addclient_label: gtk::Label,
}

impl AddClientWindow {
    pub fn new(main_window: &gtk::Window, sender: &Sender<SystemAction>) -> Self {
        let builder = gtk::Builder::from_string(ADD_CLIENT_WINDOW);

        let add_client_window = AddClientWindow {
            window: builder
                .object("addclient-window")
                .expect("Couldn't set add client window"),
            addclient_entry: builder
                .object("addclient-entry")
                .expect("Couldn't set addclient-entry"),
            addclient_button: builder
                .object("addclient-button")
                .expect("Couldn't set addclient-button"),
            addclient_label: builder
                .object("addclient-label")
                .expect("Couldn't set addclient-label"),
        };

        add_client_window.on_add_client_clicked(sender);
        add_client_window.on_delete(main_window);

        add_client_window.window.show_all();

        add_client_window
    }

    fn on_add_client_clicked(&self, sender: &Sender<SystemAction>) {
        self.addclient_button.connect_clicked(clone!(
            @strong self.addclient_entry as addclient_entry,
            @strong self.addclient_label as addclient_label
            @strong sender
            => move |_| {
                let text = addclient_entry.text();
                let text = text.as_str();

                let client = client::validate_ip(text);

                if let Some(client_error) = client.as_ref().err() {
                    addclient_label.set_text(client_error.0);
                    return;
                }

                println!("Client {} connected", text);

                let (ip, port) = client::id_to_ip_port(&text.to_string());

                let requested_add_client = sender
                    .send(SystemAction::RequestAddClient(ip, port, client.unwrap()))
                    .ok();

                if requested_add_client.is_some() {
                    addclient_entry.set_text("");
                    addclient_label.set_text("Username added sucessfully!");
                }
        }));
    }

    fn on_delete(&self, main_window: &gtk::Window) {
        self.window
            .connect_delete_event(clone!(@strong main_window => move |window, _| {
                main_window.set_accept_focus(true);

                window.hide_on_delete();

                Inhibit(false)
            }));
    }
}
