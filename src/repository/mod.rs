pub trait IParticipant {
    fn username(&self) -> String;
    fn name(&self) -> String;
    fn gender(&self) -> String;
    fn email(&self) -> String;
}

pub trait ICollege {
    fn name(&self) -> String;
}

pub trait ILoggedInAdmin {
    fn name(&self) -> String;
}

pub trait IAdminAuth<RegDesk, AdminAuth>
where
    RegDesk: IRegDesk,
    AdminAuth: IAdminAuth<RegDesk, AdminAuth>,
{
    fn signin_reg_desk(self, username: &str, password: &str) -> Result<RegDesk, AdminAuth>;
}

pub trait IRegDesk {
    type LoggedInAdmin: ILoggedInAdmin;
    type ParticipantRepo: IParticipantRepo;
    type RegVerificationMgr: IRegVerificationMgr;

    fn get_admin(&self) -> &Self::LoggedInAdmin;
    fn get_participant_repo(&self) -> &Self::ParticipantRepo;
    fn get_reg_verification(&self) -> &Self::RegVerificationMgr;
}

pub trait IParticipantRepo {
    type Participant: IParticipant;

    fn get_participant(&self, id: i32) -> Option<Self::Participant>;
    fn reset_password(&self, participant: &Self::Participant, new_password: String);
    fn update_participant(&self, updated_participant: &Self::Participant);
}

pub trait ICollegeRepo {
    type College: ICollege;
    type Participant: IParticipant;

    fn get_colleges(&self, filter: &str) -> Vec<Self::College>;
    fn participant_set_college(&self, participant: &mut Self::Participant, college: &Self::College);
    fn participant_get_college(&self, participant: &Self::Participant) -> Self::College;
}

pub trait IRegUnverifiedParticipant {}
pub trait IRegVerifiedParticipant {
    fn admin_name(&self) -> String;
}

pub trait IRegVerificationMgr {
    type Participant: IParticipant;
    type RegUnverifiedParticipant: IRegUnverifiedParticipant;
    type RegVerifiedParticipant: IRegVerifiedParticipant;

    fn is_reg_verified(
        &self,
        participant: &Self::Participant,
    ) -> Result<Self::RegVerifiedParticipant, Self::RegUnverifiedParticipant>;
    fn verify_reg(&self, participant: &Self::RegUnverifiedParticipant);
}
