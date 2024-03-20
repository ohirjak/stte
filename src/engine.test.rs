use std::collections::HashMap;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    engine::Engine,
    types::{Client, DisputeState, StoredTransaction, Transaction, TransactionType},
};

#[test]
#[should_panic(expected = "AmountMissing(1)")]
fn test_no_amount_in_deposit() {
    let mut engine = Engine::new();

    let deposit_no_amount = Transaction {
        tx_type: TransactionType::Deposit,
        client: 1,
        tx: 1,
        amount: None,
    };

    engine.process_transaction(deposit_no_amount).unwrap();
}

#[test]
#[should_panic(expected = "AmountMissing(1)")]
fn test_no_amount_in_withdrawal() {
    let mut engine = Engine::new();

    let withdrawal_no_amount = Transaction {
        tx_type: TransactionType::Withdrawal,
        client: 1,
        tx: 1,
        amount: None,
    };

    engine.process_transaction(withdrawal_no_amount).unwrap();
}

#[test]
#[should_panic(expected = "AmountNotPositive(0.0)")]
fn test_amount_must_not_be_zero() {
    let mut engine = Engine::new();

    let withdrawal_amount_zero = Transaction {
        tx_type: TransactionType::Withdrawal,
        client: 1,
        tx: 1,
        amount: Some(dec!(+0.0)),
    };

    engine.process_transaction(withdrawal_amount_zero).unwrap();
}

#[test]
#[should_panic(expected = "AmountNotPositive(-1)")]
fn test_amount_must_be_positive() {
    let mut engine = Engine::new();

    let withdrawal_amount_zero = Transaction {
        tx_type: TransactionType::Withdrawal,
        client: 1,
        tx: 1,
        amount: Some(dec!(-1)),
    };

    engine.process_transaction(withdrawal_amount_zero).unwrap();
}

#[test]
fn test_deposit() {
    let transactions_and_clients = [
        (
            // deposit to new client
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([(
                        1,
                        StoredTransaction {
                            tx_type: TransactionType::Deposit,
                            amount: dec!(1.5),
                            dispute_state: DisputeState::None,
                        },
                    )]),
                },
            )]),
        ),
        (
            // deposit to existing client
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 2,
                amount: Some(dec!(2.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(4),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(2.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
    ];

    test_transactions!(transactions_and_clients);
}

#[test]
fn test_withdrawal() {
    let transactions_and_clients = [
        (
            // try to withdraw from non-existing client
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 1,
                amount: Some(dec!(2.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::new(),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 2,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([(
                        2,
                        StoredTransaction {
                            tx_type: TransactionType::Deposit,
                            amount: dec!(1.5),
                            dispute_state: DisputeState::None,
                        },
                    )]),
                },
            )]),
        ),
        (
            // withdraw valid amount from existing client
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(dec!(0.4)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            3,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // try to withdraw invalid amount from existing client
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 4,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            3,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
    ];

    test_transactions!(transactions_and_clients);
}

#[test]
fn test_dispute_deposit() {
    let transactions_and_clients = [
        (
            // try to dispute on non-existing client
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::new(),
                },
            )]),
        ),
        (
            // try to dispute on non-existing transaction
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::new(),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([(
                        1,
                        StoredTransaction {
                            tx_type: TransactionType::Deposit,
                            amount: dec!(1.5),
                            dispute_state: DisputeState::None,
                        },
                    )]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(dec!(0.4)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // dispute not-yet disputed deposit
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(-0.4),
                    held: dec!(1.5),
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::Open,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // repeatedly try to dispute the same deposit
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(-0.4),
                    held: dec!(1.5),
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::Open,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
    ];

    test_transactions!(transactions_and_clients);
}

#[test]
fn test_dispute_withdrawal() {
    let transactions_and_clients = [
        (
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([(
                        1,
                        StoredTransaction {
                            tx_type: TransactionType::Deposit,
                            amount: dec!(1.5),
                            dispute_state: DisputeState::None,
                        },
                    )]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(dec!(0.4)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // dispute not-yet disputed withdrawal
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Open,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // repeatedly try to dispute the same withdrawal
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Open,
                            },
                        ),
                    ]),
                },
            )]),
        ),
    ];

    test_transactions!(transactions_and_clients);
}

