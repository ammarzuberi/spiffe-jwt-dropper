# SPIFFE JWT Dropper

A simple program that fetches JWT SVIDs from the SPIFFE Workload API and writes them to a specified file path.

## Usage

Set environment variables `JWT_AUD` (audience), `JWT_PATH` (output file), and optionally `WORKLOAD_API_PATH` (defaults to `/var/run/spire-agent/api.sock`)
