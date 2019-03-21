use super::*;
use mysql::prelude::*;
use std::sync::{Arc, Mutex};

pub struct Login {
    conn: Arc<Mutex<mysql::Conn>>,
}

impl Login {
    pub fn new(mut conn: mysql::Conn) -> Self {
        let setup_sql = [
            r"CREATE TABLE IF NOT EXISTS `admin` (
                `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                `name` varchar(255) NOT NULL,
                `username` varchar(255) NOT NULL,
                `password` varchar(255) NOT NULL
            );",
            r"CREATE TABLE IF NOT EXISTS `participant` (
                `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                `college_id` INT NOT NULL,
                `email` VARCHAR(255) NOT NULL,
                `password` VARCHAR(255) NOT NULL,
                `name` VARCHAR(255) NOT NULL,
                `phone` VARCHAR(255) NOT NULL,
                `gender` INT NOT NULL,
                `category` INT NOT NULL
            );",
            r"CREATE TABLE IF NOT EXISTS `college` (
                `id` INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                `name` VARCHAR(255) NOT NULL
            );",
            r"CREATE TABLE IF NOT EXISTS `offline_reg` (
                `participant_id` int PRIMARY KEY NOT NULL,
                `admin_id` int NOT NULL
            );",
            r"CREATE TABLE IF NOT EXISTS `hospitality_reg` (
                `participant_id` INT PRIMARY KEY NOT NULL,
                `admin_id` INT NOT NULL,
                `room` VARCHAR(255) NOT NULL,
                `hostel` VARCHAR(255) NOT NULL
            );",
        ];
        for stmt in setup_sql.iter() {
            conn.prep_exec(stmt, ()).unwrap();
        }
        Self {
            conn: Arc::from(Mutex::from(conn)),
        }
    }
}

impl ILogin for Login {
    fn login_reg_desk(&self, username: &str, password: &str) -> Result<Box<dyn IRegDesk>, ()> {
        let admins: Vec<Admin> = self
            .conn
            .lock()
            .unwrap()
            .prep_exec(
                r"SELECT id,name from `admin` WHERE `username`=? and `password`=?",
                (username, password),
            )
            .map(|result| {
                result
                    .map(|x| x.unwrap())
                    .map(|row| {
                        let (id, name): (i32, String) = mysql::from_row(row);
                        Admin { id, name }
                    })
                    .collect()
            })
            .unwrap();
        if admins.len() > 0 {
            Ok(Box::from(RegDesk {
                conn: self.conn.clone(),
                admin: admins[0].clone(),
            }))
        } else {
            Err(())
        }
    }
}

struct RegDesk {
    conn: Arc<Mutex<mysql::Conn>>,
    admin: Admin,
}

impl IRegDesk for RegDesk {
    fn participant_new_verified(&mut self, info: ParticipantInfo, college: College) -> Participant {
        let last_insert_id = {
        let info_ = info.clone();
        let mut lock = self.conn.lock().unwrap();
        let res = lock
            .prep_exec(
                r"
            INSERT INTO participant(college_id, email, password, name, phone, gender, category) VALUES(
                ?,?,?,?,?,?,?
            )",
                (
                    college.id(),
                    info_.email,
                    String::from("password"),
                    info_.name,
                    info_.phone,
                    gender_to_i32(info_.gender),
                    category_to_i32(info_.category),
                ),
            )
            .unwrap();
            res.last_insert_id() as i32
        };
        
        return self.participant_verify_reg(ParticipantRegNotVerified{id: last_insert_id});
    }

