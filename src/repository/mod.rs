#![allow(dead_code)]

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
    pub fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Clone)]
pub struct ParticipantInfo {
    pub name: String,
    pub gender: Gender,
    pub email: String,
}

#[derive(Copy, Clone)]
pub enum Gender {
    Male, Female, Other
}

#[derive(Clone)]
pub struct College {
    id: i32,
    pub name: String,
}

impl College {
    pub fn id(&self) -> i32 {
        self.id
    }
}

pub trait ILogin: Send + Sync {
    fn login_reg_desk(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Box<dyn IRegDesk>, ()>;
}

pub trait IRegDesk: Send + Sync {
    fn participant_new(&mut self, info: ParticipantInfo, college: College) -> Participant;
    fn participant_get(&self, id: i32) -> Option<Participant>;
    fn participant_update(&mut self, participant: &Participant);
    fn participant_verify_reg(&mut self, p: ParticipantRegNotVerified) -> Participant;
    // TODO: Implement fuzzy search
    fn college_get_filtered(&self, name: &str) -> Vec<College>;
    fn college_add(&mut self, name: String) -> College;
}
