use super::main_view::View;
use crate::repository::*;
use gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

pub struct Login {
    ui: LoginUI,
    callback: Box<dyn Fn(Message)>,
    // Fixme: Arc -> Box
    login_db: Arc<dyn ILogin>,
}

pub enum Message {
    LoginSuccessRegDesk(Box<dyn IRegDesk>),
    LoginSuccessHospi(Box<dyn IRegDesk>),
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
    pub fn new(callback: Box<dyn Fn(Message)>, login_db: Arc<dyn ILogin>) -> Rc<Login> {
        let glade_src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ui/login.glade"));
        let builder = gtk::Builder::new_from_string(glade_src);
        let ui = LoginUI::build(builder);
        let ret = Login {
            ui,
            callback,
            login_db,
        };
        let ret = Rc::from(ret);
        Self::initialize_callbacks(ret.clone());
        ret.state_default();
        ret
    }

    fn initialize_callbacks(this: Rc<Self>) {
        let this_weak = Rc::downgrade(&this);

        this.ui
            .password
            .connect_activate(clone! {this_weak => move |_| {
                let this = this_weak.upgrade().unwrap();
                this.ui.login_btn.emit_clicked();
            }});

        this.ui
            .login_btn
            .connect_clicked(clone! {this_weak => move |_| {
                let this = this_weak.upgrade().unwrap();

                let username = this.ui.username.get_text().unwrap();
                let password = this.ui.password.get_text().unwrap();
                if username.as_str().is_empty() || password.as_str().is_empty() {
                    return;
                }

                this.state_logging_in();

                let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                {
                    let username = String::from(username.as_str());
                    let password = String::from(password.as_str());
                    let login_db = this.login_db.clone();
                    thread::spawn(move || tx.send(login_db.login_reg_desk(&username, &password)));
                }

                rx.attach(
                    None,
                    clone!{ this_weak => move |reg_desk: Result<Box<dyn IRegDesk>, ()>| {
                        let this = this_weak.upgrade().unwrap();
                        if let Ok(reg_desk) = reg_desk {
                            if this.ui.reg_desk.get_active() {
                                (this.callback)(Message::LoginSuccessRegDesk(reg_desk));
                            } else {
                                (this.callback)(Message::LoginSuccessHospi(reg_desk));
                            }
                        }
                        this.state_default();
                        glib::source::Continue(false)
                    }},
                );
            }});
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

        set_sensitive!(false, self.ui{
            username,
            password,
            reg_desk,
            hospitality,
            login_btn
        });
    }

    fn state_default(&self) {
        self.ui.progress_bar.set_opacity(0.0);
        set_sensitive!(true, self.ui{
            username,
            password,
            reg_desk,
            hospitality,
            login_btn
        });
    }
}

impl View for Login {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.ui.root
    }
}
