#![allow(dead_code, unused_variables)]

mod repository;
mod view;

use repository::*;

use gtk;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

struct App {
    main_view: RefCell<view::main_view::MainView>,
    login: Option<Rc<view::login::Login>>,
    home_reg_desk: Option<Rc<view::home::Home>>,
    home_hospi: Option<Rc<view::home_hospi::HomeHospi>>,
    verify_reg: Option<Rc<view::verify_reg::VerifyReg>>,
    create_update: Option<Rc<view::create_update_participant::CreateUpdateParticipant>>,
}

fn main() {
    gtk::init().expect("Could not initialize GTK");

    let mut login_db = impl_in_mem::Login::new();
    login_db.generate_dummy_values();

    App::new(Arc::from(login_db));

    gtk::main();
}

impl App {
    fn new(login_db: Arc<dyn ILogin>) -> Rc<RefCell<Self>> {
        let this = Rc::from(RefCell::from(Self {
            main_view: RefCell::from(view::main_view::MainView::new()),
            login: None,
            home_reg_desk: None,
            home_hospi: None,
            verify_reg: None,
            create_update: None,
        }));
        {
            let login_cb = Box::from(clone! {this => move|message|{
                match message {
                    view::login::Message::LoginSuccessRegDesk(reg_desk) => {
                        this.borrow().switch_view_home_reg_desk(reg_desk);
                    },
                    view::login::Message::LoginSuccessHospi(reg_desk) => {
                        this.borrow().switch_view_home_hospi(reg_desk);
                    }
                }
            }});
            this.borrow_mut().login = Some(view::login::Login::new(login_cb, login_db));
        }
        {
            let home_reg_desk_cb = Box::from(clone! {this => move|message| {
                match message {
                    view::home::Message::NewReg(reg_desk) => {
                        this.borrow().switch_view_new_participant(reg_desk);
                    },
                    view::home::Message::VerifyReg(participant, reg_desk) => {
                        this.borrow().switch_view_verify_reg(participant, reg_desk);
                    }
                }
            }});
            this.borrow_mut().home_reg_desk = Some(view::home::Home::new(home_reg_desk_cb));
        }
        {
            let home_hospi_cb = Box::from(clone! {this => move|message| {
                match message {
                    view::home_hospi::Message::RegHospi(participant, reg_desk) => {
                        println!("Reg Hospi")
                    }
                }
            }});
            this.borrow_mut().home_hospi = Some(view::home_hospi::HomeHospi::new(home_hospi_cb));
        }
        {
            let verify_reg_cb = Box::from(clone! {this => move|message| {
                match message {
                    view::verify_reg::Message::Back(_participant, reg_desk) => {
                        this.borrow().switch_view_home_reg_desk(reg_desk);
                    },
                    view::verify_reg::Message::UpdateDetails(particpant, reg_desk) => {
                        this.borrow().switch_view_update_participant(particpant, reg_desk);
                    },
                    _ => {
                        println!("Message received from Verify Reg!");
                    }
                }
            }});
            this.borrow_mut().verify_reg = Some(view::verify_reg::VerifyReg::new(verify_reg_cb));
        }
        {
            let create_update_cb = Box::from(clone! {this => move|message| {
                use view::create_update_participant::Message;
                match message {
                    Message::Back(_participant, reg_desk) => {
                        this.borrow().switch_view_home_reg_desk(reg_desk);
                    }
                }
            }});

            this.borrow_mut().create_update = Some(
                view::create_update_participant::CreateUpdateParticipant::new(create_update_cb),
            );
        }

        this.borrow().switch_view_login();

        this
    }

    fn switch_view_login(&self) {
        self.main_view
            .borrow_mut()
            .load(self.login.as_ref().unwrap().as_ref());
    }

    fn switch_view_home_reg_desk(&self, reg_desk: Box<dyn IRegDesk>) {
        self.home_reg_desk.as_ref().unwrap().set_reg_desk(reg_desk);
        self.main_view
            .borrow_mut()
            .load(self.home_reg_desk.as_ref().unwrap().as_ref());
    }

    fn switch_view_home_hospi(&self, reg_desk: Box<dyn IRegDesk>) {
        self.home_hospi.as_ref().unwrap().set_reg_desk(reg_desk);
        self.main_view
            .borrow_mut()
            .load(self.home_hospi.as_ref().unwrap().as_ref());
    }

    fn switch_view_verify_reg(&self, participant: Participant, reg_desk: Box<dyn IRegDesk>) {
        self.verify_reg
            .as_ref()
            .unwrap()
            .set_participant_and_reg_desk(participant, reg_desk);
        self.main_view
            .borrow_mut()
            .load(self.verify_reg.as_ref().unwrap().as_ref());
    }

    fn switch_view_new_participant(&self, reg_desk: Box<dyn IRegDesk>) {
        self.create_update
            .as_ref()
            .unwrap()
            .set_mode_create(reg_desk);
        self.main_view
            .borrow_mut()
            .load(self.create_update.as_ref().unwrap().as_ref());
    }

    fn switch_view_update_participant(
        &self,
        participant: Participant,
        reg_desk: Box<dyn IRegDesk>,
    ) {
        self.create_update
            .as_ref()
            .unwrap()
            .set_mode_update(participant, reg_desk);
        self.main_view
            .borrow_mut()
            .load(self.create_update.as_ref().unwrap().as_ref());
    }
}
