# Attack Shark X8 SE DPI Helper — Project Context

Last reviewed: 2026-10-16
Status: read-only HID probe implemented on work/x8se-readonly-hid-probe; physical-device validation and all DPI control remain outstanding.

## Current implementation

The Rust CLI currently provides only AttackSharkDPI.exe detect. On Windows it
enumerates HID metadata for the reported X8 SE ID (1d57:fa60) and related
GearHub vendor ID (25a7). It emits JSON and marks all records as unconfirmed
candidates. It does not open device handles, send HID reports, or include
serial numbers.

Run on a standard non-admin Windows account with the receiver connected while
the mouse is off and while it is on. Record which interface and usage entries
appear. VID/PID and product strings alone do not confirm the model.

CI runs formatting, tests, Clippy, and a release build on Windows. Local Rust
toolchain was unavailable during initial implementation; remote CI must
establish build status.

## Product requirements

Build a portable Windows helper that applies the CafeLauncher profile DPI to
Attack Shark X8 SE. Keep it separate from CafeLauncher and limit v1 to DPI.

- Proposed executable: AttackSharkDPI.exe (Windows x64, portable).
- Reuse CafeLauncher's existing profile DPI and better-logihub hybrid download/execution pattern; inspect current implementation before integration.
- Do not change Logitech Superlight 1/2 behavior or other mouse settings.
- Do not integrate until physical X8 SE testing succeeds.
- Never select by enumeration order when multiple mouse families are present.

CafeLauncher should download only for a supported X8 SE, verify a pinned SHA-256,
cache under %LOCALAPPDATA%\CafeLauncher\Tools\, and continue other profile
actions if download or DPI control fails. Download to a temporary file, verify,
then atomically move. Reuse existing async download, locking, retry, and logging
patterns. For rapid profile changes, apply only the newest pending DPI. Bound
helper execution and continue the launcher after failures. Pin tested release
and checksum; never execute an unverified latest binary. Deep Freeze may clear
the cache, requiring download again after detection.

## Protocol evidence and limitations

OpenMouse's public session.md describes an X8 SE capture with VID 0x1d57,
PID 0xfa60, and four HID interfaces. It records that initial family detection
routed the device incorrectly and describes a vendor-control collection and
GearHub (25a7) protocol. It lists feature report 0x00 with 64-byte commands,
0x80 for firmware, and 0xD4 for DPI slots.

The current OpenMouse Attack Shark driver has a GearHub DPI read path, but its
setDpi() path only supports its separate 1d57-x11 family. This does not prove
X8 SE DPI writes work.

A separate GearHub-V5 implementation documents GET_DPI (0xD4) and SET_DPI
(0x54) over a 64-byte DPI-stage table, but targets VID 0x3151. Treat 0x54 and
the layout as unverified for X8 SE.

The official Attack Shark download page lists X8SE among models supported by
its browser-based ATTACK SHARK HUB. Use it on physical hardware, change one
stage, capture HID traffic, compare with a preceding read, preserve the
original table, and verify only intended fields changed. Test readback after
change and across power cycles to establish persistence.

## Phase 1 hardware investigation

Before implementing any HID read/write command:

1. Confirm exact configuration interface/collection on real hardware and a standard non-admin account. WebHID access does not prove native Windows access.
2. Capture official tool DPI-change traffic. Establish command, report ID, length, framing, response, and checksum.
3. Determine whether the write changes active DPI, a stored profile, or persistent memory.
4. Test receiver-present/mouse-off versus mouse-on. If safe, use bounded read/model evidence; otherwise skip ambiguous cases.
5. Measure accepted DPI values and steps. Never silently map untested values.
6. Independently verify changed DPI. If readback is unavailable, report “command sent; not independently verified.”

Phase 1 completes only when physical X8 SE identification, safe DPI change, and independent verification succeed where supported.

## Later CLI target

    AttackSharkDPI.exe detect
    AttackSharkDPI.exe dpi get
    AttackSharkDPI.exe dpi set 800

Keep JSON output and stable exit codes, with explicit unsupported and ambiguity
errors. Bound HID operations and preserve all unrelated DPI stages/settings.

## Acceptance checks

- Responsive X8 SE: requested DPI applied and verified where possible.
- Receiver connected but mouse off: exits safely; CafeLauncher continues.
- Logitech SL1/SL2 only: current behavior unchanged; no helper download.
- No supported mouse: no download or DPI action.
- Both device families: no arbitrary targeting.
- Download/checksum/timeout/HID failures: no unverified execution; other profile work continues.
- Rapid profile changes: only latest pending DPI sent.
- Standard non-admin account and clean Deep Freeze cache tested.

## References

- OpenMouse X8 SE investigation: https://github.com/OpenMouse-Project/openmouse/blob/main/session.md
- OpenMouse Attack Shark HID driver: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/src/drivers/attackshark/hid.ts
- OpenMouse GearHub-V5 codec: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/src/gearhub/index.ts
- Official Attack Shark hub: https://www.attackshark.cn/system.aspx?ClassID=18
- CafeLauncher: https://github.com/fabjoncela/Cs-script

## CafeLauncher integration review (current main)

Reviewed CafeLauncher main at the time of this note. Its Logitech path is the
reference pattern for a future X8 SE integration, but X8 SE should use its own
helper identity, cache and pinned release once validated.

- The launcher currently stores the Logitech helper under
  %LOCALAPPDATA%\CafeLauncher\logihub, with a pinned release URL and SHA-256.
  The original X8 plan calls for the new helper under
  %LOCALAPPDATA%\CafeLauncher\Tools.
- A separate hidden PowerShell worker downloads to a unique temporary file,
  has a 30-second request timeout, checks SHA-256, then moves the verified
  file to the executable cache. Completion is handed back to the GUI using a
  status file and timer; network work does not block the GUI.
- Profile requests update one pending DPI value. When downloads overlap with
  profile clicks, the latest requested DPI is used after the helper is ready.
  Other profile work proceeds independently.
- Before use, the cached executable hash is checked again. Helper failures are
  logged as step outcomes and do not terminate the launcher.
- Device selection uses machine-readable enumeration, exact supported model
  IDs, connected state, and explicit device index. It refuses zero or multiple
  matches instead of choosing the first entry. The selected device identity is
  rechecked before applying and reading back DPI.
- DPI is verified by reading it back. G HUB is left running and may override a
  live value later. Onboard DPI writes are intentionally not used. The
  Superlight 1 host-mode fallback is model-specific and should not be carried
  over to the Attack Shark helper without separate evidence.

Future X8 work should reuse the launcher’s async-download, hash-before-cache,
latest-request, bounded execution, logging, and continue-on-failure behavior.
Do not reuse Logitech command arguments, cache identity, device model checks,
or the Superlight-specific fallback. CafeLauncher integration remains deferred
until the X8 SE collection and DPI protocol pass physical-device validation.
