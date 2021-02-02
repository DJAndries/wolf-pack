#!/bin/sh

cargo build

trap "kill 0" EXIT

./target/debug/wolf-pack --server &

sleep 1

./target/debug/wolf-pack --windowed --switcher --fps 127.0.0.1 darnell
