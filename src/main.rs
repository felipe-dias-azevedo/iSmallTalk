mod channel;
mod messaging;
mod networking;

use std::io::{Read, Write};
use std::thread;
use std::{cell::RefCell, rc::Rc};

use gtk::{
    gdk::keys::constants::{ISO_Enter, KP_Enter, Return, _3270_Enter},
    gio::ApplicationFlags,
    glib,
    prelude::*,
};

use crate::channel::system_action::SystemAction;
use crate::messaging::host_messenger::HostMessenger;
use crate::messaging::sending::send_message;
use crate::networking::{client, local_ip, server};
use channel::chat_info::{ChatInfo, TypeChat};
use channel::chat_message::ChatMessage;
use messaging::messenger::Messenger;

fn main() {
    let (tx_sys, rx_sys) = glib::MainContext::channel::<SystemAction>(glib::PRIORITY_DEFAULT);

    let ip = local_ip::get();
    let (server, port) = server::bind_ip_port(&ip);
    let default_messenger = Messenger::new(ip, port);
    let default_host_messenger: Option<HostMessenger> = None;
    let default_host_messenger = Rc::new(RefCell::new(default_host_messenger));

    let actual_text = Rc::new(RefCell::new(String::from("")));

    gtk::init().expect("GTK failed");

    let app = gtk::Application::new(
        Some("com.felipe.iSmallTalk"),
        ApplicationFlags::HANDLES_OPEN,
    );

    let builder = gtk::Builder::from_file("src/windows/ismalltalk-main.glade");
    let window: gtk::Window = builder.object("main-window").expect("Couldn't set window");

    window.show_all();

    let tx_sys_clone_quit = tx_sys.clone();
    window.connect_delete_event(move |_, _| {
        tx_sys_clone_quit
            .send(SystemAction::LeaveChatAndQuit)
            .unwrap();
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
    sendbutton.set_sensitive(false);

    let entrytext: gtk::Entry = builder
        .object("main-entrytext")
        .expect("Couldn't get main-entrytext");
    entrytext.set_sensitive(false);

    let leavebutton: gtk::Button = builder
        .object("main-leavebutton")
        .expect("Couldn't get main-leavebutton");
    leavebutton.hide();

    let statuslabel: gtk::Label = builder
        .object("main-statuslabel")
        .expect("Couldn't get main-statuslabel");

    let idlabel: gtk::Label = builder
        .object("main-id")
        .expect("Couldn't get main-id");
    idlabel.set_text(default_messenger.get_id().as_str());

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
    connectbutton.set_sensitive(false);

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

    // TODO: read from user config file (get last used)
    let is_host = hostcheck.is_active();
    if !is_host {
        addbutton.hide();
    }
    // else {
    //     connectbutton.hide();
    // }

    let id_messenger = default_messenger.get_id();
    let messenger = Rc::new(RefCell::new(default_messenger));

    let addbutton_clone = addbutton.clone();
    let tx_sys_clone_hostcheck = tx_sys.clone();
    hostcheck.connect_clicked(move |is_host| {
        if is_host.is_active() {
            addbutton_clone.show();
            connectbutton.hide();

            let toggled_host = tx_sys_clone_hostcheck
                .send(SystemAction::ToggleHost(true))
                .ok();

            if toggled_host.is_some() {
                is_host.set_sensitive(false);
            }
        }
        // else {
        //     addbutton_clone.hide();
        //     connectbutton.show();
        //
        //     tx_sys_clone.send(SystemAction::ToggleHost(false)).unwrap();
        // }
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
    entrytext.connect_key_press_event(move |_, x| {
        let key = x.keyval();
        if key == Return || key == ISO_Enter || key == KP_Enter || key == _3270_Enter {
            sendbutton_clone.emit_clicked();
        }
        Inhibit(false)
    });

    let id_messenger_clone_sendbutton = id_messenger.clone();
    let tx_clone_sendbutton = tx_sys.clone();
    let actual_text_clone_sendbutton = Rc::clone(&actual_text);
    sendbutton.connect_clicked(move |_| {
        let mut actual_text = actual_text_clone_sendbutton.borrow_mut();

        if actual_text.is_empty() {
            return;
        }

        let text = actual_text.clone();

        tx_clone_sendbutton
            .send(SystemAction::SendChatMessage(
                ChatMessage::new(&id_messenger_clone_sendbutton, text),
                true,
            ))
            .unwrap();

        tx_clone_sendbutton
            .send(SystemAction::ResetMainTextEntry)
            .unwrap();

        *actual_text = String::new();
    });

    let tx_sys_clone_addclient = tx_sys.clone();
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

        let tx_sys_clone_addclient_button = tx_sys_clone_addclient.clone();
        addclient_button.connect_clicked(move |_| {
            let text = addclient_entry.text();
            let text = text.as_str();

            let client = client::validate_ip(text);

            if let Some(client_error) = client.as_ref().err() {
                addclient_label.set_text(client_error.0);
                return;
            }

            println!("Client {} connected", text);

            let (ip, port) = client::id_to_ip_port(&text.to_string());

            let requested_add_client = tx_sys_clone_addclient_button
                .send(SystemAction::RequestAddClient(ip, port, client.unwrap()))
                .ok();

            if requested_add_client.is_some() {
                addclient_entry.set_text("");
                addclient_label.set_text("Username added sucessfully!");
            }
        });

        let window_clone = window.clone();
        new_window.connect_delete_event(move |_, _| {
            window_clone.set_accept_focus(true);
            Inhibit(false)
        });
    });

    let id_messenger_clone_receiver = id_messenger.clone();
    let tx_server_clone = tx_sys.clone();
    thread::spawn(move || {
        for client in server.incoming() {
            if client.is_err() {
                let error = client.as_ref().unwrap_err().to_string();
                println!("ERROR: {}", error);
                tx_server_clone
                    .send(SystemAction::SendChatInfo(
                        ChatInfo::new(
                            String::from(&id_messenger_clone_receiver),
                            TypeChat::Error,
                            error,
                        ),
                        false,
                    ))
                    .unwrap();
            }

            let id_messenger_clone_receiver_thread = id_messenger_clone_receiver.clone();
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

                if text.starts_with("CM") {
                    let text = text.split("CM ").last().unwrap_or("").to_string();
                    tx_server_clone_thread
                        .send(SystemAction::SendChatMessage(
                            ChatMessage::from(text),
                            false,
                        ))
                        .unwrap();
                } else if text.starts_with("CI") {
                    let text = text.split("CI ").last().unwrap_or("").to_string();
                    tx_server_clone_thread
                        .send(SystemAction::SendChatInfo(
                            ChatInfo::from(String::from(&id_messenger_clone_receiver_thread), text),
                            false,
                        ))
                        .unwrap();
                } else if text.starts_with("RAC") {
                    let ip_port = text.split("RAC ").last().unwrap_or("").to_string();

                    let tcp_stream = client::connect(&ip_port);
                    let (ip, port) = client::id_to_ip_port(&ip_port);

                    if let Some(client) = tcp_stream {
                        tx_server_clone_thread
                            .send(SystemAction::AddClient(ip, port, client))
                            .unwrap();
                        // TODO: Cancel request add client if error

                        tx_server_clone_thread
                            .send(SystemAction::SendChatInfo(
                                ChatInfo::new(
                                    String::from(&ip_port),
                                    TypeChat::Info,
                                    format!("User {} connected the chat", ip_port),
                                ),
                                false,
                            ))
                            .unwrap();
                    }
                } else if text.starts_with("CEC") {
                    let ip_port = text.split("CEC ").last().unwrap_or("").to_string();

                    tx_server_clone_thread
                        .send(SystemAction::ClientExitChat(String::from(&ip_port)))
                        .unwrap();

                    tx_server_clone_thread
                        .send(SystemAction::SendChatInfo(
                            ChatInfo::new(
                                String::from(&ip_port),
                                TypeChat::Info,
                                format!("User {} left the chat", ip_port),
                            ),
                            false,
                        ))
                        .unwrap();
                }
            });
        }
    });

    let loaderspinner_clone_system = loaderspinner.clone();
    let statuslabel_clone_system = statuslabel.clone();
    let default_messenger_clone_system = Rc::clone(&messenger);
    let default_host_messenger_clone_system = Rc::clone(&default_host_messenger);
    rx_sys.attach(None, move |system_action| {
        loaderspinner_clone_system.set_active(true);

        match system_action {
            SystemAction::SendChatInfo(chat_info, is_sent) => {
                println!("signal SendChatInfo");

                send_message(
                    true,
                    &chat_info.to_send_text(),
                    &chat_info.to_chat_text(),
                    &chat_info.id,
                    is_sent,
                    &default_host_messenger_clone_system,
                    &default_messenger_clone_system,
                    &buffer,
                    &statuslabel_clone_system,
                    &textview,
                    &sendbutton,
                    &entrytext,
                );
            }
            SystemAction::SendChatMessage(chat_message, is_sent) => {
                println!("signal SendChatMessage");

                send_message(
                    false,
                    &chat_message.to_send_text(),
                    &chat_message.to_chat_text(),
                    &chat_message.id,
                    is_sent,
                    &default_host_messenger_clone_system,
                    &default_messenger_clone_system,
                    &buffer,
                    &statuslabel_clone_system,
                    &textview,
                    &sendbutton,
                    &entrytext,
                );
            }
            SystemAction::ToggleHost(is_host) => {
                println!("signal ToggleHost");

                let messenger = default_messenger_clone_system.borrow();
                let mut host_messenger = default_host_messenger_clone_system.borrow_mut();

                if is_host {
                    *host_messenger = Some(HostMessenger::from(&*messenger));
                    statuslabel_clone_system.set_text("0 connected");
                } else {
                    *host_messenger = None;
                    statuslabel_clone_system.set_text("Connected to host");
                }
            }
            SystemAction::AddClient(ip, port, client) => {
                println!("signal AddClient");

                let mut host_messenger = default_host_messenger_clone_system.borrow_mut();

                if host_messenger.is_some() {
                    let host_messenger = host_messenger.as_mut().unwrap();

                    host_messenger.add_connection(ip, port, client);

                    statuslabel_clone_system.set_text(
                        format!("{} connected", host_messenger.get_ammount_connected()).as_str(),
                    );
                } else {
                    let mut messenger = default_messenger_clone_system.borrow_mut();

                    messenger.client = Some(client);

                    statuslabel_clone_system.set_text("Connected to host");
                }

                sendbutton.set_sensitive(true);
                entrytext.set_sensitive(true);
            }
            SystemAction::RequestAddClient(ip, port, mut client) => {
                println!("signal RequestAddClient");

                let (_, mut end) = buffer.bounds();

                let mut host_messenger = default_host_messenger_clone_system.borrow_mut();
                let mut messenger = default_messenger_clone_system.borrow_mut();

                let error = client
                    .write(format!("RAC {}", messenger.get_id()).as_bytes())
                    .err();

                if let Some(err) = error {
                    // TODO: show in dialog that error happened
                    println!("ERROR: {}", err);
                } else {
                    if host_messenger.is_some() {
                        let host_messenger = host_messenger.as_mut().unwrap();

                        for host_client_option in &host_messenger.clients {
                            if let Some(host_client) = host_client_option {
                                let chat_info = ChatInfo::new(messenger.get_id(), TypeChat::Info, format!("User {} connected the chat", host_client.get_id()));

                                let error = client
                                    .write(chat_info.to_send_text().as_bytes())
                                    .err();

                                if let Some(err) = error {
                                    // TODO: show in dialog that error happened
                                    println!("ERROR: {}", err);
                                }
                            }
                        }

                        host_messenger.add_connection(ip, port, client);

                        statuslabel_clone_system.set_text(
                            format!("{} connected", host_messenger.get_ammount_connected())
                                .as_str(),
                        );

                        let ip_port = format!("{}:{}", ip, port);

                        let chat_info = ChatInfo::new(
                            String::from(&ip_port),
                            TypeChat::Info,
                            format!("User {} connected the chat", ip_port),
                        );

                        host_messenger.send_broadcast_message(&chat_info.to_send_text(), &ip_port);

                        buffer.insert_markup(&mut end, &chat_info.to_chat_text());

                        textview.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
                    } else {
                        messenger.client = Some(client);

                        statuslabel_clone_system.set_text("Connected to host");
                    }

                    sendbutton.set_sensitive(true);
                    entrytext.set_sensitive(true);
                }
            }
            SystemAction::ResetMainTextEntry => {
                entrytext.set_text("");
            }
            SystemAction::LeaveChatAndQuit => {
                println!("signal LeaveChatAndQuit");

                let mut host_messenger = default_host_messenger_clone_system.borrow_mut();

                if host_messenger.is_some() {
                    let host_messenger = host_messenger.as_mut().unwrap();

                    let message = format!("CEC {}", host_messenger.get_id());

                    let errors = host_messenger.send_message(&message);

                    for err in errors {
                        // TODO: show in dialog that error happened
                        println!("ERROR: {}", err)
                    }
                } else {
                    let mut messenger = default_messenger_clone_system.borrow_mut();

                    let message = format!("CEC {}", messenger.get_id());

                    let error = messenger.send_message(&message);

                    if let Some(err) = error {
                        // TODO: show in dialog that error happened
                        println!("ERROR: {}", err)
                    }
                }

                gtk::main_quit();
            }
            SystemAction::LeaveChat => {}
            SystemAction::ClientExitChat(ip_port) => {
                println!("signal ClientExitChat");

                let mut host_messenger = default_host_messenger_clone_system.borrow_mut();

                if host_messenger.is_some() {
                    let host_messenger = host_messenger.as_mut().unwrap();

                    host_messenger.remove_connection(ip_port);

                    statuslabel_clone_system.set_text(
                        format!("{} connected", host_messenger.get_ammount_connected()).as_str(),
                    );

                    if host_messenger.get_ammount_connected() == 0 {
                        sendbutton.set_sensitive(false);
                        entrytext.set_sensitive(false);
                    }
                } else {
                    let mut messenger = default_messenger_clone_system.borrow_mut();

                    messenger.remove_connection();

                    sendbutton.set_sensitive(false);
                    entrytext.set_sensitive(false);

                    statuslabel_clone_system.set_text("Not connected");
                }
            }
            _ => {}
        }

        loaderspinner_clone_system.set_active(false);

        Continue(true)
    });

    gtk::main();
}
