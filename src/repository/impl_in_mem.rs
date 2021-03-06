use super::*;
use std::collections::HashMap;
use std::{thread, time};

static DELAY: u64 = 100;

type Username = String;
struct AdminEx {
    id: i32,
    username: String,
    password: String,
    info: Admin,
}

pub struct Login {
    admins: HashMap<Username, AdminEx>,
}

impl Login {
    pub fn new() -> Login {
        Login {
            admins: HashMap::new(),
        }
    }

    pub fn generate_dummy_values(&mut self) {
        self.admins.insert(
            String::from("admin"),
            AdminEx {
                id: 1000,
                username: String::from("admin"),
                password: String::from("admin"),
                info: Admin {
                    id: 1000,
                    name: String::from("Admin"),
                },
            },
        );
    }
}

impl ILogin for Login {
    fn login_reg_desk(&self, username: &str, password: &str) -> Result<Box<dyn IRegDesk>, ()> {
        thread::sleep(time::Duration::from_millis(DELAY));

        if let Some(admin) = self.admins.get(username) {
            if admin.password == password {
                let mut reg_desk = RegDesk::new(admin.info.clone());
                reg_desk.generate_dummy_values();
                return Ok(Box::from(reg_desk));
            }
        }
        Err(())
    }
}

struct RegDesk {
    logged_in_admin: Admin,
    participants: HashMap<i32, Participant>,
    colleges: HashMap<i32, College>,
    participant_last_id: i32,
    college_last_id: i32,
}

impl RegDesk {
    fn new(logged_in_admin: Admin) -> RegDesk {
        RegDesk {
            logged_in_admin,
            participants: HashMap::new(),
            colleges: HashMap::new(),
            participant_last_id: 1000,
            college_last_id: 1000,
        }
    }

    fn generate_dummy_values(&mut self) {
        let c1 = self.college_add(String::from("NIT Calicut"));
        let c2 = self.college_add(String::from("GEC Kannur"));
        self.college_add(String::from("GEC Thrissur"));
        self.college_add(String::from("CET Trivandrum"));
        self.college_add(String::from("TKM Kollam"));
        self.college_add(String::from("Amrita Coimbatore"));

        self.participant_new(
            ParticipantInfo {
                name: String::from("Test"),
                gender: Gender::Male,
                email: String::from("test@gmail.com"),
                phone: String::from("9876567891")
            },
            c1,
        );

        let p2 = self.participant_new(
            ParticipantInfo {
                name: String::from("Test 2"),
                gender: Gender::Female,
                email: String::from("test2@gmail.com"),
                phone: String::from("1234512346")
            },
            c2,
        );

        self.participant_verify_reg(p2.reg_status.err().unwrap());
    }
}

impl IRegDesk for RegDesk {
    fn participant_new(&mut self, info: ParticipantInfo, college: College) -> Participant {
        thread::sleep(time::Duration::from_millis(DELAY));
        self.participant_last_id += 1;
        let participant = Participant {
            id: self.participant_last_id,
            info,
            college,
            reg_status: Err(ParticipantRegNotVerified {
                id: self.participant_last_id,
            }),
            hospitality: None,
            category: ParticipantCategory::Ragam
        };

        self.participants
            .insert(self.participant_last_id, participant.clone());
        participant
    }

    fn participant_get(&self, id: i32) -> Option<Participant> {
        thread::sleep(time::Duration::from_millis(DELAY));
        self.participants.get(&id).cloned()
    }

    fn participant_update(&mut self, participant: &Participant) {
        thread::sleep(time::Duration::from_millis(DELAY));
        self.participants
            .insert(participant.id, participant.clone());
    }

    fn participant_verify_reg(&mut self, p: ParticipantRegNotVerified) -> Participant {
        thread::sleep(time::Duration::from_millis(DELAY));
        let admin = self.logged_in_admin.clone();
        if let Some(participant) = self.participants.get_mut(&p.id) {
            participant.reg_status = Ok(ParticipantRegVerified { admin });
        }

        self.participant_get(p.id).unwrap()
    }

    fn participant_update_hospi(
        &mut self,
        p: Participant,
        hostel: &str,
        room: &str,
    ) -> Participant {
        thread::sleep(time::Duration::from_millis(DELAY));
        let admin = self.logged_in_admin.clone();
        if let Some(participant) = self.participants.get_mut(&p.id) {
            participant.hospitality = Some(HospitalityVerified {
                admin,
                hostel: hostel.to_owned(),
                room: room.to_owned(),
            });
        }

        self.participant_get(p.id).unwrap()
    }

    fn college_get_filtered(&self, name: &str) -> Vec<College> {
        thread::sleep(time::Duration::from_millis(DELAY));
        self.colleges
            .iter()
            .filter(|(_, c)| c.name.starts_with(name))
            .map(|(_, c)| c.clone())
            .collect()
    }

    fn college_add(&mut self, name: String) -> College {
        thread::sleep(time::Duration::from_millis(DELAY));
        self.college_last_id += 1;
        self.colleges.insert(
            self.college_last_id,
            College {
                id: self.college_last_id,
                name,
            },
        );
        self.colleges[&self.college_last_id].clone()
    }
}
