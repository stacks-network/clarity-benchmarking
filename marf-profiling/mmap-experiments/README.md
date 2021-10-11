# `mmap_size` experiments

This is some python analysis code for analyzing the Marf read times with and
without using the `mmap_size` pragma.



## Usage

### Extract Raw Data
The script `extract_data` expects as input the LOGS output from running this code:
https://github.com/blockstack/stacks-blockchain/pull/2867

In particular, we are scraping lines of the form:

```
info!("MARF read"; "db" => c.db_path , "time_micros" => duration);
```

To run the code, run a line like:

```
head -n 1000000 ~/data/no_mmap.log | python3 extract_data.py no_mmap > data/no_mmap.txt
```

### Average the Data



