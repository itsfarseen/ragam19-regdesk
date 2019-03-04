mod repository;
mod view;
use gtk;
use repository::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    gtk::init().expect("Could not initialize GTK");

    let app = Rc::from(RefCell::from(view::App::new()));

    let app_ref = app.clone();

    let login_cb = move |reg_desk: Arc<dyn IRegDesk>| {
        // app_ref.borrow_mut().load(&view::Home::new(
        //     reg_desk,
        //     Rc::from(|| {
        //         println!("New reg!");
        //     }),
        //     Rc::from(|_| {}),
        // ));
    };

    let mut login_db = repository::impl_in_mem::Login::new();
    login_db.generate_dummy_values();

    let login = view::Login::new(Arc::new(login_cb), Arc::new(login_db));

    app.borrow_mut().load(login.as_ref());
    gtk::main();
}
