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
    MilestoneLimitExceeded = 12,
    ProtocolPaused = 13,
    MilestoneOutOfOrder = 14,
    DeadlineNotPassed = 15,
    CannotCancelFunded = 16,
    InvalidFeeBps = 17,
    NoPendingAdmin = 18,
    DisputeTimeoutNotPassed = 19,
    InvalidAmount = 20,
    MilestoneNotSubmitted = 21,
    DurationExceeded = 22,
    SameArbiter = 23,
    AgreementDurationTooShort = 24,
}
