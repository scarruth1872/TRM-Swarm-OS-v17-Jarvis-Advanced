#!/bin/sh
# Windows: rustc --edition 2021 -O -o phx_k3.exe main.rs
# These modules are copied from trm_uefi_kernel/src so the tool compiles standalone.
rustc --edition 2021 -O -o phx_k3.exe main.rs
