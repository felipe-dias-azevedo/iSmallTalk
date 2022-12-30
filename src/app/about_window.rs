use gtk::prelude::*;

pub struct AboutWindow {
    window: gtk::AboutDialog,
}

impl AboutWindow {
    pub fn new(builder: &gtk::Builder) -> Self {
        let about_window = AboutWindow {
            window: builder
                .object("main-aboutdialog")
                .expect("Couldn't get main-aboutdialog"),
        };

        about_window
            .window
            .connect_delete_event(move |x, _| x.hide_on_delete());

        about_window.window.show_all();

        about_window
    }
}
