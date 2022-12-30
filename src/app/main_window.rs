use crate::app::about_window::AboutWindow;
use crate::app::add_client_window::AddClientWindow;
use crate::channel::chat_message::ChatMessage;
use crate::channel::system_action::SystemAction;
use gtk::gdk::keys::constants::{ISO_Enter, KP_Enter, Return, _3270_Enter};
use gtk::glib::{clone, Sender};
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MainWindow {
    pub window: gtk::Window,
    hostcheck: gtk::CheckButton,
    pub textview: gtk::TextView,
    pub sendbutton: gtk::Button,
    pub entrytext: gtk::Entry,
    leavebutton: gtk::Button,
    pub statuslabel: gtk::Label,
    idlabel: gtk::Label,
    pub loaderspinner: gtk::Spinner,
    addbutton: gtk::Button,
    connectbutton: gtk::Button,
    menupopover: gtk::Popover,
    aboutbutton: gtk::Button,
}

impl MainWindow {
    pub fn new(builder: &gtk::Builder, id: &String) -> Self {
        let main_window = MainWindow {
            window: builder.object("main-window").expect("Couldn't set window"),
            hostcheck: builder
                .object("main-hostcheck")
                .expect("Couldn't get main-hostcheck"),
            textview: builder
                .object("main-textview")
                .expect("Couldn't get main-textview"),
            sendbutton: builder
                .object("main-sendbutton")
                .expect("Couldn't get main-sendbutton"),
            entrytext: builder
                .object("main-entrytext")
                .expect("Couldn't get main-entrytext"),
            leavebutton: builder
                .object("main-leavebutton")
                .expect("Couldn't get main-leavebutton"),
            statuslabel: builder
                .object("main-statuslabel")
                .expect("Couldn't get main-statuslabel"),
            idlabel: builder.object("main-id").expect("Couldn't get main-id"),
            loaderspinner: builder
                .object("main-loader")
                .expect("Couldn't get main-loader"),
            addbutton: builder
                .object("main-addbutton")
                .expect("Couldn't get main-addbutton"),
            connectbutton: builder
                .object("main-connectbutton")
                .expect("Couldn't get main-connectbutton"),
            menupopover: builder
                .object("main-menupopover")
                .expect("Couldn't get main-menupopover"),
            aboutbutton: builder
                .object("main-aboutbutton")
                .expect("Couldn't get main-aboutbutton"),
        };

        main_window.window.show_all();

        main_window.reset_components(id.to_owned());

        main_window
    }

    fn reset_components(&self, id: String) {
        self.sendbutton.set_sensitive(false);

        self.entrytext.set_sensitive(false);

        self.leavebutton.hide();

        self.idlabel.set_text(id.as_str());

        self.connectbutton.hide();
        self.connectbutton.set_sensitive(false);

        // TODO: read from user config file (get last used)
        if !self.hostcheck.is_active() {
            self.addbutton.hide();
        }
        // else {
        //     self.connectbutton.hide();
        // }
    }

    pub fn on_delete(&self, sender: &Sender<SystemAction>) {
        self.window
            .connect_delete_event(clone!(@strong sender => move |_, _| {
                sender
                    .send(SystemAction::LeaveChatAndQuit)
                    .unwrap();

                Inhibit(false)
            }));
    }

    pub fn on_about_clicked(&self, builder: &gtk::Builder) {
        self.aboutbutton.connect_clicked(clone!(
            @strong self.menupopover as menupopover,
            @strong builder
            => move |_| {
                let _ = AboutWindow::new(&builder);

                menupopover.hide();
        }));
    }

    pub fn on_host_change(&self, sender: &Sender<SystemAction>) {
        self.hostcheck.connect_clicked(clone!(
            @strong self.addbutton as addbutton,
            @strong self.connectbutton as connectbutton,
            @strong sender
            => move |is_host| {
                if is_host.is_active() {
                    addbutton.show();
                    connectbutton.hide();

                    let toggled_host = sender
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
        }));
    }

    pub fn on_text_change(&self, actual_text: &Rc<RefCell<String>>) {
        self.entrytext
            .connect_changed(clone!(@strong actual_text => move |e| {
                let mut actual_text = actual_text.borrow_mut();
                let new_text = e.text();
                let new_text = new_text.as_str();
                *actual_text = String::from(new_text);
                e.set_text(new_text);
            }));
    }

    pub fn on_entry_keyboard_press(&self) {
        self.entrytext.connect_key_press_event(clone!(
            @strong self.sendbutton as sendbutton
            => move |_, x| {
                let key = x.keyval();
                if key == Return || key == ISO_Enter || key == KP_Enter || key == _3270_Enter {
                    sendbutton.emit_clicked();
                }
                Inhibit(false)
        }));
    }

    pub fn on_send_clicked(
        &self,
        id: &String,
        actual_text: &Rc<RefCell<String>>,
        sender: &Sender<SystemAction>,
    ) {
        self.sendbutton.connect_clicked(clone!(
            @strong id,
            @strong actual_text,
            @strong sender,
            => move |_| {
                let mut actual_text = actual_text.borrow_mut();

                if actual_text.is_empty() {
                    return;
                }

                let text = actual_text.clone();

                sender
                    .send(SystemAction::SendChatMessage(
                        ChatMessage::new(&id, text),
                        true,
                    ))
                    .unwrap();

                sender
                    .send(SystemAction::ResetMainTextEntry)
                    .unwrap();

                *actual_text = String::new();
        }));
    }

    pub fn on_add_client_clicked(&self, sender: &Sender<SystemAction>) {
        self.addbutton.connect_clicked(clone!(
        @strong self.window as window,
        @strong sender
        => move |_| {
            let _ = AddClientWindow::new(&window, &sender);

            window.set_accept_focus(false);
        }));
    }
}
