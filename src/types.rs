use soroban_sdk::{contracttype, Address, String, Vec};

pub const DAY_IN_LEDGERS: u32 = 17280;
pub const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub const PERSISTENT_BUMP_AMOUNT: u32 = 120 * DAY_IN_LEDGERS;
pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = PERSISTENT_BUMP_AMOUNT - 30 * DAY_IN_LEDGERS;

pub const MAX_MILESTONES: u32 = 25;
pub const MAX_DESC_LENGTH: u32 = 256;
pub const MAX_PROOF_LENGTH: u32 = 128;
pub const MAX_FEE_BPS: u32 = 1000; // Max protocol fee 10%
pub const MIN_MILESTONE_AMOUNT: i128 = 1; // Strict floor > 0
pub const MAX_AGREEMENT_DURATION: u64 = 365 * 86400; // Max 1 year ceiling
pub const MIN_AGREEMENT_DURATION: u64 = 3600; // Min 1 hour duration
pub const DEFAULT_DISPUTE_WINDOW: u64 = 30 * 86400; // 30-day arbitration resolution window

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgreementSummary {
    pub id: u64,
    pub client: Address,
    pub provider: Address,
    pub arbiter: Address,
    pub token: Address,
    pub total_amount: i128,
    pub balance: i128,
    pub status: AgreementStatus,
    pub milestones_count: u32,
    pub deadline: u64,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolFeeConfig {
    pub fee_bps: u32,
    pub recipient: Address,
}

#[contracttype]
pub enum DataKey {
    Admin,
    AgreementCounter,
    Agreement(u64),
    IsPaused,
    PendingAdmin,
    FeeConfig,
    DisputeTimestamp(u64),
    MilestoneRejectionCount(u64, u32),
    ProtocolMetrics,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestonesStatusBreakdown {
    pub pending: u32,
    pub submitted: u32,
    pub approved: u32,
    pub disputed: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractMetadata {
    pub name: String,
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolMetrics {
    pub total_escrow_volume: i128,
    pub total_agreements_created: u64,
    pub total_agreements_completed: u64,
    pub total_agreements_disputed: u64,
}

pub fn checked_stroop_to_xlm(stroops: i128) -> Option<i128> {
    stroops.checked_div(10_000_000)
}

pub fn checked_xlm_to_stroop(xlm: i128) -> Option<i128> {
    xlm.checked_mul(10_000_000)
}

pub fn is_valid_agreement_transition(from: AgreementStatus, to: AgreementStatus) -> bool {
    match (from, to) {
        (AgreementStatus::Created, AgreementStatus::Funded) => true,
        (AgreementStatus::Created, AgreementStatus::Cancelled) => true,
        (AgreementStatus::Funded, AgreementStatus::Completed) => true,
        (AgreementStatus::Funded, AgreementStatus::Disputed) => true,
        (AgreementStatus::Funded, AgreementStatus::Cancelled) => true,
        (AgreementStatus::Disputed, AgreementStatus::Resolved) => true,
        (AgreementStatus::Disputed, AgreementStatus::Completed) => true,
        _ => false,
    }
}

impl ContractError {
    pub fn description(&self) -> &'static str {
        match self {
            ContractError::AlreadyInitialized => "Protocol is already initialized",
            ContractError::Unauthorized => "Caller is not authorized for this action",
            ContractError::AgreementNotFound => "Requested agreement does not exist",
            ContractError::AgreementAlreadyFunded => "Agreement is already funded",
            ContractError::AgreementNotActive => "Agreement is not in an active state",
            ContractError::AgreementExpired => "Agreement deadline has passed",
            ContractError::AgreementNotExpired => "Agreement deadline has not yet passed",
            ContractError::InvalidMilestone => "Invalid milestone index or state",
            ContractError::MilestoneAlreadyApproved => "Milestone has already been approved",
            ContractError::InsufficientEscrowBalance => "Insufficient balance in escrow",
            ContractError::DisputeAlreadyRaised => "Dispute is already open for agreement",
            ContractError::NoActiveDispute => "No active dispute found for agreement",
            ContractError::InvalidDisputeSplit => {
                "Dispute resolution split amounts do not match escrow balance"
            }
            ContractError::ContractPaused => "Protocol contract operations are currently paused",
            ContractError::ProtocolFeeTooHigh => "Protocol fee basis points exceed allowable limit",
            ContractError::NoNominatedAdmin => "No nominated admin pending acceptance",
            ContractError::NominatedAdminMismatch => {
                "Caller does not match nominated admin address"
            }
            ContractError::MilestoneAlreadyCompleted => "Milestone is already completed",
            ContractError::MilestoneNotPendingApproval => "Milestone is not pending approval",
            ContractError::ArbiterAlreadyAssigned => "Arbiter is already assigned to this address",
            ContractError::DisputeWindowNotPassed => "Dispute resolution window has not elapsed",
            ContractError::AgreementAlreadyCompleted => "Agreement has already completed",
            ContractError::MilestoneAmountBelowMinimum => {
                "Milestone amount is below minimum threshold"
            }
            ContractError::AgreementDurationTooShort => {
                "Agreement duration is shorter than minimum allowed"
            }
            ContractError::AgreementDurationExceeded => {
                "Agreement duration exceeds maximum allowed limit"
            }
            ContractError::MilestoneDescriptionEmpty => "Milestone description cannot be empty",
            ContractError::MilestoneIndexOutOfBounds => "Milestone index is out of bounds",
        }
    }
}
