use super::main_view::View;
use crate::repository::*;
use fuzzy_filter;
use gdk;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub struct CreateUpdateParticipant {
    ui: CreateUpdateParticipantUI,

    college_list: RefCell<CollegeList>,

    callback: Box<dyn Fn(Message)>,

    reg_desk: Cell<Option<Box<dyn IRegDesk>>>,

    participant: Cell<Option<Participant>>,
    mode: Cell<Option<Mode>>,
}

pub enum Message {
    Back(Option<Participant>, Box<dyn IRegDesk>),
}

enum CollegeList {
    _InitSoon,
    NotLoaded(Weak<CreateUpdateParticipant>),
    Loaded(Vec<College>),
}

ui_struct! {
    struct CreateUpdateParticipantUI {
        root: gtk::Widget,

        title: gtk::Label,

        ragam_id: gtk::Label,
        name: gtk::Entry,

        male: gtk::RadioButton,
        female: gtk::RadioButton,
        other: gtk::RadioButton,

        college: gtk::Entry,
        college_popup: gtk::Popover,
        college_search: gtk::SearchEntry,
        college_list: gtk::ListBox,
        new_college_entry: gtk::Entry,
        new_college: gtk::Button,

        email: gtk::Entry,
        phone: gtk::Entry,

        back: gtk::Button,
        save: gtk::Button,

        saved_successfully: gtk::Label
    }
}

#[derive(Copy, Clone)]
enum Mode {
    CreateRagam,
    CreateKalotsavam,
    Update,
}

