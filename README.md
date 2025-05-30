# Framework for efficient Secure Collaborative Analytics

Meeting each Friday at 16:00


# Structure of Repo


## Documentation
- `/docs` - all documentation and material is contained here
- `/docs/preparation` - extra preparation material
- `/docs/Secrecy` - working project of Secrecy test
- `/fesca` - main project
- `/fesca/client` - main project from side of client
- `/fesca/cloud` - main project from side of cloud
- `/fesca/analyst` - main project from side of data analyst



# Preparation Material

## Project Profile
[Overleaf](https://sharelatex.tu-darmstadt.de/project/681dcd5358308663611983b5)

## Rust

- [Rust book](https://doc.rust-lang.org/book/)
- [Rust exercises](https://github.com/rust-lang/rustlings)

## Query papers 

- [Oblivious Query Execution](https://github.com/CASP-Systems-BU/Secrecy/tree/main)
- [Conversion of some traditional database operators and basic logic gates](https://www.usenix.org/system/files/nsdi23-liagouris.pdf)
- Micro-benchmark of the framework on some real-world datasets (e.g., Hospital, Financial Organization)

## Special preparation
- [Execution of XOR, AND gate protocol](./docs/)

# Goals

## Common Goals

- Implement a communication framework (based on gRPC) that allows computing nodes to exchange messages in general  => Rust: tonic
- Implement the Replicated Secret Sharing (RSS) protocol that includes:
- - Correlated Randomness Generation
- - Message Exchange among computing nodes
- - Execution of XOR, AND gate protocol  (I will send you a one-bit example)

## Goals for 9 CP 
- Integrate advanced oblivious operators (LIKE, Order By...)
- Optimizations of existing protocols or queries (Reduce online computing costs...)
- An accurate model to convert queries to basic logic gate (not just rule-based)
- A full benchmark on more real world queries
