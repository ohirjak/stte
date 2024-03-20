use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

pub(crate) type ClientId = u16;
type TransactionId = u32;
type Amount = Decimal;

#[derive(Deserialize)]
pub(crate) struct Transaction {
    /// Using `tx_type` because `type` is reserved word.
    #[serde(alias = "type")]
    pub(crate) tx_type: TransactionType,
    pub(crate) client: ClientId,
    pub(crate) tx: TransactionId,
    /// Hint serde to use string instead of float for amount deserialization to avoid rounding errors.
    #[serde(with = "rust_decimal::serde::str_option")]
    pub(crate) amount: Option<Amount>,
}

impl Transaction {
    pub(crate) fn get_amount(&self) -> Result<Amount, EngineError> {
        let amount = self.amount.ok_or(EngineError::AmountMissing(self.tx))?;
        if amount <= Decimal::ZERO {
            return Err(EngineError::AmountNotPositive(amount));
        }
        Ok(amount)
    }
}

#[derive(PartialEq)]
pub(crate) enum DisputeState {
    /// Initial state / set on resolve
    None,
    /// Set on dispute
    Open,
    /// Final state set on chargeback
    Chargeback,
}

/// Structure for storing transaction for potential disputes.
pub(crate) struct StoredTransaction {
    pub(crate) tx_type: TransactionType,
    pub(crate) amount: Amount,
    pub(crate) dispute_state: DisputeState,
}

impl StoredTransaction {
    pub(crate) fn new(tx_type: TransactionType, amount: Amount) -> Self {
        assert!(tx_type == TransactionType::Deposit || tx_type == TransactionType::Withdrawal);

        Self {
            tx_type,
            amount,
            dispute_state: DisputeState::None,
        }
    }
}

/// Client == Account, as stated in requirements: "The client has a single asset account."
#[derive(Default)]
pub(crate) struct Client {
    pub(crate) available: Amount,
    pub(crate) held: Amount,
    pub(crate) locked: bool,
    pub(crate) transactions: HashMap<TransactionId, StoredTransaction>,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Error parsing CSV input: {0}")]
    InvalidInput(#[from] csv::Error),
    #[error("Missing amount field in transaction with id: {0}")]
    AmountMissing(TransactionId),
    #[error("Amount must be positive: {0}")]
    AmountNotPositive(Decimal),
}
