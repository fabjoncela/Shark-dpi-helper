# Shark DPI Helper

Early development for validating Attack Shark X8 SE HID access. The current
executable is a read-only probe. It enumerates Windows HID metadata and prints
JSON for the reported X8 SE ID or related GearHub vendor ID. It sends no HID
reports, opens no device handles, and omits serial numbers.

USB identifiers and product strings are candidate evidence only;
model_confirmed always remains false. No DPI get/set command exists yet.

The library contains two offline-only research modules:

- [Official hub settings-block encoder](docs/x8se-official-hub-codec.md): reconstructs the 60-byte X8 SE settings block. It does not send reports.
- `gearhub_25a7`: adapts OpenMouse's 0x25a7 GET_DPI request and response layout. It does not open HID or send reports. This read path has not been confirmed by a successful X8 SE hardware capture, and OpenMouse does not implement an X8 SE DPI setter.

See [the OpenMouse GearHub read-path note](docs/openmouse-gearhub-read.md) for the source mapping and hardware-validation limits.

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

## JSON output and exit codes

A matching collection appears in a successful response like this:

    {
      "ok": true,
      "read_only": true,
      "devices": [
        {
          "vendor_id": 7511,
          "product_id": 64096,
          "interface_number": 2,
          "usage_page": 65280,
          "usage": 1,
          "manufacturer": "Example",
          "product": "USB Receiver",
          "candidate_class": "x8_se_reported_id",
          "model_confirmed": false
        }
      ],
      "error": null
    }

The example values illustrate the shape only. An empty devices array means no
candidate IDs were enumerated; it does not mean no mouse is attached.

If HID API setup fails, detect prints JSON and exits with code 1:

    {
      "ok": false,
      "read_only": true,
      "devices": [],
      "error": "could not initialize Windows HID API: ..."
    }

Exit codes: 0 means detection ran successfully (including zero matches); 1 means
HID API initialization failed; 2 is Clap's command-line usage error, printed
to stderr. Command-line usage errors may not be JSON.
