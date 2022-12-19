pub mod chat;
pub mod message;

use std::cell::RefCell;
use std::rc::Rc;
use gtk::{prelude::*};
use gtk::{Application, ApplicationWindow, Button, Label, Box, Entry, Align};
use crate::chat::Chat;

fn main() {
    let application = Application::builder()
        .application_id("com.ISmallTalk")
        .build();

    application.connect_activate(|app| application_run(app));

    application.run();
}

fn application_run(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("iSmallTalk")
        .default_width(420)
        .default_height(580)
        .window_position(gtk::WindowPosition::Center)
        .build();

    let texts = Rc::new(RefCell::new(Chat::new()));
    let actual_text = Rc::new(RefCell::new(String::from("")));

    let box1 = Box::new(gtk::Orientation::Vertical, 8);
    let box2 = Box::new(gtk::Orientation::Horizontal, 8);

    let text_clone_entry = Rc::clone(&actual_text);
    let entry = Entry::new();
    entry.set_text(&*text_clone_entry.borrow());
    entry.set_placeholder_text(Some("Type your message:"));

    let title_label = Label::new(None);
    title_label.set_markup("<b>Messages:</b>");

    let text_clone_label = Rc::clone(&actual_text);
    let texts_clone_label = Rc::clone(&texts);
    let label = Label::with_mnemonic(&*text_clone_label.borrow());
    label.set_line_wrap(true);
    label.set_text(texts_clone_label.borrow().show().as_str());
    label.set_halign(Align::Start);
    label.set_margin(8);

    let button = Button::with_label("Send");

    box1.pack_start(&title_label, false, false, 8);
    box1.pack_start(&label, false, false, 8);
    box1.pack_end(&box2, false, true, 8);
    box2.pack_start(&entry, true, true, 8);
    box2.pack_end(&button, true, true, 8);
    window.add(&box1);

    let text_clone_entry = Rc::clone(&actual_text);
    entry.connect_changed(move |x| {
        let mut t = text_clone_entry.borrow_mut();
        let new_text = x.text();
        let new_text = new_text.as_str();
        *t = String::from(new_text);
    });

    let text_clone_button = Rc::clone(&actual_text);
    let texts_clone_button = Rc::clone(&texts);
    button.connect_clicked(move |_| {
        let t = text_clone_button.borrow();
        println!("input: {}", *t);
        let text = t.to_owned();
        if text.is_empty() {
            return
        }
        let mut texts = texts_clone_button.borrow_mut();
        texts.add_message(text);
        label.set_text(texts.show().as_str());
    });

    window.show_all();
}