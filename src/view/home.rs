use super::main_view::View;
use crate::repository::*;
use gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

pub struct Home {
    ui: HomeUI,
    reg_desk: Arc<dyn IRegDesk>,
    new_reg_cb: Rc<dyn Fn()>,
    verify_reg_cb: Rc<dyn Fn(Participant)>,
}

ui_struct! {
    struct HomeUI {
        root: gtk::Widget,
        ragam_id: gtk::Entry,
        ragam_id_not_found: gtk::Label,
        search: gtk::Button,
        new_registration: gtk::Button
    }
}

impl Home {
    pub fn new(
        reg_desk: Arc<dyn IRegDesk>,
        new_reg_cb: Rc<dyn Fn()>,
        verify_reg_cb: Rc<dyn Fn(Participant)>,
    ) -> Arc<Self> {
        let glade_src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ui/home.glade"));
        let builder = gtk::Builder::new_from_string(glade_src);

        let home = Home {
            ui: HomeUI::build(builder),
            reg_desk,
            new_reg_cb,
            verify_reg_cb,
        };

        let ret = Arc::from(home);
        Self::initialize_callbacks(ret.clone());
        ret
    }

    fn initialize_callbacks(this: Arc<Self>) {
        let this_weak = Arc::downgrade(&this);

        this.ui.search.connect_clicked(clone! {this_weak => move |_| {
            let this = this_weak.upgrade().expect("Home.ui.search: Reference to Home dropped unexpectedly.");
            let ragam_id_text = this.ui.ragam_id.get_text().unwrap();
            let mut ragam_id = ragam_id_text.as_str();

            if ragam_id.starts_with("R19") {
                ragam_id = &ragam_id[3..];
            }

            ragam_id.parse().ok().and_then(|ragam_id: i32| {
                let this = this.clone();
                Some(ragam_id)
            });

        }});
    }
}

impl View for Home {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.ui.root
    }
}
