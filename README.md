This repository implements the transaction processor as described in the problem statement. 

# Build and Run

Build the code using 
```
cargo build
```

Run the code as follows. 
```
cargo run -- path/to/file.csv
```
The specified file should be a csv file formatted according to the format in the problem statement.

Run the tests
```
cargo test
```

# Key Assumptions
* A client's available balance cannot go negative. Instead the transaction that would cause this should be ignored.
* Only deposits can be disputed (It is unclear from the problem statement if withdrawals can also be disputed.
  Realistically it seems like they could be. But the description for dispute handling suggests it only covers deposits).

# Design
The program maintains two "databases" (implemented as hashmaps), which store client accounts, and transactions that 
may be disputed. The code iterates over the transaction log one record at a time. For each record it processes the transaction, by finding and updating the relevant account, and if necessary, storing the teransaction for future dispute.

This design uses system resources efficiently. It extends to receiving transactions over a different protocol (e.g
a stream). It can scale by making the databases external components, and running multiple transaction processors in parallel.

## Modules
The code is split across 3 modules:
* `transaction.rs` contains the code for parsing a transaction log, and structs/enums for handing different transaction types.
* `account.rs` contains code for handling client accounts, including the logic for deposits, withdrawals, disputes, resolutions and cargebacks.
* `main.rs` drives the flow of execution, and manages the "databases" (hash maps) that are needed for the program. 

## Key crates
* `serde` and `csv`: For handling the transaction log, and outputting account data.
* `rust_decimal`: For handling floating point operations safely without rounding errors
* `clap`: For argument parsing and help text (might be overkill)

# Testing / Correctness

The type system is used where practical to ensure correctness. In particular, only validly formed transaction events are passed into the internal business logic. 

The code has primarily been tested in two ways:
* Via the module-level unit tests.
* By running sample input files by hand and inspecting the output. See samples in `test-data/`. This includes the sample input and output provided in the problem statment.

# To Do
* Add logging to the program (especially when errors are created).
* Add more checks to input parsing, specifically that deposits and withdrawals have at most 4 digits after the decimal, and other transactions do not have amounts.
* We could tidy up the code to generate and propogate errors by using the `anyhow` crate.
* Type aliases for the primitives (e.g. transaction ID, client ID, amounts) to improve readability and maintainability.
* End-to-end regression testing. Either via UTs in `main.rs` or a shell script that runs the program over the samples and checks output against a reference.
* Introduce traits for the client and deposit databases, so they can be replaced by external databases in future. This will also simulation of database failures.


