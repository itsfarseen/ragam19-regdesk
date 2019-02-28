#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CollegeID(pub(super) i32);

pub trait ICollege {
    type CollegeUpdate: ICollegeUpdate;

    fn id(&self) -> CollegeID;
    fn name(&self) -> &str;
    fn update(self) -> Self::CollegeUpdate;
}

pub trait ICollegeUpdate: ICollege {
    fn set_name(&mut self, name: &str);
}

pub trait ICollegeRepo {
    type College: ICollege;
    type CollegeUpdate: ICollegeUpdate;

    fn get_college(&self, college_id: CollegeID) -> Self::College;
    fn get_colleges(&self, filter: &str) -> Vec<Self::College>;
    fn new_college(&mut self, college_name: &str) -> CollegeID;
    fn update_college(&mut self, college: Self::CollegeUpdate) -> Self::College;
}
