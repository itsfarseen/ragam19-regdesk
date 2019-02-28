use super::{IParticipantRepo, IRegVerificationMgr};

pub trait ILoggedInAdmin {
    fn name(&self) -> &str;
}

pub struct RegDesk<LoggedInAdmin, ParticipantRepo, RegVerificationMgr>
where
    LoggedInAdmin: ILoggedInAdmin,
    ParticipantRepo: IParticipantRepo,
    RegVerificationMgr: IRegVerificationMgr,
{
    admin: LoggedInAdmin,
    participant_repo: ParticipantRepo,
    reg_verification_mgr: RegVerificationMgr,
}

pub trait IAdminAuth<AdminAuth>
where
    AdminAuth: IAdminAuth<AdminAuth>,
{
    type LoggedInAdmin: ILoggedInAdmin;
    type ParticipantRepo: IParticipantRepo;
    type RegVerificationMgr: IRegVerificationMgr;

    #[allow(clippy::type_complexity)]
    fn signin_reg_desk(
        self,
        username: &str,
        password: &str,
    ) -> Result<
        RegDesk<Self::LoggedInAdmin, Self::ParticipantRepo, Self::RegVerificationMgr>,
        AdminAuth,
    >;
}