impl CreateUpdateParticipant {
    pub fn new(callback: Box<dyn Fn(Message)>) -> Rc<Self> {
        let glade_src = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/ui/create_update_participant.glade"
        ));
        let builder = gtk::Builder::new_from_string(glade_src);

        let ret = Rc::from(Self {
            ui: CreateUpdateParticipantUI::build(builder),
            college_list: RefCell::from(CollegeList::_InitSoon),
            callback,
            reg_desk: Cell::from(None),
            participant: Cell::from(None),
            mode: Cell::from(None),
        });

        ret.college_list
            .replace(CollegeList::NotLoaded(Rc::downgrade(&ret)));

        Self::initialize_callbacks(ret.clone());

        ret
    }

    pub fn set_mode_create_ragam(&self, reg_desk: Box<dyn IRegDesk>) {
        self.reg_desk.set(Some(reg_desk));
        self.mode.set(Some(Mode::CreateRagam));

        self.state_default_create();
        self.load_colleges();

        self.ui.name.set_text("");
        self.ui.email.set_text("");
        self.ui.college.set_text("");
        self.ui.phone.set_text("");
        self.ui.male.set_active(true);

        self.participant.set(None);
    }

    pub fn set_mode_create_kalotsavam(&self, reg_desk: Box<dyn IRegDesk>) {
        self.state_default_create();

        self.reg_desk.set(Some(reg_desk));
        self.mode.set(Some(Mode::CreateKalotsavam));
        self.load_colleges();

        self.ui.name.set_text("");
        self.ui.email.set_text("");
        self.ui.college.set_text("");
        self.ui.phone.set_text("");
        self.ui.male.set_active(true);

        self.participant.set(None);
    }

    pub fn set_mode_update(&self, participant: Participant, reg_desk: Box<dyn IRegDesk>) {
        self.state_default_update();

        self.reg_desk.set(Some(reg_desk));
        self.mode.set(Some(Mode::Update));
        self.load_colleges();

        self.load_participant(&participant);
        self.participant.set(Some(participant));
    }

    fn load_colleges(&self) {
        if self.college_list.borrow().is_loaded() {
            return;
        }

        self.state_action_pending();

        let reg_desk = self.reg_desk.take().unwrap();

        let this_weak = match self.college_list.replace(CollegeList::_InitSoon) {
            CollegeList::NotLoaded(this) => this,
            _ => panic!(),
        };

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        std::thread::spawn(move || tx.send((reg_desk.college_get_filtered(""), reg_desk)));

        rx.attach(None, move |(colleges, reg_desk)| {
            let this = this_weak.upgrade().unwrap();
            this.reg_desk.set(Some(reg_desk));
            this.college_list.replace(CollegeList::new_loaded(colleges));

            match this.mode.get().unwrap() {
                Mode::CreateRagam | Mode::CreateKalotsavam => this.state_default_create(),
                Mode::Update => {
                    let p = this.participant.take().unwrap();
                    this.load_participant(&p);
                    this.participant.set(Some(p));
                    this.state_default_update();
                }
            }

            glib::source::Continue(false)
        });
    }

    fn initialize_callbacks(this: Rc<Self>) {
        let this_weak = Rc::downgrade(&this);

        this.ui
            .college
            .connect_property_has_focus_notify(clone! {this_weak => move |entry| {
                if entry.has_focus() {
                    let this = this_weak.upgrade().unwrap();
                    this.ui.college_popup.show();
                    this.ui.college_search.grab_focus();
                }
            }});

        this.ui.college.connect_button_press_event(
            clone! {this_weak => move |entry, event_button| {
                if entry.has_focus() {
                    let this = this_weak.upgrade().unwrap();
                    this.ui.college_search.set_text(entry.get_text().unwrap().as_str());
                    this.ui.college_popup.show();
                    glib::signal::Inhibit(true)
                } else {
                    glib::signal::Inhibit(false)
                }
            }},
        );

        this.ui.college_list.connect_key_press_event(
            clone! {this_weak => move |entry, event_key| {
                if event_key.get_keyval() == gdk::enums::key::Return {
                    let this = this_weak.upgrade().unwrap();
                    this.ui.college_popup.hide();
                    glib::signal::Inhibit(true)
                } else {
                    glib::signal::Inhibit(false)
                }
            }},
        );

        this.ui
            .college_search
            .connect_property_text_notify(clone!(this_weak => move |entry| {
                let this = this_weak.upgrade().unwrap();

                let college_g_str = entry.get_text().unwrap();
                let college_str = college_g_str.as_str();
                let college_list =
                    this
                        .college_list
                        .borrow();
                let colleges = college_list
                        .fuzzy_filter(college_str)
                        .take(100);

                this.ui.college_list.foreach(|child| this.ui.college_list.remove(child));
                for college in colleges
                {
                    let row = gtk::Label::new(Some(college.name.as_str()));
                    row.show_all();
                    this.ui.college_list.add(&row);
                }
            }));

        this.ui
            .college_search
            .connect_key_press_event(clone! {this_weak => move |w, key| {
                let this = this_weak.upgrade().unwrap();

                if key.get_keyval() == gdk::enums::key::downarrow {
                    this.ui.college_list.grab_focus();
                    glib::signal::Inhibit(true)
                } else {
                    glib::signal::Inhibit(false)
                }
            }});

        this.ui
            .college_list
            .connect_key_press_event(clone! {this_weak => move |w, key| {
                let this = this_weak.upgrade().unwrap();

                if key.get_keyval() != gdk::enums::key::Down && key.get_keyval() != gdk::enums::key::Up {
                    this.ui.college_search.grab_focus();
                    this.ui.college_search.event(key);
                    glib::signal::Inhibit(true)
                } else {
                    glib::signal::Inhibit(false)
                }
            }});

        this.ui.college_list.connect_row_selected(clone! { this_weak => move |w, row| {
            let this = this_weak.upgrade().unwrap();

            if let Some(row) = row {
                this.ui.college.set_text(row.get_child().unwrap().downcast::<gtk::Label>().unwrap().get_text().unwrap().as_str());
            }
        }});

        this.ui
            .new_college
            .connect_clicked(clone! {this_weak => move |_| {
                let this = this_weak.upgrade().unwrap();

                let mut reg_desk = this.reg_desk.take().unwrap();

                this.ui.new_college.set_sensitive(false);
                this.ui.new_college_entry.set_sensitive(false);

                let college_g_str = this.ui.new_college_entry.get_text().unwrap();
                let college = String::from(college_g_str.as_str());

                let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                std::thread::spawn(move || {
                    let college = reg_desk.college_add(college);
                    let colleges = reg_desk.college_get_filtered("");
                    tx.send((college, colleges, reg_desk))
                });

                let this_weak = this_weak.clone();
                rx.attach(None, move |(college, colleges, reg_desk)| {
                    let this = this_weak.upgrade().unwrap();
                    this.college_list.replace(CollegeList::new_loaded(colleges));
                    this.ui.college.set_text(&college.name);
                    this.ui.college_search.set_text(&college.name);
                    this.reg_desk.set(Some(reg_desk));
                    this.ui.new_college.set_sensitive(true);
                    this.ui.new_college_entry.set_sensitive(true);
                    glib::source::Continue(false)
                });
            }});

        this.ui.save.connect_clicked(clone! {this_weak => move |_|{
            let this = this_weak.upgrade().unwrap();

            let (participant_info, college) = this.new_participant_from_fields();
            if college.is_none() {
                return;
            }
            let college = college.unwrap();

            this.state_action_pending();

            match this.mode.get().unwrap() {
                Mode::CreateRagam|Mode::CreateKalotsavam => {
                    let mut reg_desk = this.reg_desk.take().unwrap();

                    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                    std::thread::spawn(move || {
                        tx.send((reg_desk.participant_new_verified(participant_info, college), reg_desk))
                    });

                    let this_weak = this_weak.clone();
                    rx.attach(None, move |(participant, reg_desk)| {
                        let this = this_weak.upgrade().unwrap();
                        this.load_participant(&participant);
                        this.state_create_complete();
                        this.reg_desk.set(Some(reg_desk));
                        this.participant.set(Some(participant));
                        glib::source::Continue(false)
                    });
                },
                Mode::Update => {
                    let mut reg_desk = this.reg_desk.take().unwrap();
                    let mut participant = this.participant.take().unwrap();
                    participant.college = college;
                    participant.info = participant_info;

                    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                    std::thread::spawn(move || {
                        reg_desk.participant_update(&participant);
                        tx.send((participant, reg_desk))
                    });

                    let this_weak = this_weak.clone();
                    rx.attach(None, move |(participant, reg_desk)| {
                        let this = this_weak.upgrade().unwrap();
                        this.load_participant(&participant);
                        this.state_update_complete();
                        this.reg_desk.set(Some(reg_desk));
                        this.participant.set(Some(participant));
                        glib::source::Continue(false)
                    });
                }
            }
        }});
        this.ui.back.connect_clicked(clone! {this_weak => move |_| {
            let this = this_weak.upgrade().unwrap();
            (this.callback)(Message::Back(this.participant.take(), this.reg_desk.take().unwrap()));
        }});
    }

    fn state_initializing(&self) {
        set_sensitive!(false, self.ui{
            college,
            save
        });
    }

    // TODO: Merge state_default_* methods
    fn state_default_create(&self) {
        match self.mode.get().unwrap() {
            Mode::CreateRagam => self.ui.title.set_text("Ragam Registration"),
            Mode::CreateKalotsavam => self.ui.title.set_text("Kalotsavam Registration"),
            _ => {}
        }
        self.ui.ragam_id.set_opacity(0.0);
        self.ui.saved_successfully.set_opacity(0.0);
        set_sensitive!(true, self.ui{
            name,
            male,
            female,
            other,
            college,
            phone,
            email,
            back,
            save
        });
    }

    fn state_default_update(&self) {
        self.ui.title.set_text("Update Details");
        self.ui.saved_successfully.set_opacity(0.0);
        self.ui.ragam_id.set_opacity(1.0);
        set_sensitive!(true, self.ui{
            name,
            male,
            female,
            other,
            college,
            phone,
            email,
            back,
            save
        });
    }

    fn state_action_pending(&self) {
        set_sensitive!(false, self.ui{
            name,
            male,
            female,
            other,
            college,
            phone,
            email,
            back,
            save
        });
    }

    fn state_update_complete(&self) {
        self.ui.saved_successfully.set_opacity(1.0);
        let saved_successfully = self.ui.saved_successfully.clone();
        glib::timeout_add_local(5000, move || {
            saved_successfully.set_opacity(0.0);
            glib::source::Continue(false)
        });
        set_sensitive!(true, self.ui{
            name,
            male,
            female,
            other,
            college,
            phone,
            email,
            back,
            save
        });
    }

    fn state_create_complete(&self) {
        self.ui.saved_successfully.set_opacity(1.0);
        self.ui.ragam_id.set_opacity(1.0);
        set_sensitive!(false, self.ui{
            name,
            male,
            female,
            other,
            college,
            phone,
            email,
            save
        });
        set_sensitive!(true, self.ui.back);
    }

    fn load_participant(&self, participant: &Participant) {
        let id = match participant.info.category {
            ParticipantCategory::Kalotsavam => format!("K19{:06}", participant.id()),
            ParticipantCategory::Ragam => format!("R19{:06}", participant.id()),
        };
        self.ui.ragam_id.set_text(&id);
        self.ui.name.set_text(&participant.info.name);
        match participant.info.gender {
            Gender::Male => &self.ui.male,
            Gender::Female => &self.ui.female,
            Gender::Other => &self.ui.other,
        }
        .set_active(true);
        self.ui.college.set_text(&participant.college.name);
        self.ui.email.set_text(&participant.info.email);
        self.ui.phone.set_text(&participant.info.phone);
    }

    fn new_participant_from_fields(&self) -> (ParticipantInfo, Option<College>) {
        let name = self.ui.name.get_text().unwrap().to_string();
        let email = self.ui.email.get_text().unwrap().to_string();
        let phone = self.ui.phone.get_text().unwrap().to_string();
        let college_list = self.college_list.borrow();
        let college = self
            .ui
            .college
            .get_text()
            .as_ref()
            .map(glib::GString::as_str)
            .and_then(|s| college_list.find(s));
        (
            ParticipantInfo {
                name,
                email,
                phone,
                gender: {
                    if self.ui.male.get_active() {
                        Gender::Male
                    } else if self.ui.female.get_active() {
                        Gender::Female
                    } else {
                        Gender::Other
                    }
                },
                category: match self.mode.get().unwrap() {
                    Mode::CreateRagam => ParticipantCategory::Ragam,
                    _ => ParticipantCategory::Kalotsavam,
                },
            },
            college,
        )
    }
}

impl CollegeList {
    pub fn new_loaded(colleges: Vec<College>) -> Self {
        CollegeList::Loaded(colleges)
    }

    pub fn is_loaded(&self) -> bool {
        match self {
            CollegeList::Loaded(_) => true,
            CollegeList::NotLoaded(_) => false,
            CollegeList::_InitSoon => panic!("CollegeList::is_loaded(): self not initialized"),
        }
    }

    pub fn fuzzy_filter<'a>(&'a self, key: &'a str) -> Box<dyn Iterator<Item = &'a College> + 'a> {
        match self {
            CollegeList::Loaded(colleges) => Box::from(colleges.iter().filter(move |c| {
                fuzzy_filter::matches(&key.to_lowercase(), &c.name.to_lowercase())
            })),
            _ => Box::from(Vec::new().into_iter()),
        }
    }

    pub fn find(&self, name: &str) -> Option<College> {
        match self {
            CollegeList::Loaded(colleges) => colleges.iter().find(|c| c.name == name).cloned(),
            _ => panic!("College List not loaded"),
        }
    }
}

impl View for CreateUpdateParticipant {
    fn get_root_widget(&self) -> &gtk::Widget {
        &self.ui.root
    }
}
