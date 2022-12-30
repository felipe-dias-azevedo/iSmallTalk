use crate::app::main_window::MainWindow;
use crate::channel::chat_info::{ChatInfo, TypeChat};
use crate::channel::system_action::SystemAction;
use crate::messaging::host_messenger::HostMessenger;
use crate::messaging::messenger::Messenger;
use crate::messaging::sending::send_message;
use gtk::glib::{clone, Receiver};
use gtk::prelude::*;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

pub fn start(
    receiver: Receiver<SystemAction>,
    main_window: &MainWindow,
    default_messenger: &Rc<RefCell<Messenger>>,
    default_host_messenger: &Rc<RefCell<Option<HostMessenger>>>,
) {
    let buffer = main_window.textview.buffer().unwrap();

    receiver.attach(None, clone!(
        @strong main_window.loaderspinner as loaderspinner,
        @strong main_window.statuslabel as statuslabel,
        @strong main_window.sendbutton as sendbutton,
        @strong main_window.entrytext as entrytext,
        @strong main_window.textview as textview,
        @strong buffer,
        @strong default_messenger,
        @strong default_host_messenger
        => move |system_action| {
            loaderspinner.set_active(true);

            match system_action {
                SystemAction::SendChatInfo(chat_info, is_sent) => {
                    println!("signal SendChatInfo");

                    send_message(
                        true,
                        &chat_info.to_send_text(),
                        &chat_info.to_chat_text(),
                        &chat_info.id,
                        is_sent,
                        &default_host_messenger,
                        &default_messenger,
                        &buffer,
                        &statuslabel,
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
                        &default_host_messenger,
                        &default_messenger,
                        &buffer,
                        &statuslabel,
                        &textview,
                        &sendbutton,
                        &entrytext,
                    );
                }
                SystemAction::ToggleHost(is_host) => {
                    println!("signal ToggleHost");

                    let messenger = default_messenger.borrow();
                    let mut host_messenger = default_host_messenger.borrow_mut();

                    if is_host {
                        *host_messenger = Some(HostMessenger::from(&*messenger));
                        statuslabel.set_text("0 connected");
                    } else {
                        *host_messenger = None;
                        statuslabel.set_text("Connected to host");
                    }
                }
                SystemAction::AddClient(ip, port, client) => {
                    println!("signal AddClient");

                    let mut host_messenger = default_host_messenger.borrow_mut();

                    if host_messenger.is_some() {
                        let host_messenger = host_messenger.as_mut().unwrap();

                        host_messenger.add_connection(ip, port, client);

                        statuslabel.set_text(
                            format!("{} connected", host_messenger.get_ammount_connected()).as_str(),
                        );
                    } else {
                        let mut messenger = default_messenger.borrow_mut();

                        messenger.client = Some(client);

                        statuslabel.set_text("Connected to host");
                    }

                    sendbutton.set_sensitive(true);
                    entrytext.set_sensitive(true);
                }
                SystemAction::RequestAddClient(ip, port, mut client) => {
                    println!("signal RequestAddClient");

                    let (_, mut end) = buffer.bounds();

                    let mut host_messenger = default_host_messenger.borrow_mut();
                    let mut messenger = default_messenger.borrow_mut();

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
                                    let chat_info = ChatInfo::new(
                                        messenger.get_id(),
                                        TypeChat::Info,
                                        format!("User {} connected the chat", host_client.get_id()),
                                    );

                                    let error = client.write(chat_info.to_send_text().as_bytes()).err();

                                    if let Some(err) = error {
                                        // TODO: show in dialog that error happened
                                        println!("ERROR: {}", err);
                                    }
                                }
                            }

                            host_messenger.add_connection(ip, port, client);

                            statuslabel.set_text(
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

                            statuslabel.set_text("Connected to host");
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

                    let mut host_messenger = default_host_messenger.borrow_mut();

                    if host_messenger.is_some() {
                        let host_messenger = host_messenger.as_mut().unwrap();

                        let message = format!("CEC {}", host_messenger.get_id());

                        let errors = host_messenger.send_message(&message);

                        for err in errors {
                            // TODO: show in dialog that error happened
                            println!("ERROR: {}", err)
                        }
                    } else {
                        let mut messenger = default_messenger.borrow_mut();

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

                    let mut host_messenger = default_host_messenger.borrow_mut();

                    if host_messenger.is_some() {
                        let host_messenger = host_messenger.as_mut().unwrap();

                        host_messenger.remove_connection(ip_port);

                        statuslabel.set_text(
                            format!("{} connected", host_messenger.get_ammount_connected()).as_str(),
                        );

                        if host_messenger.get_ammount_connected() == 0 {
                            sendbutton.set_sensitive(false);
                            entrytext.set_sensitive(false);
                        }
                    } else {
                        let mut messenger = default_messenger.borrow_mut();

                        messenger.remove_connection();

                        sendbutton.set_sensitive(false);
                        entrytext.set_sensitive(false);

                        statuslabel.set_text("Not connected");
                    }
                }
            }

            loaderspinner.set_active(false);

            Continue(true)
        }
    ));
}
