use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    AgreementNotFound = 4,
    InvalidMilestone = 5,
    InvalidStatus = 6,
    MilestoneNotCompleted = 7,
    DisputeAlreadyActive = 8,
    DisputeNotActive = 9,
    DeadlinePassed = 10,
    InvalidSplit = 11,
}
