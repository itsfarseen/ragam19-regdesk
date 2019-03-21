use super::main_view::View;
use crate::repository::*;
use glib;
use gtk;
use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct HospiReg {
    ui: HospiRegUI,
    participant: Cell<Option<Participant>>,
    reg_desk: Cell<Option<Box<dyn IRegDesk>>>,
    callback: Box<dyn Fn(Message)>,
}

pub enum Message {
    Back(Participant, Box<dyn IRegDesk>),
}

ui_struct! {
    struct HospiRegUI {
        root: gtk::Widget,
        ragam_id: gtk::Label,
        name: gtk::Label,
        college: gtk::Label,
        reg_status: gtk::Label,
        hostel: gtk::Entry,
        room: gtk::Entry,
        saved_successfully: gtk::Label,
        back: gtk::Button,
        save: gtk::Button
    }
}

impl HospiReg {
    pub fn new(callback: Box<dyn Fn(Message)>) -> Rc<Self> {
        let glade_src = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/ui/hospi_reg.glade"
        ));
        let builder = gtk::Builder::new_from_string(glade_src);

        let ret = Rc::from(HospiReg {
            ui: HospiRegUI::build(builder),
            callback,
            participant: Cell::from(None),
            reg_desk: Cell::from(None),
        });

        Self::initialize_callbacks(ret.clone());

        ret
    }

    pub fn set_participant_and_reg_desk(
        &self,
        participant: Participant,
        reg_desk: Box<dyn IRegDesk>,
    ) {
        self.state_default();
        self.load_participant(&participant);
        self.participant.replace(Some(participant));
        self.reg_desk.replace(Some(reg_desk));
    }

    fn initialize_callbacks(this: Rc<Self>) {
        let this_weak = Rc::downgrade(&this);

        this.ui
            .save
            .connect_clicked(clone! {this_weak => move |_|{
                let this = this_weak.upgrade().unwrap();
                this.state_saving();

                let mut reg_desk = this.reg_desk.take().unwrap();
                let participant = this.participant.take().unwrap();

                let hostel = this.ui.hostel.get_text().unwrap().as_str().to_owned();
                let room = this.ui.room.get_text().unwrap().as_str().to_owned();


                let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                std::thread::spawn(move || {
                    tx.send((reg_desk.participant_update_hospi(participant, &hostel, &room), reg_desk))
                });

                let this_weak = this_weak.clone();
                rx.attach(None, move |(participant, reg_desk)| {
                    let this = this_weak.upgrade().unwrap();
                    this.load_participant(&participant);
                    this.participant.set(Some(participant));
                    this.reg_desk.set(Some(reg_desk));
                    this.state_saved();
                    glib::source::Continue(false)
                });
            }});

        this.ui.back.connect_clicked(clone!{this_weak => move |_| {
            let this = this_weak.upgrade().unwrap();
            (this.callback)(Message::Back(this.participant.take().unwrap(), this.reg_desk.take().unwrap()));
        }});
    }

    fn load_participant(&self, participant: &Participant) {
        let id = format!("R19{:06}", participant.id());
        self.ui.ragam_id.set_text(&id);
        self.ui.name.set_text(&participant.info.name);
        self.ui.college.set_text(&participant.college.name);
        match participant.hospitality {
            Some(ref hospi_regd) => {
                self.ui
                    .reg_status
                    .set_text(&format!("Registered by {}", hospi_regd.admin.name));
                self.ui
                    .hostel
                    .set_text(&hospi_regd.hostel);
                self.ui
                    .room
                    .set_text(&hospi_regd.room);
            },
            None => {
                self.ui
                    .reg_status
                    .set_text("N/A");
                self.ui
                    .hostel
                    .set_text("");
                self.ui
                    .room
                    .set_text("");
            }
        }
    }

    fn state_default(&self) {
        self.ui.saved_successfully.set_opacity(0.0);
        self.ui.save.set_label("Save");
        set_sensitive!(true, self.ui{
            back,
            save,
            hostel,
            room
        });
    }

    fn state_saving(&self) {
        self.ui.save.set_label("Saving ..");
        set_sensitive!(false, self.ui{
            back,
            save,
            hostel,
            room
        });
    }

    fn state_saved(&self) {
        self.ui.save.set_label("Save");
        set_sensitive!(true, self.ui{
            back,
            save,
            hostel,
            room
        });
        self.ui.saved_successfully.set_opacity(1.0);
        let saved_successfully = self.ui.saved_successfully.clone();
        glib::timeout_add_local(5000, move || {
            saved_successfully.set_opacity(0.0);
            glib::source::Continue(false)
        });
    }
}

impl View for HospiReg {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.ui.root
    }
}
