# Shark DPI Helper

Early development for validating Attack Shark X8 SE HID access. The current
executable is a read-only probe. It enumerates Windows HID metadata and prints
JSON for the reported X8 SE ID or related GearHub vendor ID. It sends no HID
reports, opens no device handles, and omits serial numbers.

USB identifiers and product strings are candidate evidence only;
model_confirmed always remains false. No DPI get/set command exists yet.

## Build on Windows

Install Rust using rustup, then run:

    cargo test
    cargo build --release
    .\target\release\AttackSharkDPI.exe detect

Run on a standard non-admin Windows account with the X8 SE receiver connected,
first while the mouse is off and then while it is on. Save the JSON output from
each case. Do not share raw device paths or serial numbers.

## Safety gate

Do not add DPI writes based only on the 1d57:fa60 identifier or the unverified
GearHub 0x54 lead. Capture the official utility's traffic on physical X8 SE
hardware, identify the precise collection and report framing, preserve
untouched DPI stages, and independently confirm the result. See CONTEXT.md.

## Checks

    cargo fmt --check
    cargo test
    cargo clippy --all-targets -- -D warnings
