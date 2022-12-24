mod channel;
mod messaging;
mod networking;

use std::borrow::Borrow;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};

use gtk::{
    gdk::keys::constants::{ISO_Enter, KP_Enter, Return, _3270_Enter},
    gio::ApplicationFlags,
    glib,
    prelude::*,
};
use regex::Regex;

use crate::channel::chat_info;
use crate::channel::chat_info::TypeSystem;
use crate::networking::client;
use channel::chat_info::{ChatInfo, TypeChat};
use channel::chat_message::ChatMessage;
use channel::window_info::WindowInfo;
use messaging::messenger::Messenger;

fn main() {
    let (tx, rx) = glib::MainContext::channel::<WindowInfo>(glib::PRIORITY_DEFAULT);

    let actual_text = Rc::new(RefCell::new(String::from("")));

    gtk::init().expect("GTK failed");

    let app = gtk::Application::new(
        Some("com.felipe.iSmallTalk"),
        ApplicationFlags::HANDLES_OPEN,
    );

    let builder = gtk::Builder::from_file("src/windows/ismalltalk-main.glade");
    let window: gtk::Window = builder.object("main-window").expect("Couldn't set window");

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let window_clone = window.clone();
    app.connect_activate(move |app| {
        if let Some(existing_window) = app.active_window() {
            existing_window.present();
        } else {
            window_clone.set_application(Some(app));
            app.add_window(&window_clone);
        }
    });

    let hostcheck: gtk::CheckButton = builder
        .object("main-hostcheck")
        .expect("Couldn't get main-hostcheck");

    let textview: gtk::TextView = builder
        .object("main-textview")
        .expect("Couldn't get main-textview");

    let sendbutton: gtk::Button = builder
        .object("main-sendbutton")
        .expect("Couldn't get main-sendbutton");

    let entrytext: gtk::Entry = builder
        .object("main-entrytext")
        .expect("Couldn't get main-entrytext");

    let leavebutton: gtk::Button = builder
        .object("main-leavebutton")
        .expect("Couldn't get main-leavebutton");
    leavebutton.hide();

    let statuslabel: gtk::Label = builder
        .object("main-statuslabel")
        .expect("Couldn't get main-statuslabel");

    let loaderspinner: gtk::Spinner = builder
        .object("main-loader")
        .expect("Couldn't get main-loader");

    let addbutton: gtk::Button = builder
        .object("main-addbutton")
        .expect("Couldn't get main-addbutton");

    let connectbutton: gtk::Button = builder
        .object("main-connectbutton")
        .expect("Couldn't get main-connectbutton");
    connectbutton.hide();

    let menupopover: gtk::Popover = builder
        .object("main-menupopover")
        .expect("Couldn't get main-menupopover");

    let aboutbutton: gtk::Button = builder
        .object("main-aboutbutton")
        .expect("Couldn't get main-aboutbutton");

    aboutbutton.connect_clicked(move |_| {
        let aboutdialog: gtk::AboutDialog = builder
            .object("main-aboutdialog")
            .expect("Couldn't get main-aboutdialog");

        aboutdialog.show_all();
        menupopover.hide();

        aboutdialog.connect_delete_event(move |x, _| x.hide_on_delete());
    });

    let is_host = hostcheck.is_active();
    if !is_host {
        addbutton.hide();
    }
    // else {
    //     connectbutton.hide();
    // }

    let (mess, mess_server) = Messenger::new(is_host);

    let id_messenger = mess.get_id();
    let messenger = Rc::new(RefCell::new(mess));

    let addbutton_clone = addbutton.clone();
    let messenger_clone = Rc::clone(&messenger);
    hostcheck.connect_clicked(move |a| {
        if a.is_active() {
            addbutton_clone.show();
            // connectbutton.hide();

            messenger_clone.borrow_mut().change_type(true);
        } else {
            addbutton_clone.hide();
            // connectbutton.show();

            messenger_clone.borrow_mut().change_type(false);
        }
    });

    let buffer = textview.buffer().unwrap();

    let actual_text_clone_entrytext = Rc::clone(&actual_text);
    entrytext.connect_changed(move |e| {
        let mut actual_text = actual_text_clone_entrytext.borrow_mut();
        let new_text = e.text();
        let new_text = new_text.as_str();
        *actual_text = String::from(new_text);
        e.set_text(new_text);
    });

    let sendbutton_clone = sendbutton.clone();
    entrytext.connect_key_press_event(move |e, x| {
        let key = x.keyval();
        if key == Return || key == ISO_Enter || key == KP_Enter || key == _3270_Enter {
            sendbutton_clone.emit_clicked();
            e.set_text("");
        }
        Inhibit(false)
    });

    let tx_clone_sendbutton = tx.clone();
    let actual_text_clone_sendbutton = Rc::clone(&actual_text);
    sendbutton.connect_clicked(move |_| {
        let mut actual_text = actual_text_clone_sendbutton.borrow_mut();

        if actual_text.is_empty() {
            return;
        }

        let text = actual_text.clone();

        tx_clone_sendbutton
            .send(WindowInfo::ChatMessage(
                ChatMessage::new(&id_messenger, text),
                false,
            ))
            .unwrap();

        // TODO: fix
        //entrytext.set_text(""); // -> ERROR 'already borrowed: BorrowMutError'
        *actual_text = String::new();
    });

    let statuslabel_clone = statuslabel.clone();
    let messenger_clone_addclients = Rc::clone(&messenger);
    addbutton.connect_clicked(move |_| {
        let builder = gtk::Builder::from_file("src/windows/ismalltalk-addclient.ui");
        let new_window: gtk::Window = builder
            .object("addclient-window")
            .expect("Couldn't set add client window");

        let addclient_entry: gtk::Entry = builder
            .object("addclient-entry")
            .expect("Couldn't set addclient-entry");

        let addclient_button: gtk::Button = builder
            .object("addclient-button")
            .expect("Couldn't set addclient-button");

        let addclient_label: gtk::Label = builder
            .object("addclient-label")
            .expect("Couldn't set addclient-label");

        window.set_accept_focus(false);
        new_window.show_all();

        let statuslabel_clone_button = statuslabel_clone.clone();
        let messenger_clone_addclients_button = Rc::clone(&messenger_clone_addclients);
        addclient_button.connect_clicked(move |_| {
            let clients = &mut messenger_clone_addclients_button.borrow_mut().clients;

            let text = addclient_entry.text();
            let text = text.as_str();

            let client = client::validate_ip(text);

            if let Some(client_error) = client.as_ref().err() {
                statuslabel_clone_button.set_text(client_error.0);
            }

            println!("Client {} connected", text);

            // TODO: Add AddClients !!sending!!

            clients.push(client.unwrap());
            addclient_entry.set_text("");
            addclient_label.set_text("Username added sucessfully!");
            statuslabel_clone_button.set_text(format!("{} connected", clients.len()).as_str());

            // TODO: Add HostClients !!sending!!
        });

        let window_clone = window.clone();
        new_window.connect_delete_event(move |_, _| {
            window_clone.set_accept_focus(true);
            Inhibit(false)
        });
    });

    let server_clone = Arc::clone(&mess_server);
    let tx_server_clone = tx.clone();
    thread::spawn(move || {
        let server = server_clone.lock().expect("Couldn't get server...");

        for client in server.incoming() {
            if client.is_err() {
                let error = client.as_ref().unwrap_err().to_string();
                println!("ERROR: {}", error);
                tx_server_clone
                    .send(WindowInfo::ChatInfo(
                        ChatInfo::new(TypeChat::Error, error),
                        true,
                    ))
                    .unwrap();
            }

            // TODO: Add HostClients and AddClients !!receiving!!
            // tx_server_clone.send(WindowInfo::ChatInfo(ChatInfo::from_type_system(TypeSystem::AddClients), false)).unwrap();

            let client_clone = client;
            let tx_server_clone_thread = tx_server_clone.clone();
            thread::spawn(move || loop {
                let stream = client_clone.as_ref();

                if stream.is_err() {
                    break;
                }

                let mut stream = client_clone.as_ref().unwrap();

                let mut buffer = [0; 1024];

                stream
                    .read(&mut buffer)
                    .expect("Couldn't read from buffer...");

                let text = String::from_utf8(Vec::from(&buffer[..])).unwrap();

                let text = text.trim_end_matches(char::from(0));

                if text.is_empty() {
                    break;
                }

                tx_server_clone_thread
                    .send(WindowInfo::ChatMessage(
                        ChatMessage::from(text.to_string()),
                        true,
                    ))
                    .unwrap();
            });
        }
    });

    let loaderspinner_clone = loaderspinner.clone();
    let statuslabel_clone_rx = statuslabel.clone();
    let messenger_clone_rx = Rc::clone(&messenger);
    rx.attach(None, move |msg| {
        loaderspinner_clone.set_active(true);

        let (_, mut end) = buffer.bounds();
        let mut messenger = messenger_clone_rx.borrow_mut();

        // if let WindowInfo::ChatInfo(chat_info, _) = msg {
        //     if let TypeChat::System(type_system) = chat_info.type_chat {
        //         match type_system {
        //             TypeSystem::AddClients =>
        //         }
        //     }
        // }

        let (message, is_sent) = msg.get_chat_data();

        if !is_sent {
            let sent = messenger.send_text(&message);

            if !sent.is_empty() {
                for err in sent {
                    println!("ERROR: {}", err)
                }
            }
        }

        statuslabel_clone_rx.set_text(format!("{} connected", messenger.clients.len()).as_str());

        buffer.insert_markup(&mut end, message.as_str());

        textview.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);

        loaderspinner_clone.set_active(false);

        Continue(true)
    });

    gtk::main();
}
