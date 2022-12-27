use crate::messaging::host_messenger::HostMessenger;
use crate::messaging::messenger::Messenger;
use gtk::prelude::TextBufferExt;
use gtk::{prelude::*, Button, Entry, Label, TextBuffer, TextView};
use std::cell::RefCell;
use std::rc::Rc;

pub fn send_message(
    is_info: bool,
    text: &String,
    chat_text: &str,
    chat_id: &String,
    is_sent: bool,
    default_host_messenger: &Rc<RefCell<Option<HostMessenger>>>,
    default_messenger: &Rc<RefCell<Messenger>>,
    buffer: &TextBuffer,
    status_label: &Label,
    text_viewer: &TextView,
    send_message_button: &Button,
    message_entry: &Entry,
) {
    let (_, mut end) = buffer.bounds();

    let mut host_messenger = default_host_messenger.borrow_mut();

    if host_messenger.is_some() {
        let host_messenger = host_messenger.as_mut().unwrap();

        if is_sent {
            let errors = host_messenger.send_message(text);

            for err in errors {
                // TODO: show in dialog that error happened
                println!("ERROR: {}", err)
            }
        } else {
            let errors = host_messenger.send_broadcast_message(text, chat_id);

            for err in errors {
                // TODO: show in dialog that error happened
                println!("ERROR: {}", err)
            }
        }

        let ammount_connected = host_messenger.get_ammount_connected();

        status_label.set_text(format!("{} connected", ammount_connected).as_str());

        if is_info || ammount_connected > 0 {
            buffer.insert_markup(&mut end, chat_text);

            text_viewer.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
        }

        if ammount_connected == 0 {
            send_message_button.set_sensitive(false);
            message_entry.set_sensitive(false);
        }
    } else {
        let mut messenger = default_messenger.borrow_mut();

        if is_sent {
            let error = messenger.send_message(text);

            if let Some(err) = error {
                // TODO: show in dialog that error happened
                println!("ERROR: {}", err)
            }
        }

        let has_host = messenger.client.is_some();

        if is_info || has_host {
            buffer.insert_markup(&mut end, chat_text);

            text_viewer.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
        }

        if has_host {
            status_label.set_text("Connected to host");
        } else {
            send_message_button.set_sensitive(false);
            message_entry.set_sensitive(false);

            status_label.set_text("Not connected");
        }
    }
}
