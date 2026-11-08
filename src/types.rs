use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AgreementStatus {
    Created = 0,
    Funded = 1,
    Completed = 2,
    Disputed = 3,
    Resolved = 4,
    Cancelled = 5,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MilestoneStatus {
    Pending = 0,
    Submitted = 1,
    Approved = 2,
    Disputed = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub description: String,
    pub amount: i128,
    pub status: MilestoneStatus,
    pub proof_hash: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Agreement {
    pub id: u64,
    pub client: Address,
    pub provider: Address,
    pub arbiter: Address,
    pub token: Address,
    pub total_amount: i128,
    pub balance: i128,
    pub status: AgreementStatus,
    pub milestones: Vec<Milestone>,
    pub deadline: u64,
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    AgreementCounter,
    Agreement(u64),
}
