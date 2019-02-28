use super::IParticipant;

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
    fn verify_reg(&mut self, participant: &Self::RegUnverifiedParticipant);
}