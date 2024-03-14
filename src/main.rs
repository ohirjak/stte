use std::{collections::HashMap, env};

use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

type ClientId = u16;
type TransactionId = u32;
type Amount = Decimal;

#[derive(Deserialize)]
struct Transaction {
    /// Using `tx_type` because `type` is reserved word.
    #[serde(alias = "type")]
    tx_type: TransactionType,
    client: ClientId,
    tx: TransactionId,
    /// Hint serde to use string instead of float for amount deserialization to avoid rounding errors.
    #[serde(with = "rust_decimal::serde::str_option")]
    amount: Option<Amount>,
}

impl Transaction {
    fn get_amount(&self) -> Result<Amount, EngineError> {
        let amount = self.amount.ok_or(EngineError::AmountMissing(self.tx))?;
        if amount <= Decimal::ZERO {
            return Err(EngineError::AmountNotPositive(amount));
        }
        Ok(amount)
    }
}

#[derive(PartialEq)]
enum DisputeState {
    /// Initial state / set on resolve
    None,
    /// Set on dispute
    Open,
    /// Final state set on chargeback
    Chargeback,
}

/// Structure for storing transaction for potential disputes.
struct StoredTransaction {
    tx_type: TransactionType,
    amount: Amount,
    dispute_state: DisputeState,
}

impl StoredTransaction {
    fn new(tx_type: TransactionType, amount: Amount) -> Self {
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
struct Client {
    available: Amount,
    held: Amount,
    locked: bool,
    transactions: HashMap<TransactionId, StoredTransaction>,
}

struct Engine {
    clients: HashMap<ClientId, Client>,
}

impl Engine {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    fn read_and_process_input(&mut self, filename: &String) -> Result<(), EngineError> {
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_path(filename)?;

        for result in reader.deserialize() {
            self.process_transaction(result?)?;
        }

        Ok(())
    }

    fn print_clients(&self) {
        println!("client, available, held, total, locked");

        for (client_id, client) in &self.clients {
            println!(
                "{}, {}, {}, {}, {}",
                client_id,
                client.available,
                client.held,
                client.available + client.held,
                client.locked
            );
        }
    }

    /// Process all types of transactions.
    ///
    /// Note: Not splitting processing of particular transaction types into separate functions as actual processing is quite simple.
    fn process_transaction(&mut self, transaction: Transaction) -> Result<(), EngineError> {
        let client = self.clients.entry(transaction.client).or_default();

        if client.locked {
            return Ok(());
        }

        match transaction.tx_type {
            TransactionType::Deposit => {
                let amount = transaction.get_amount()?;

                client.available += amount;
                client.transactions.insert(
                    transaction.tx,
                    StoredTransaction::new(transaction.tx_type, amount),
                );
            }
            TransactionType::Withdrawal => {
                let amount = transaction.get_amount()?;

                if client.available >= amount {
                    client.available -= amount;
                    client.transactions.insert(
                        transaction.tx,
                        StoredTransaction::new(transaction.tx_type, amount),
                    );
                }
            }
            TransactionType::Dispute => {
                if let Some(disputed_trans) = client.transactions.get_mut(&transaction.tx) {
                    if disputed_trans.dispute_state == DisputeState::None {
                        disputed_trans.dispute_state = DisputeState::Open;

                        if disputed_trans.tx_type == TransactionType::Deposit {
                            client.available -= disputed_trans.amount;
                            client.held += disputed_trans.amount;
                        }
                    }
                }
            }
            TransactionType::Resolve => {
                if let Some(disputed_trans) = client.transactions.get_mut(&transaction.tx) {
                    if disputed_trans.dispute_state == DisputeState::Open {
                        disputed_trans.dispute_state = DisputeState::None;

                        if disputed_trans.tx_type == TransactionType::Deposit {
                            client.available += disputed_trans.amount;
                            client.held -= disputed_trans.amount;
                        }
                    }
                }
            }
            TransactionType::Chargeback => {
                if let Some(disputed_trans) = client.transactions.get_mut(&transaction.tx) {
                    if disputed_trans.dispute_state == DisputeState::Open {
                        disputed_trans.dispute_state = DisputeState::Chargeback;
                        client.locked = true;

                        match disputed_trans.tx_type {
                            TransactionType::Deposit => client.held -= disputed_trans.amount,
                            TransactionType::Withdrawal => {
                                client.available += disputed_trans.amount
                            }
                            TransactionType::Dispute
                            | TransactionType::Resolve
                            | TransactionType::Chargeback => panic!("Cannot get here"),
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn get_filename_argument() -> Result<String, EngineError> {
    env::args().nth(1).ok_or(EngineError::ArgumentMissing)
}

fn run() -> Result<(), EngineError> {
    let filename = get_filename_argument()?;
    let mut engine = Engine::new();

    engine.read_and_process_input(&filename)?;
    engine.print_clients();

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("Error occured: {}", err);
    }
}

#[derive(Error, Debug)]
enum EngineError {
    #[error("Missing filename argument")]
    ArgumentMissing,
    #[error("Error parsing CSV input: {0}")]
    InvalidInput(#[from] csv::Error),
    #[error("Missing amount field in transaction with id: {0}")]
    AmountMissing(TransactionId),
    #[error("Amount must be positive: {0}")]
    AmountNotPositive(Decimal),
}
