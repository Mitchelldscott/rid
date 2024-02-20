# rid

This crate is meant to be shared by a client and host communicating over USB. 

Currently the only host programs are tests in this repo. 

To run the tests make sure to use the std feature

    cargo test --features="std" <test_name>

To build and open the docs run

    cargo doc --open
