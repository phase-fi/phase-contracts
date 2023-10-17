# Phase finance DCA contracts

This repo contains all the phase finance smart contracts for DCA

## Building
Run the following two commands to get the repo ready for running tests/deploying/etc.

`RUSTFLAGS='-C link-arg=-s' cargo wasm-dca`

`RUSTFLAGS='-C link-arg=-s' cargo wasm-router`

## Testing
`cargo test -p pf-dca`