use super::CollegeID;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ParticipantID(pub(super) i32);

pub trait IParticipant {
    type ParticipantUpdate: IParticipant;

    fn id(&self) -> ParticipantID;
    fn username(&self) -> &str;
    fn name(&self) -> &str;
    fn college_id(&self) -> CollegeID;
    fn gender(&self) -> &str;
    fn email(&self) -> &str;
    fn update(self) -> Self::ParticipantUpdate;
}

pub trait IParticipantUpdate: IParticipant {
    fn set_password(&mut self, password: &str);
    fn set_gender(&mut self, gender: &str);
    fn set_email(&mut self, email: &str);
    fn set_college_id(&mut self, college_id: CollegeID);
}

#[derive(Clone)]
pub struct NewParticipant {
    pub username: String,
    pub password: String,
    pub name: String,
    pub college_id: CollegeID,
    pub gender: String,
    pub email: String,
}

pub trait IParticipantRepo {
    type Participant: IParticipant;
    type ParticipantUpdate: IParticipantUpdate;

    fn get_participant(&self, id: i32) -> Option<Self::Participant>;
    fn update_participant(&mut self, updated_participant: Self::ParticipantUpdate) -> Self::Participant;
    fn new_participant(&mut self, new_participant: NewParticipant) -> Self::Participant;
}