    fn participant_get(&self, id: i32) -> Option<Participant> {
        let mut lock = self.conn.lock().unwrap();
        let mut stmt = lock
            .prepare(
                r"
                SELECT participant.id,

                       participant.name,
                       gender,
                       email,
                       phone,
                       category,

                       college_id,
                       college.name as college_name,

                       r_admin.id,
                       r_admin.name,

                       h_admin.id,
                       h_admin.name,
                       hospitality_reg.hostel,
                       hospitality_reg.room

                FROM `participant` 
                JOIN college ON participant.college_id=college.id
                LEFT JOIN offline_reg on participant.id=offline_reg.participant_id
                LEFT JOIN admin as r_admin on r_admin.id=offline_reg.admin_id
                LEFT JOIN hospitality_reg on participant.id=hospitality_reg.participant_id
                LEFT JOIN admin as h_admin on h_admin.id=hospitality_reg.admin_id
                WHERE participant.id=?",
            )
            .unwrap();
        let row = stmt.execute((id,)).unwrap().last();
        if row.is_none() {
            return None;
        }
        let row = row.unwrap().unwrap();

        let (r_admin_id, r_admin_name) = (row.get(8).unwrap(), row.get(9).unwrap());
        let (h_admin_id, h_admin_name) = (row.get(10).unwrap(), row.get(11).unwrap());
        Some(Participant {
            id: row.get_opt(0).unwrap().expect("0"),
            info: ParticipantInfo {
                name: row.get_opt(1).unwrap().expect("1"),
                gender: gender_from_i32(row.get_opt(2).unwrap().expect("2")),
                email: row.get_opt(3).unwrap().expect("3"),
                phone: row.get_opt(4).unwrap().expect("4"),
                category: category_from_i32(row.get_opt(5).unwrap().expect("5")),
            },
            college: College {
                id: row.get_opt(6).unwrap().expect("6"),
                name: row.get_opt(7).unwrap().expect("7"),
            },
            reg_status: match (r_admin_id, r_admin_name) {
                (Some(id), Some(name)) => Ok(ParticipantRegVerified {
                    admin: Admin { id, name },
                }),
                _ => Err(ParticipantRegNotVerified { id }),
            },
            hospitality: match (h_admin_id, h_admin_name) {
                (Some(id), Some(name)) => Some(HospitalityVerified {
                    admin: Admin { id, name },
                    hostel: row.get_opt(12).unwrap().expect("12"),
                    room: row.get_opt(13).unwrap().expect("13"),
                }),
                _ => None,
            },
        })
    }
    fn participant_update(&mut self, participant: &Participant) {
        let mut lock = self.conn.lock().unwrap();
        let res = lock
            .prep_exec(
                r"UPDATE participant SET college_id=?, email=?, password=?, name=?, phone=?, gender=?, category=? WHERE id=?",
                (
                    participant.college.id(),
                    participant.info.email.clone(),
                    String::from("password"),
                    participant.info.name.clone(),
                    participant.info.phone.clone(),
                    gender_to_i32(participant.info.gender),
                    category_to_i32(participant.info.category),
                    participant.id
                ),
            )
            .unwrap();
    }

    fn participant_verify_reg(&mut self, p: ParticipantRegNotVerified) -> Participant {
        {
            let mut lock = self.conn.lock().unwrap();
            let res = lock
                .prep_exec(
                    r"INSERT INTO offline_reg(participant_id, admin_id) VALUES(?,?)",
                    (p.id, self.admin.id),
                )
                .unwrap();
        }
        self.participant_get(p.id).unwrap()
    }

    fn participant_update_hospi(
        &mut self,
        p: Participant,
        hostel: &str,
        room: &str,
    ) -> Participant {
        {
            let mut lock = self.conn.lock().unwrap();
            let res = lock
            .prep_exec(
                r"INSERT INTO hospitality_reg(participant_id, admin_id, hostel, room) VALUES(?,?,?,?) ON DUPLICATE KEY UPDATE hostel=VALUES(hostel), room=VALUES(room);",
                (p.id, self.admin.id, hostel, room),
            )
            .unwrap();
        }
        self.participant_get(p.id).unwrap()
    }

    fn college_get_filtered(&self, _name: &str) -> Vec<College> {
        let colleges: Vec<College> = self
            .conn
            .lock()
            .unwrap()
            .prep_exec(r"SELECT id,name from `college`", ())
            .map(|result| {
                result
                    .map(|x| x.unwrap())
                    .map(|row| {
                        let (id, name): (i32, String) = mysql::from_row(row);
                        College { id, name }
                    })
                    .collect()
            })
            .unwrap();
        colleges
    }

    fn college_add(&mut self, name: String) -> College {
        let mut lock = self.conn.lock().unwrap();
        let res = lock
            .prep_exec(r"INSERT INTO `college`(name) VALUES(?)", (name.clone(),))
            .unwrap();
        College {
            id: res.last_insert_id() as i32,
            name,
        }
    }
}

fn gender_to_i32(gender: Gender) -> i32 {
    match gender {
        Gender::Male => 0,
        Gender::Female => 1,
        Gender::Other => 2,
    }
}

fn gender_from_i32(gender: i32) -> Gender {
    match gender {
        0 => Gender::Male,
        1 => Gender::Female,
        _ => Gender::Other,
    }
}

fn category_to_i32(cat: ParticipantCategory) -> i32 {
    match cat {
        ParticipantCategory::Ragam => 0,
        ParticipantCategory::Kalotsavam => 1,
    }
}

fn category_from_i32(cat: i32) -> ParticipantCategory {
    match cat {
        0 => ParticipantCategory::Ragam,
        _ => ParticipantCategory::Kalotsavam,
    }
}
