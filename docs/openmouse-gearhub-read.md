# OpenMouse GearHub DPI read-path adaptation

## What was adapted

The Rust library module `src/gearhub_25a7.rs` mirrors the X8 SE GearHub/MU
GET_DPI request and response shape used by OpenMouse's Attack Shark driver:

- Feature report ID: `0x00`
- Report payload length: 64 bytes
- GET_DPI command: `0xD4`, followed by the profile number
- Command checksum: the two-byte command is padded to nine bytes; byte 7 is
  `0xFF - (sum of bytes 0..6 mod 256)`
- Response: command at byte 0, active slot index at byte 2, slot count at
  byte 3, and little-endian DPI values from byte 8

The module only constructs and parses bytes in memory. It is not connected to
the CLI and does not open the HID device or send a feature report.

## What this does not establish

OpenMouse's current Attack Shark `setDpi()` uses its separate X11 report
`0x04` path. It does not implement a GearHub DPI setter for the X8 SE.
OpenMouse's X8 SE investigation also reports no successful feature-report
exchange in its analyzed capture. The adapted GET_DPI shape is therefore a
candidate read protocol, not a verified X8 SE transaction.

The 60-byte settings block built by the official web hub is a different
application-level block. This module does not explain how the native helper
frames that block for HID.

Do not use GearHub-V5's `SET_DPI 0x54` command or 64-byte DPI table for the
X8 SE based on a shared protocol label. That code targets a different VID and
device family.

## When hardware is available

1. Save the current X8 SE settings.
2. Run `AttackSharkDPI.exe detect` with the receiver present and the mouse
   both off and on; identify the vendor-control collection.
3. Use the official hub to change one DPI stage while capturing actual USB HID
   traffic. Record report ID, transmitted length, response, and interface.
4. Compare the captured GET_DPI request/reply with the offline codec. Do not
   add a CLI HID read until report framing and a successful reply are verified.
5. Capture a one-stage DPI change separately before implementing a setter.
   Preserve the other stages and verify the value by readback and pointer test.

## Source reviewed

- OpenMouse Attack Shark driver:
  https://github.com/OpenMouse-Project/mouse-protocol/blob/main/src/drivers/attackshark/hid.ts
- OpenMouse X8 SE investigation:
  https://github.com/OpenMouse-Project/openmouse/blob/main/session.md

Reviewed on 2026-10-16. The source code review is not a hardware capture.
