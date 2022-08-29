# Clarity Native Function Benchmarking
Benchmarking suite to determine constants for usage in the Clarity Cost Functions boot contract

## Usage

### Setup 

Ensure that the `blockstack-core` dependency is set currently. This library 
relies on a modified version of the repository. The branch it currently depends on is called 
`clarity-benchmarking-stacks-2.1`.

### Running the benchmark suite

Running the benchmarking suite can be done via the cargo `bench` command:

```
cargo bench
```

Benchmarking results will be outputted to the `target/criterion/` directory.

To obtain the costs-2 and costs-3 contract, we ran the benchmarks on a machine 
with the following specs on Google Cloud Platform:
- CPUs (4)
- CPU Platform (Intel Cascade Lake)
- GCP Machine type (n2-standard-4)
- Memory (16GB)
- Hard Drive size (500GB)
- Hard Drive type (Standard persistent disk)
- GCP Network Tier (Premium)

### Running regression analysis

Once the benchmarking results have been collected, analysis scripts are
used to compute regressions on the data, outputting estimated functions
for each cost function. These estimated functions have units of
`nanoseconds per 100 executions` in the `y` dimension, and the input
unit for the Clarity cost function in the `x` dimension.

The analysis script requires `SciPy`, so for convenience, a Dockerfile is
included to perform analysis.

Via docker:
```
docker build -f analysis/analysis.Dockerfile --build-arg criterion_dir="latest-data/criterion" -o /tmp/analysis-output/ .
```

The `criterion_dir` optional argument tells the analysis script where in
the current directory to look for the benchmarking outputs. `latest-data`
in this repository contains the data from the execution used to produce
the current proposal data.

This will output `cost_constants.csv` to `/tmp/analysis-output` and graphs
of the analyzed data and regression.

### Translating regression analysis into proposed costs

Once the regression is performed, the proposed cost functions need to
scale those estimates into the correct dimensions for Clarity costs.
The runtime block limit in the Stacks network is `5e9` (unitless), and
the goal of the current proposal is that this should correspond to 30
seconds or `3e10` nanoseconds. The `proposal/make_cost_functions.py`
script performs that scaling and formats output appropriate for a new
costs contract (i.e., Clarity language formatted) and for inclusion in
markdown proposals (a markdown-formatted table).

To run this script, you will need to copy over the computed runtime
data. Before doing so, you will want to remove the 4 lines associated with
the various analysis passes (those do not go into the costs contract):
cost_arithmetic_only_checker, cost_read_only, cost_trait_checker, and
cost_type_checker. 

```
cd ./proposal
cp /tmp/analysis-output/cost_constants.csv ./estimated_constants.csv
python ./make_cost_functions.py
```

This will output to `new_costs.clar` and `updates_table.md`
