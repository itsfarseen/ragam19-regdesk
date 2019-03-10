use super::main_view::View;
use crate::repository::*;
use glib;
use gtk;
use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub struct VerifyReg {
    ui: VerifyRegUI,
    participant: Cell<Option<Participant>>,
    reg_desk: Cell<Option<Box<dyn IRegDesk>>>,
    callback: Box<dyn Fn(Message)>,
}

pub enum Message {
    Back(Participant, Box<dyn IRegDesk>),
    UpdateDetails(Participant, Box<dyn IRegDesk>),
    ResetPassword(Participant, Box<dyn IRegDesk>),
}

ui_struct! {
    struct VerifyRegUI {
        root: gtk::Widget,
        ragam_id: gtk::Label,
        name: gtk::Label,
        gender: gtk::Label,
        college: gtk::Label,
        email: gtk::Label,
        reg_status: gtk::Label,
        back: gtk::Button,
        verify_reg: gtk::Button,
        update_details: gtk::Button,
        reset_password: gtk::Button
    }
}

impl VerifyReg {
    pub fn new(callback: Box<dyn Fn(Message)>) -> Rc<Self> {
        let glade_src = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/ui/verify_reg.glade"
        ));
        let builder = gtk::Builder::new_from_string(glade_src);

        let ret = Rc::from(VerifyReg {
            ui: VerifyRegUI::build(builder),
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
        self.load_participant(&participant);
        self.participant.replace(Some(participant));
        self.reg_desk.replace(Some(reg_desk));
        self.state_default();
    }

    fn initialize_callbacks(this: Rc<Self>) {
        // verify reg
        let this_weak = Rc::downgrade(&this);

        this.ui
            .verify_reg
            .connect_clicked(clone! {this_weak => move |_|{
                let this = this_weak.upgrade().unwrap();
                this.state_verifying();

                let mut reg_desk = this.reg_desk.take().unwrap();
                let participant = this.participant.take().unwrap();
                let reg_not_verfied: ParticipantRegNotVerified = participant.reg_status.err().unwrap();

                let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                std::thread::spawn(move || {
                    tx.send((reg_desk.participant_verify_reg(reg_not_verfied), reg_desk))
                });

                let this_weak = this_weak.clone();
                rx.attach(None, move |(participant, reg_desk)| {
                    let this = this_weak.upgrade().unwrap();
                    this.load_participant(&participant);
                    this.participant.set(Some(participant));
                    this.reg_desk.set(Some(reg_desk));
                    this.state_default();
                    glib::source::Continue(false)
                });
            }});

        this.ui.back.connect_clicked(clone!{this_weak => move |_| {
            let this = this_weak.upgrade().unwrap();
            (this.callback)(Message::Back(this.participant.take().unwrap(), this.reg_desk.take().unwrap()));
        }});

        this.ui.update_details.connect_clicked(clone!{this_weak => move |_| {
            let this = this_weak.upgrade().unwrap();
            (this.callback)(Message::UpdateDetails(this.participant.take().unwrap(), this.reg_desk.take().unwrap()));
        }});

        this.ui.reset_password.connect_clicked(clone!{this_weak => move |_| {
            let this = this_weak.upgrade().unwrap();
            (this.callback)(Message::ResetPassword(this.participant.take().unwrap(), this.reg_desk.take().unwrap()));
        }});
    }

    fn load_participant(&self, participant: &Participant) {
        let id = format!("R19{:06}", participant.id());
        self.ui.ragam_id.set_text(&id);
        self.ui.name.set_text(&participant.info.name);
        self.ui.gender.set_text(&participant.info.gender);
        self.ui.college.set_text(&participant.college.name);
        self.ui.email.set_text(&participant.info.email);
        match participant.reg_status {
            Ok(ref reg_verified) => {
                self.ui
                    .reg_status
                    .set_text(&format!("Verified by {}", reg_verified.admin.name));
                self.ui.verify_reg.set_label("Verified");
                self.ui.verify_reg.set_sensitive(false);
            }
            Err(ref _reg_not_verified) => {
                self.ui.reg_status.set_text("Unverifed");
                self.ui.verify_reg.set_label("Verify");
                self.ui.verify_reg.set_sensitive(true);
            }
        }
    }

    fn state_default(&self) {
        self.ui.verify_reg.set_label("Verify Registration");
        set_sensitive!(true, self.ui{
            back,
            verify_reg,
            update_details,
            reset_password
        });
    }

    fn state_verifying(&self) {
        self.ui.verify_reg.set_label("Verifying..");
        set_sensitive!(false, self.ui{
            back,
            verify_reg,
            update_details,
            reset_password
        });
    }
}

impl View for VerifyReg {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.ui.root
    }
}
