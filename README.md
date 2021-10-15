# Clarity Native Function Benchmarking
Benchmarking suite to determine constants for usage in the Clarity Cost Functions boot contract

## Usage

### Running the benchmark suite

Running the benchmarking suite can be done via the cargo `bench` command:

```
cargo bench
```

Benchmarking results will be outputted to the `target/criterion/` directory.


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

To run this script:

```
cd ./proposal
cp /tmp/analysis-output/cost_constants.csv ./estimated_constants.csv
python ./make_cost_functions.py
```

This will output to `new_costs.clar` and `updates_table.md`
