# Demo sandbox

Run `srr demo` with no setup. It creates a new random directory under the
operating system temporary directory, seeds a realistic `database-backup`
schedule, and prints paths to its state file and standalone weekly evidence
page. It never reads or writes `SRR_DATA` or a `--data` path.

The landing page also opens with the same type of sample evidence already
visible. The **Try it with sample data** action moves to that viewer; choosing
a local JSON report is local browser parsing only and does not alter CLI data.
