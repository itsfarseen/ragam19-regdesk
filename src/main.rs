#![allow(dead_code, unused_variables)]

mod repository;
mod view;
use gtk;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

struct App {
    main_view: RefCell<view::main_view::MainView>,
    login: Option<Rc<view::login::Login>>,
    home: Option<Rc<view::home::Home>>,
}

fn main() {
    gtk::init().expect("Could not initialize GTK");

    let mut login_db = repository::impl_in_mem::Login::new();
    login_db.generate_dummy_values();

    App::new(Arc::from(login_db));

    gtk::main();
}

impl App {
    fn new(login_db: Arc<dyn repository::ILogin>) -> Rc<RefCell<Self>> {
        let this = Rc::from(RefCell::from(Self {
            main_view: RefCell::from(view::main_view::MainView::new()),
            login: None,
            home: None,
        }));
        {
            let login_cb = Box::from(clone! {this => move|message|{
                match message {
                    view::login::Message::LoginSuccess(reg_desk) => {
                        this.borrow().switch_view_home(reg_desk);
                    }
                }
            }});
            this.borrow_mut().login = Some(view::login::Login::new(login_cb, login_db));
        }
        {
            let home_cb = Box::from(clone! {this => move|message| {
                println!("Message received from Home!");
            }});
            this.borrow_mut().home = Some(view::home::Home::new(home_cb));
        }
        this.borrow().switch_view_login();

        this
    }

    fn switch_view_login(&self) {
        self.main_view
            .borrow_mut()
            .load(self.login.as_ref().unwrap().as_ref());
    }

    fn switch_view_home(&self, reg_desk: Box<dyn repository::IRegDesk>) {
        self.home.as_ref().unwrap().set_reg_desk(reg_desk);
        self.main_view
            .borrow_mut()
            .load(self.home.as_ref().unwrap().as_ref());
    }
}
