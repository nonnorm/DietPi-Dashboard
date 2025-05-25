#!/bin/bash -ex

./scripts/build-assets.bash

cargo build --release --bins
