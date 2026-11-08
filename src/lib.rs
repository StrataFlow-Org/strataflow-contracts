#![no_std]

pub mod errors;
pub mod events;
pub mod storage;
pub mod types;
pub mod contract;

#[cfg(test)]
mod test;

pub use contract::{StrataFlowContract, StrataFlowContractClient};
pub use errors::Error;
pub use types::{
    Agreement, AgreementStatus, AgreementSummary, DataKey, Milestone, MilestoneStatus,
    ProtocolFeeConfig, MAX_DESC_LENGTH, MAX_FEE_BPS, MAX_MILESTONES, MAX_PROOF_LENGTH,
};
