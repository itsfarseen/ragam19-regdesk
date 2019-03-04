#![allow(dead_code)]
use std::sync::Arc;

pub mod impl_in_mem;

#[derive(Clone)]
pub struct Admin {
    pub id: i32,
    pub name: String,
}

#[derive(Clone)]
pub struct Participant {
    id: i32,
    pub info: ParticipantInfo,
    pub college: College,
    pub reg_status: Result<ParticipantRegVerified, ParticipantRegNotVerified>,
}

#[derive(Clone)]
pub struct ParticipantRegVerified {
    pub admin: Admin,
}

#[derive(Copy, Clone)]
pub struct ParticipantRegNotVerified {
    id: i32,
}

impl Participant {
    fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Clone)]
pub struct ParticipantInfo {
    pub username: String,
    pub name: String,
    pub gender: String,
    pub email: String,
}

#[derive(Clone)]
pub struct College {
    id: i32,
    pub name: String,
}

impl College {
    fn id(&self) -> i32 {
        self.id
    }
}

pub enum RegistrationStatus {
    Verified { admin: Admin },
    NotVerified { id: i32 },
}

pub trait ILogin {
    fn login_reg_desk(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Arc<dyn IRegDesk + Send + Sync>, ()>;
}

pub trait IRegDesk {
    fn participant_new(&mut self, info: ParticipantInfo, college: College) -> Participant;
    fn participant_get(&self, id: i32) -> Option<Participant>;
    fn participant_update_info(&mut self, id: i32, info: ParticipantInfo) -> Option<Participant>;
    fn participant_update_college(&mut self, id: i32, college: College) -> Option<Participant>;
    fn participant_verify_reg(&mut self, p: ParticipantRegNotVerified) -> Participant;
    fn college_get_filtered(&self, name: &str) -> Vec<College>;
    fn college_add(&mut self, name: String) -> College;
}
