#[macro_use] mod macro_ui_struct;

mod login;
pub use login::Login;

use gtk;
use gtk::prelude::*;

pub struct App {
    window: gtk::Window,
    app_container: gtk::Container,
    last_widget: Option<gtk::Widget>,
}

impl App {
    pub fn new() -> Self {
        let glade_src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ui/app.glade"));

        let builder = gtk::Builder::new_from_string(glade_src);
        let window: gtk::Window = builder.get_object("app_window").unwrap();
        let app_container: gtk::Container = builder.get_object("app_container").unwrap();
        window.show_all();
        window.connect_destroy(|_| {
            gtk::main_quit();
        });

        App {
            window,
            app_container,
            last_widget: None,
        }
    }

    pub fn load(&mut self, view: &dyn View) {
        if let Some(child) = &self.last_widget {
            self.app_container.remove(child);
        }

        self.app_container.add(view.get_root_widget());
        self.last_widget.replace(view.get_root_widget().clone());
    }
}

pub trait View {
    fn get_root_widget(&self) -> &gtk::Widget;
}
