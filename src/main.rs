#![allow(dead_code, unused_variables)]

#[macro_use]
extern crate mysql;

mod repository;
mod view;

use repository::*;

use dotenv::dotenv;
use gtk;
use mysql::OptsBuilder;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

struct App {
    main_view: RefCell<view::main_view::MainView>,
    login: Option<Rc<view::login::Login>>,
    home_reg_desk: Option<Rc<view::home::Home>>,
    home_hospi: Option<Rc<view::home_hospi::HomeHospi>>,
    verify_reg: Option<Rc<view::verify_reg::VerifyReg>>,
    hospi_reg: Option<Rc<view::hospi_reg::HospiReg>>,
    create_update: Option<Rc<view::create_update_participant::CreateUpdateParticipant>>,
}

fn main() {
    dotenv().ok();
    gtk::init().expect("Could not initialize GTK");

    let mut builder = OptsBuilder::new();
    builder
        .ip_or_hostname(Some(
            std::env::var("MYSQL_HOST").expect("Please set MYSQL_HOST env var"),
        ))
        .db_name(Some(
            std::env::var("MYSQL_DB").expect("Please set MYSQL_DB env var"),
        ))
        .tcp_port(
            std::env::var("MYSQL_PORT")
                .expect("Please set MYSQL_PORT env var")
                .parse()
                .expect("Invalid MYSQL_PORT"),
        )
        .user(Some(
            std::env::var("MYSQL_USER").expect("Please set MYSQL_USER env var"),
        ))
        .pass(Some(
            std::env::var("MYSQL_PASS").expect("Please set MYSQL_PASS env var"),
        ));

    let mysql_conn = mysql::Conn::new(builder).expect("Failed to connect to MySql");

    let mut login_db = repository::impl_mysql::Login::new(mysql_conn);

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
            hospi_reg: None,
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
                    view::home::Message::RagamReg(reg_desk) => {
                        this.borrow().switch_view_new_ragam_reg(reg_desk);
                    },
                    view::home::Message::KaloReg(reg_desk) => {
                        this.borrow().switch_view_new_kalo_reg(reg_desk);
                    },
                    view::home::Message::VerifyReg(participant, reg_desk) => {
                        this.borrow().switch_view_verify_reg(participant, reg_desk);
                    },
                    view::home::Message::Logout(_) => {
                        this.borrow().switch_view_login();
                    }
                }
            }});
            this.borrow_mut().home_reg_desk = Some(view::home::Home::new(home_reg_desk_cb));
        }
        {
            let home_hospi_cb = Box::from(clone! {this => move|message| {
                match message {
                    view::home_hospi::Message::RegHospi(participant, reg_desk) => {
                        this.borrow().switch_view_hospi_reg(participant, reg_desk);
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
            let hospi_reg_cb = Box::from(clone! {this => move|message| {
                match message {
                    view::hospi_reg::Message::Back(_participant, reg_desk) => {
                        this.borrow().switch_view_home_hospi(reg_desk);
                    }
                }
            }});
            this.borrow_mut().hospi_reg = Some(view::hospi_reg::HospiReg::new(hospi_reg_cb));
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

    fn switch_view_hospi_reg(&self, participant: Participant, reg_desk: Box<dyn IRegDesk>) {
        self.hospi_reg
            .as_ref()
            .unwrap()
            .set_participant_and_reg_desk(participant, reg_desk);
        self.main_view
            .borrow_mut()
            .load(self.hospi_reg.as_ref().unwrap().as_ref());
    }

    fn switch_view_new_ragam_reg(&self, reg_desk: Box<dyn IRegDesk>) {
        self.create_update
            .as_ref()
            .unwrap()
            .set_mode_create_ragam(reg_desk);
        self.main_view
            .borrow_mut()
            .load(self.create_update.as_ref().unwrap().as_ref());
    }

    fn switch_view_new_kalo_reg(&self, reg_desk: Box<dyn IRegDesk>) {
        self.create_update
            .as_ref()
            .unwrap()
            .set_mode_create_kalotsavam(reg_desk);
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
