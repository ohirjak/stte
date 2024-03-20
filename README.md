# Simple Toy Transactions Engine

## Requirements interpretation

### Dispute deposit assumptions

When `disputed transaction amount > client available amount` => allow negative number in available amount.

### Dispute withdrawal assumptions

Provided dispute process describes/matches only handling/reversing of deposit transactions. Assuming handling of withdrawals:

The only action in the whole dispute withdrawal handling is to increase available amount (= reverse withdrawal) and lock account on chargeback.
Starting the dispute changes only `dispute_state` of the transaction.

### Creating new client considerations

Currently, a new client is created irrespectively of transaction type if one for particular client id does not exist.
However, new client should probably be created only when processing `Deposit` type of transactions...

## Implementation notes

Using type system for checking input file correctness. Checking presence and validity of the `amount` field for deposit/withdrawal transactions programmatically.
Field `amount` and internally stored amounts use type `Decimal` from crate [rust_decimal](https://crates.io/crates/rust_decimal).
See crate description: "Decimal number implementation written in pure Rust suitable for financial and fixed-precision calculations."

Error handling with crate [thiserror](https://crates.io/crates/thiserror).

### Testing

Testing correctness of transaction processing with unit tests.

```sh
$ cargo test
```

Testing checking of input csv file validity with invalid input files in `data` folder.

```sh
$ NUM=1
$ cargo run -- data/input-invalid${NUM}.csv
```

Testing more complicated transaction "flows" with `data/input-flow?.csv` / `data/output-flow?.csv` files.

## Running

```sh
$ cargo run -- transactions.csv > accounts.csv
```
