# rid

This crate is meant to be shared by a client and host communicating over USB. 

Currently the only host programs are tests in this repo. 

To run the tests make sure to use the std feature

    cargo test --features="std" <test_name>

To build and open the docs run

    cargo doc --open

## Test Results

![RID linear offset conversion](doc/ptp_results/ptp_offset_err.png)
PTP Offset Error: difference in measured times and estimated times

![RID linear offset conversion](doc/ptp_results/offset.png)
PTP Offset: offset calculation over time

![RID linear offset conversion](doc/ptp_results/linear_conv.png)
Linear offset conversion: C(t) = m * H(t) + b

![RID Packet flight time](doc/ptp_results/flight_time.png)
Packet flight times: host time (milliseconds) vs flight time (microseconds)
