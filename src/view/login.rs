use super::*;
use crate::repository::*;
use gtk;
use gtk::prelude::*;
use std::sync::Arc;
use std::thread;

pub struct Login {
    ui: LoginUI,
    login_success_cb: Arc<dyn Fn(Arc<dyn IRegDesk>)>,
    login_db: Arc<dyn ILogin + Send + Sync>,
}

ui_struct! {
    struct LoginUI {
        root: gtk::Widget,
        progress_bar: gtk::ProgressBar,
        username: gtk::Entry,
        password: gtk::Entry,
        hospitality: gtk::RadioButton,
        reg_desk: gtk::RadioButton,
        login_btn: gtk::Button
    }
}

impl Login {
    pub fn new(
        login_success_cb: Arc<Fn(Arc<dyn IRegDesk>)>,
        login_db: Arc<dyn ILogin + Send + Sync>,
    ) -> Arc<Login> {
        let glade_src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ui/login.glade"));
        let builder = gtk::Builder::new_from_string(glade_src);
        let ui = LoginUI::build(builder);
        let ret = Login {
            ui,
            login_success_cb,
            login_db,
        };
        let ret = Arc::from(ret);
        Self::init_cb(ret.clone());
        ret
    }

    fn init_cb(self_: Arc<Self>) {
        self_.clone().ui.login_btn.connect_clicked(move |_| {
            let username = self_.ui.username.get_text().unwrap();
            let password = self_.ui.password.get_text().unwrap();
            if username.as_str().is_empty() || password.as_str().is_empty() {
                return;
            }

            self_.state_logging_in();

            let (rx, tx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            {
                let username = String::from(username.as_str());
                let password = String::from(password.as_str());
                let login_db = self_.login_db.clone();
                thread::spawn(move || rx.send(login_db.login_reg_desk(&username, &password)));
            }

            {
                let self_ = self_.clone();
                tx.attach(
                    None,
                    move |reg_desk: Result<Arc<dyn IRegDesk + Send + Sync>, ()>| {
                        if let Ok(reg_desk) = reg_desk {
                            (self_.login_success_cb)(reg_desk);
                        }
                        self_.state_default();
                        glib::source::Continue(false)
                    },
                );
            }
        });
    }

    fn state_logging_in(&self) {
        let progress_bar = self.ui.progress_bar.clone();
        glib::timeout_add_local(100, move || {
            progress_bar.pulse();
            if progress_bar.get_opacity() as u8 == 1 {
                glib::source::Continue(true)
            } else {
                glib::source::Continue(false)
            }
        });
        self.ui.progress_bar.set_fraction(0.0);
        self.ui.progress_bar.set_opacity(1.0);

        self.ui.username.set_sensitive(false);
        self.ui.password.set_sensitive(false);
        self.ui.reg_desk.set_sensitive(false);
        self.ui.hospitality.set_sensitive(false);
        self.ui.login_btn.set_sensitive(false);
    }

    fn state_default(&self) {
        self.ui.progress_bar.set_opacity(0.0);
        self.ui.username.set_sensitive(true);
        self.ui.password.set_sensitive(true);
        self.ui.reg_desk.set_sensitive(true);
        self.ui.hospitality.set_sensitive(true);
        self.ui.login_btn.set_sensitive(true);
    }
}

impl View for Login {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.ui.root
    }
}
