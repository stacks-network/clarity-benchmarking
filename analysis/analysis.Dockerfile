FROM jupyter/scipy-notebook AS build-stage

WORKDIR /src/

ARG criterion_dir=target/criterion

COPY . .
RUN python ./analysis/cost_estimator.py "$criterion_dir"

FROM scratch AS export-stage
COPY --from=build-stage /src/analysis_target /
