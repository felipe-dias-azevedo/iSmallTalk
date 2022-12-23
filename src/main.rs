mod channel;
mod messaging;
mod networking;

use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{cell::RefCell, rc::Rc};

use gtk::{
    gdk::keys::constants::{ISO_Enter, KP_Enter, Return, _3270_Enter},
    gio::ApplicationFlags,
    glib,
    prelude::*,
};

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

    let addbutton: gtk::Button = builder
        .object("main-addbutton")
        .expect("Couldn't get main-addbutton");

    let connectbutton: gtk::Button = builder
        .object("main-connectbutton")
        .expect("Couldn't get main-connectbutton");

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
    if is_host {
        connectbutton.hide();
    } else {
        addbutton.hide();
    }

    let (mess, mess_server) = Messenger::new(is_host);

    let id_messenger = mess.get_id();
    let messenger = Rc::new(RefCell::new(mess));

    let addbutton_clone = addbutton.clone();
    let messenger_clone = Rc::clone(&messenger);
    hostcheck.connect_clicked(move |a| {
        if a.is_active() {
            addbutton_clone.show();
            connectbutton.hide();

            messenger_clone.borrow_mut().change_type(true);
        } else {
            connectbutton.show();
            addbutton_clone.hide();

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
            .send(WindowInfo::new_chat_message(
                ChatMessage::new(&id_messenger, text),
                true,
            ))
            .unwrap();

        // TODO: fix
        //entrytext.set_text(""); // -> ERROR 'already borrowed: BorrowMutError'
        *actual_text = String::new();
    });

    addbutton.connect_clicked(move |_| {
        let builder = gtk::Builder::from_file("src/windows/template-example-1.ui");
        let new_window: gtk::Window = builder
            .object("template-window")
            .expect("Couldn't set window 2");

        window.set_accept_focus(false);
        new_window.show_all();

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
                    .send(WindowInfo::new_chat_info(
                        ChatInfo::new(TypeChat::Error, error),
                        true,
                    ))
                    .unwrap();
            }

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
                    .send(WindowInfo::new_chat_message(
                        ChatMessage::from(text.to_string()),
                        false,
                    ))
                    .unwrap();
            });
        }
    });

    let messenger_clone_rx = Rc::clone(&messenger);
    rx.attach(None, move |msg| {
        let (_, mut end) = buffer.bounds();
        let mut messenger = messenger_clone_rx.borrow_mut();

        if !msg.is_sent {
            let sent = match msg.get_chat() {
                Some(text) => messenger.send_text(text),
                None => {
                    println!("ERROR: message empty...");
                    vec![]
                }
            };

            if !sent.is_empty() {
                for err in sent {
                    println!("ERROR: {}", err)
                }
            }
        }

        if let Some(message) = msg.get_chat() {
            buffer.insert_markup(&mut end, message.as_str());

            textview.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
        }

        Continue(true)
    });

    gtk::main();
}
