mod messages;
pub mod messenger;
pub mod local_ip;
pub mod server;

use std::sync::{Arc, Mutex};
use std::{rc::Rc, cell::RefCell};
use std::thread;
use std::io::Read;

use gtk::{prelude::*, gdk::keys::constants::{KP_Enter, ISO_Enter, Return, _3270_Enter}, gio::ApplicationFlags, glib};

use messages::window_info::WindowInfo;
use messages::chat_info::{ChatInfo, TypeChat};
use messages::chat_message::ChatMessage;
use messenger::Messenger;

fn main() {

    let (tx, rx) = glib::MainContext::channel::<WindowInfo>(glib::PRIORITY_DEFAULT);

    let actual_text = Rc::new(RefCell::new(String::from("")));

    gtk::init()
        .expect("GTK failed");

    let app = gtk::Application::new(Some("com.felipe.iSmallTalk"), ApplicationFlags::HANDLES_OPEN);

    let builder = gtk::Builder::from_file("src/windows/ismalltalk-main-3.glade");
    let window: gtk::Window = builder.object("main-window")
        .expect("Couldn't set window");

    window.show_all();

    window.connect_delete_event(|_,_| {
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

    let hostcheck: gtk::CheckButton = builder.object("main-hostcheck")
        .expect("Couldn't get main-hostcheck");
        
    let textview: gtk::TextView = builder.object("main-textview")
        .expect("Couldn't get main-textview");

    let sendbutton: gtk::Button = builder.object("main-sendbutton")
        .expect("Couldn't get main-sendbutton");

    let entrytext: gtk::Entry = builder.object("main-entrytext")
        .expect("Couldn't get main-entrytext");

    let leavebutton: gtk::Button = builder.object("main-leavebutton")
        .expect("Couldn't get main-leavebutton");
    leavebutton.hide();

    let addbutton: gtk::Button = builder.object("main-addbutton")
        .expect("Couldn't get main-addbutton");

    let connectbutton: gtk::Button = builder.object("main-connectbutton")
        .expect("Couldn't get main-connectbutton");

    let is_host = hostcheck.is_active();
    if is_host {
        connectbutton.hide();
    } else {
        addbutton.hide();
    }

    let (mess, mess_server) = Messenger::new(is_host);
    let messenger = Rc::new(RefCell::new(mess));

    let addbutton_clone = addbutton.clone();
    let messenger_clone = Rc::clone(&messenger);
    hostcheck.connect_clicked(move |a| {
        if a.is_active() {
            addbutton_clone.show();
            connectbutton.hide();

            messenger_clone.borrow_mut().change_type(true);

            // let tx_clone = tx.clone();
            // thread::spawn(move || {
            //     let mut x = 2f64;
            //     loop {
            //         x *= 1.2;
            //         thread::sleep(Duration::from_millis(x as u64));
            //         let sent = tx_clone.send(Message::new(String::from("OK?")));
            //         match sent {
            //             Ok(_) => println!("Message sent!"),
            //             Err(err) => println!("ERROR: {}", err)
            //         }
            //     }
            // });
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

    let buffer_button = buffer.clone();
    let leavebutton_clone = leavebutton.clone();
    let actual_text_clone_sendbutton = Rc::clone(&actual_text);
    sendbutton.connect_clicked(move |_| {
        let (_, mut end) = buffer_button.bounds();

        let mut actual_text = actual_text_clone_sendbutton.borrow_mut();

        if actual_text.is_empty() {
            return;
        }

        let new_text = format!("<span color='#df0e0f'>{}</span>\n", *actual_text);
        //entrytext.set_text(""); -> ERROR 'already borrowed: BorrowMutError'
        *actual_text = String::new();

        buffer_button.insert_markup(&mut end, new_text.as_str());
        leavebutton_clone.show();
    });

    addbutton.connect_clicked(move |_| {
        let builder = gtk::Builder::from_file("src/windows/template-example-1.ui");
        let new_window: gtk::Window = builder.object("template-window")
            .expect("Couldn't set window 2");

        window.set_accept_focus(false);
        new_window.show_all();

        let window_clone = window.clone();
        new_window.connect_delete_event(move |_,_| {
            window_clone.set_accept_focus(true);
            Inhibit(false)
        });
    });

    let server_clone = Arc::clone(&mess_server);
    let tx_server_clone = tx.clone();
    thread::spawn(move || {
        let server = server_clone.lock()
            .expect("Couldn't get server...");

        for client in server.incoming() {
        
            if client.is_err() {
                let error = client.as_ref().unwrap_err().to_string();
                println!("ERROR: {}", error);
                tx_server_clone.send(WindowInfo::new_chat_info(ChatInfo::new(TypeChat::Error, error), true))
                    .unwrap();
            }

            let client_clone = client;
            let tx_server_clone_thread = tx_server_clone.clone();
            thread::spawn(move || {
                loop {
                    let mut stream = client_clone.as_ref().unwrap();

                    let mut buffer = [0; 1024];        
        
                    stream.read(&mut buffer)
                        .expect("Couldn't read from buffer...");
        
                    let text = String::from_utf8(Vec::from(&buffer[..])).unwrap();
        
                    let text = text.trim_end_matches(char::from(0));
        
                    tx_server_clone_thread.send(WindowInfo::new_chat_message(ChatMessage::new(text.to_string()), false))
                        .unwrap();   
                }
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
