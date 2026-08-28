# Scheduled Run Receipts sample scenario

`srr demo` creates this disposable, local-only scenario in a fresh directory
under the operating system temporary directory:

- `database-backup` runs each day at 02:00 UTC with a 15-minute grace window.
- The six-day evidence window contains successful, late, missing, failed, and
  still-running receipts.
- `weekly-evidence.html` is a standalone report ready to open in a browser.

The command never reads or writes the configured `SRR_DATA` path.
