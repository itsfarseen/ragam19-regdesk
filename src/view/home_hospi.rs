use super::main_view::View;
use crate::repository::*;
use glib;
use gtk;
use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct HomeHospi {
    ui: HomeHospiUI,
    reg_desk: Cell<Option<Box<dyn IRegDesk>>>,
    callback: Box<dyn Fn(Message)>,
}

pub enum Message {
    RegHospi(Participant, Box<dyn IRegDesk>),
}

ui_struct! {
    struct HomeHospiUI {
        root: gtk::Widget,
        ragam_id: gtk::Entry,
        ragam_id_not_found: gtk::Label,
        search: gtk::Button
    }
}

impl HomeHospi {
    pub fn new(callback: Box<dyn Fn(Message)>) -> Rc<Self> {
        let glade_src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ui/home_hospi.glade"));
        let builder = gtk::Builder::new_from_string(glade_src);

        let home = Self {
            ui: HomeHospiUI::build(builder),
            reg_desk: Cell::from(None),
            callback,
        };

        let ret = Rc::from(home);
        Self::initialize_callbacks(ret.clone());
        ret.state_default();
        ret
    }

    pub fn set_reg_desk(&self, reg_desk: Box<dyn IRegDesk>) {
        self.reg_desk.set(Some(reg_desk));
    }

    fn initialize_callbacks(this: Rc<Self>) {
        let this_weak = Rc::downgrade(&this);

        this.ui
            .ragam_id
            .connect_activate(clone! {this_weak => move |_| {
                let this = this_weak.upgrade().expect("HomeHospi.ui.ragam_id: Reference to Home dropped unexpectedly.");
                this.ui.search.emit_clicked();
            }});

        this.ui.search.connect_clicked(clone! {this_weak => move |_| {
            let this = this_weak.upgrade().expect("HomeHospi.ui.search: Reference to Home dropped unexpectedly.");
            let ragam_id_text = this.ui.ragam_id.get_text().unwrap();
            let mut ragam_id = ragam_id_text.as_str();

            if ragam_id.starts_with("R19") {
                ragam_id = &ragam_id[3..];
            }

            let ragam_id = ragam_id.parse::<i32>();
            if ragam_id.is_err() {
                this.state_ragam_id_invalid();
                return;
            }

            this.state_searching_participant();

            let ragam_id = ragam_id.unwrap();

            let reg_desk = this.reg_desk.take().expect(concat!(line!(), "HomeHospi: reg_desk is None"));
            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            std::thread::spawn(move || {
                let participant = reg_desk.participant_get(ragam_id);
                tx.send((participant, reg_desk))
            });

            let this = this.clone();
            rx.attach(None, move |(participant, reg_desk)| {
                if let Some(participant) = participant {
                    this.state_default();
                    (this.callback)(Message::RegHospi(participant, reg_desk));
                } else {
                    this.state_ragam_id_not_found();
                    this.reg_desk.set(Some(reg_desk))
                }
                glib::source::Continue(false)
            });
        }});

    }

    fn state_searching_participant(&self) {
        self.ui
            .ragam_id_not_found
            .set_text("Searching participant..");
        self.ui.ragam_id_not_found.set_opacity(1.0);
        self.ui.ragam_id.set_sensitive(false);
        self.ui.search.set_sensitive(false);
    }

    fn state_default(&self) {
        self.ui.ragam_id_not_found.set_opacity(0.0);
        self.ui.ragam_id.set_sensitive(true);
        self.ui.search.set_sensitive(true);
    }

    fn state_ragam_id_not_found(&self) {
        self.ui.ragam_id_not_found.set_text("Ragam ID not found");
        self.ui.ragam_id_not_found.set_opacity(1.0);
        self.ui.ragam_id.set_sensitive(true);
        self.ui.search.set_sensitive(true);
    }

    fn state_ragam_id_invalid(&self) {
        self.ui.ragam_id_not_found.set_text("Ragam ID invalid");
        self.ui.ragam_id_not_found.set_opacity(1.0);
        self.ui.ragam_id.set_sensitive(true);
        self.ui.search.set_sensitive(true);
    }
}

impl View for HomeHospi {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.ui.root
    }
}