#[test]
fn test_resolve_deposit() {
    let transactions_and_clients = [
        (
            // try to resolve on non-existing client
            Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::new(),
                },
            )]),
        ),
        (
            // try to resolve on non-existing transaction
            Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::new(),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([(
                        1,
                        StoredTransaction {
                            tx_type: TransactionType::Deposit,
                            amount: dec!(1.5),
                            dispute_state: DisputeState::None,
                        },
                    )]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(dec!(0.4)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // try to resolve not disputed deposit
            Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(-0.4),
                    held: dec!(1.5),
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::Open,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // resolve disputed deposit
            Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // repeatedly try to resolve already resolved = non-disputed deposit
            Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
    ];

    test_transactions!(transactions_and_clients);
}

#[test]
fn test_resolve_withdrawal() {
    let transactions_and_clients = [
        (
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([(
                        1,
                        StoredTransaction {
                            tx_type: TransactionType::Deposit,
                            amount: dec!(1.5),
                            dispute_state: DisputeState::None,
                        },
                    )]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(dec!(0.4)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Open,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // resolve disputed withdrawal
            Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 2,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // repeatedly try to resolve already resolved = non-disputed withdrawal
            Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 2,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
    ];

    test_transactions!(transactions_and_clients);
}

#[test]
fn test_chargeback_deposit() {
    let transactions_and_clients = [
        (
            // try to chargeback non-existing client
            Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::new(),
                },
            )]),
        ),
        (
            // try to chargeback non-existing transaction
            Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: Decimal::ZERO,
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::new(),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([(
                        1,
                        StoredTransaction {
                            tx_type: TransactionType::Deposit,
                            amount: dec!(1.5),
                            dispute_state: DisputeState::None,
                        },
                    )]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(dec!(0.4)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // try to chargeback non-disputed transaction
            Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 1,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(-0.4),
                    held: dec!(1.5),
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::Open,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // chargeback deposit => lock client account
            Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(-0.4),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
    ];

    test_transactions!(transactions_and_clients);
}

#[test]
fn test_chargeback_withdrawal() {
    let transactions_and_clients = [
        (
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([(
                        1,
                        StoredTransaction {
                            tx_type: TransactionType::Deposit,
                            amount: dec!(1.5),
                            dispute_state: DisputeState::None,
                        },
                    )]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(dec!(0.4)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // try to chargeback non-disputed withdrawal
            Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 2,
                amount: Some(dec!(1.5)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::None,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.1),
                    held: Decimal::ZERO,
                    locked: false,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Open,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            // chargeback withdrawal => lock client acccount & all transactions from now on are effectively no-op
            Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 2,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Deposit,
                client: 1,
                tx: 4,
                amount: Some(dec!(20.0)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Withdrawal,
                client: 1,
                tx: 5,
                amount: Some(dec!(5.0)),
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 4,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Resolve,
                client: 1,
                tx: 4,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Dispute,
                client: 1,
                tx: 5,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 1,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                    ]),
                },
            )]),
        ),
        (
            Transaction {
                tx_type: TransactionType::Chargeback,
                client: 1,
                tx: 5,
                amount: None,
            },
            HashMap::from([(
                1,
                Client {
                    available: dec!(1.5),
                    held: Decimal::ZERO,
                    locked: true,
                    transactions: HashMap::from([
                        (
                            1,
                            StoredTransaction {
                                tx_type: TransactionType::Deposit,
                                amount: dec!(1.5),
                                dispute_state: DisputeState::None,
                            },
                        ),
                        (
                            2,
                            StoredTransaction {
                                tx_type: TransactionType::Withdrawal,
                                amount: dec!(0.4),
                                dispute_state: DisputeState::Chargeback,
                            },
                        ),
                    ]),
                },
            )]),
        ),
    ];

    test_transactions!(transactions_and_clients);
}

/// Step through all transactions and check resulting clients state
macro_rules! test_transactions {
    ( $t_a_c:ident ) => {
        let mut engine = Engine::new();
        assert!(engine.clients.is_empty());

        for (trans, clients) in $t_a_c {
            assert!(engine.process_transaction(trans).is_ok());

            assert_eq!(engine.clients, clients);
        }
    };
}
use test_transactions;
