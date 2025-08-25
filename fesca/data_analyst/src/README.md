# How does the query translation into MPC primitives work.
State of 15.08.2025

### The procedure itself

0. There is a table employees.csv under fesca\data_analyst\src\employees.csv . The testing and "small" benchmarking was done using this file. There is another file customers-10000.csv generated using TPCH, which will be used for "big" benchmarking later.
1. A Catalog struct implemented in fesca\data_analyst\src\local_exec.rs translates this CSV into a table object, which SQL engine can later operate on. 
2. The in 'fesca\data_analyst\src\lib.rs' hardocded string "SELECT AVG(salary) FROM employees WHERE dept = 'R&D'" is tokenized and translated into AST (Abstract Syntax Tree) by integrated built-in Rust original crate sqlparser-rs. Improvement idea: accept queries via CLI.
3. The AST object is translated into logical plan in 'fesca\data_analyst\src\sql_to_logical.rs'. This script walks through the AST and translated it to pre-defined Logical Plan units (so called bricks) that are defined in 'fesca\data_analyst\src\logical_plan.rs'
4. The logical plan object is then parsed to logical -> MPC circuits translator. The translator ('fesca\data_analyst\src\logical_to_circuits.rs') walks through the Logical Plan object and translates them into predefined circuits using circuit builder ('fesca\data_analyst\src\circuit_builder.rs'). An object of Struct Circuit is then delivered.

Logical Plan may contain (for now) following elements:
1. Scan (The SELECT translation). It iterates over all rows of the provided table.
2. Filter, the translation of WHERE. It applies equality/inequality to hardcoded string (in our hardcoded query ...HWERE dept == 'R&D') implemented as XOR/XNOR in circuit level. In this case the dept column is hardcoded to id 1 (see pt. 3 in Very important) and the R&D string is converted to bitstring by circuit builder. The comparison is done by XNORing (equality) of each table entry with the hardcoded string.
3. Aggregate on hardcoded column 0 (salary). Again, right now it's just parity.

> ### Very important to-do's until the end of the project:
> 1. The translators assume that each entry in table is only 1 bit, so the produced query is translated only to //one// object of Circuit. All circuit elements are connected with each other   with an AND. This means later (either in protocol itself or on the data analyst side) the code needs to be adjusted to accept bitstrings.
> 2. The translators assign wires based on //the table size//. Right now the table size is hardcoded to 2 columns and 5 rows, so there are "as if" 10 imput wires. Output wires are then computed dynamically by the program.
>3. The AVG function needs to be implemented as Ripple-Carry adder or Carry-lookahead adder. Right now only parity is computed instead of AVG (parity is a XOR b XOR c = bit)

### The Circuit object
Circuit object is a placeholder for actual AND/XOR gates protocol. The protocol can then be integrated as a plug-in solution. Right now the Circuit object contains number of all wires for the *whole* table, **optimize it to have only 1 circuit per row, that will be iterated over the whole table.**
Currently it has (example, //this is only 1 layer of the gate network computing parity of 4 pairs of bits, without AND. If you want for testing you can manually add an AND gate to the object):
```
Circuit object: Circuit {
        wire_count: 14,
        gates: [
            Input {
                output: 0,
            },
            Input {
                output: 1,
            },
            Input {
                output: 2,
            },
            Input {
                output: 3,
            },
            Input {
                output: 4,
            },
            Input {
                output: 5,
            },
            Input {
                output: 6,
            },
            Input {
                output: 7,
            },
            Input {
                output: 8,
            },
            Input {
                output: 9,
            },
            ----- beginning of parity/aggregation
            Xor {
                left: 1,
                right: 3,
                output: 10,
            },
            Xor {
                left: 10,
                right: 5,
                output: 11,
            },
            Xor {
                left: 11,
                right: 7,
                output: 12,
            },
            Xor {
                left: 12,
                right: 9,
                output: 13,
            },
            ----- end of parity/aggregation
        ],
        outputs: [
            13,
        ],
    }
```

An employees table for explanation reasons:
```
employees
dept (0)          salary (1)     Aggr - paritaet R&D // ASCII - 3 bytes //0/0..100 XOR 1 - 1 
0 //wire 0          1 //1           a
------------------------------ add a new column, for only one predicate (avg)
1 //wire 2          0 //3           b
------------------------------
1 //wire 4          0 //4           c
------------------------------
0 //wire 6          1 //7           d
1 //wire 8          0 //9           e // output wire 13 a xor b xor c ...
```