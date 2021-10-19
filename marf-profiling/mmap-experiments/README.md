# `mmap_size` experiments

This directory contains python analysis code for analyzing the Marf read times with and
without using the `mmap_size` pragma.

## Usage

### Extract Raw Data (in Rust)
The script `extract_data.py` expects as STDIN input the `LOGS` output by
running this code:
https://github.com/blockstack/stacks-blockchain/pull/2867

In particular, we are scraping lines output by the Rust code:

```
info!("MARF read"; "db" => c.db_path , "time_micros" => duration);
```

To run the code, use a line like:

```
head -n 1000000 ~/data/no_mmap.log | python3 extract_data.py no_mmap > data/no_mmap.txt
```

### Summarize the Data
Get the average and length of the data by running:

````
python3 summarize_data.py data/no_mmap.txt
```

### Moving Average

To calculate a rolling moving average over the time series data run:

```
python3 moving_average_over_time.py data/with_mmap.txt
```
