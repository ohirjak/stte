use std::collections::HashMap;

use crate::types::{
    Client, ClientId, DisputeState, EngineError, StoredTransaction, Transaction, TransactionType,
};

pub struct Engine {
    clients: HashMap<ClientId, Client>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn read_and_process_input(&mut self, filename: &String) -> Result<(), EngineError> {
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_path(filename)?;

        for result in reader.deserialize() {
            self.process_transaction(result?)?;
        }

        Ok(())
    }

    pub fn print_clients(&self) {
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

#[cfg(test)]
#[path = "engine.test.rs"]
mod tests;
