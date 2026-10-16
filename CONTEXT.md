# Attack Shark X8 SE DPI Helper — Project Context

Last reviewed: 2026-10-16
Status: read-only Rust HID probe merged to `main`; Windows x64 cross-build succeeded. The experimental offline ATTACK SHARK HUB X8 SE DPI block encoder and tests are on branch `work/x8se-official-hub-codec` in PR #4; formatting, tests, Clippy, and the Windows release build pass. Physical-device validation and DPI write support remain outstanding.

## Current implementation

The Rust CLI currently provides only AttackSharkDPI.exe detect. On Windows it
enumerates HID metadata for the reported X8 SE ID (1d57:fa60) and related
GearHub vendor ID (25a7). It emits JSON and marks all records as unconfirmed
candidates. It does not open device handles, send HID reports, or include
serial numbers.

Run on a standard non-admin Windows account with the receiver connected while
the mouse is off and while it is on. Record which interface and usage entries
appear. VID/PID and product strings alone do not confirm the model.

CI formatting, tests, Clippy, and the Windows x64 release build passed. Rust
and MinGW-w64 are also installed in the build environment; the local x86_64
Windows GNU cross-build succeeded. The resulting read-only executable SHA-256
is `491ff806180a0d115c992eef870eac66826691702165b8ae06ac39d394dd6d42`.

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

The current OpenMouse Attack Shark driver routes an X8 SE with the vendor-control
collection to its `25a7` family. That path reads DPI slots with feature report
`0x00` / command `0xD4` and exposes generic DPI options. However, the current
`setDpi()` implementation calls `requireX11Dpi()` and writes through the
separate X11 report `0x04` path; it does not provide a GearHub DPI setter for
the X8 SE. OpenMouse's supported-device listing marks X8 SE as needing its
Bridge companion, which supplies native HID access, but Bridge availability is
not itself proof that writes are implemented. Its X8 SE investigation also
reports no successful feature-report exchanges in the analyzed capture.

A separate GearHub-V5 implementation documents GET_DPI (0xD4) and SET_DPI
(0x54) over a 64-byte DPI-stage table, but targets VID 0x3151. The X8 SE is
VID 0x1d57 and follows a separate driver path. Treat 0x54 and the layout as a
protocol lead only; do not send it to the X8 SE until confirmed from an X8 SE
vendor-driver capture.

The official Attack Shark download page lists X8SE among models supported by
its browser-based ATTACK SHARK HUB. Use it on physical hardware, change one
stage, capture HID traffic, compare with a preceding read, preserve the
original table, and verify only intended fields changed. Test readback after
change and across power cycles to establish persistence.

## Official ATTACK SHARK HUB bundle review (static analysis only)

Reviewed the deployed web bundle on 2026-10-16 without executing the application. The main bundle maps X8SE to device ID `0x02`; the X8 SE page calls an X8SE serializer to build a 60-byte settings block and sends a connection-method/device-ID prefix plus that block as hex through its local WebSocket helper. This confirms the browser application constructs and submits a message to its helper; it does not reveal the helper's final HID report or prove device acceptance.

The observed UI defaults are eight slots `800, 1600, 2400, 3200, 8000, 25000, 0, 0`, with stage 2 active. The block includes a profile byte, sensitivity parameters, enabled-stage mask, X/Y high-range flags, eight X and Y slot arrays, active-stage byte, eight indicator colors, indication type, a big-endian 16-bit additive checksum, and eight trailing zero bytes. DPI conversion uses a 220-entry lookup table and range-specific scaling. The source also contains a `n+++d` first-indicator loop expression that creates a byte gap/overwrite; the offline module deliberately reproduces it for comparison.

The static assets inspected were `index-CHmxtNQf.js` (SHA-256 `ae803cd1849f67bd20e0fff87af21aabdb2f918a728fecd4cc7d32ba13cbf5f0`) and `dpiX8SE-Bh7olmqY.js` (SHA-256 `661f26a4a60f5693153761606e1dabb11eeebe36fae9dc5b3a4954baba0fd5b7`). `src/x8se_official_hub.rs` mirrors only the in-memory settings-block serializer. Its golden test vector comes from the deployed JavaScript and is synthetic, not a HID capture. See `docs/x8se-official-hub-codec.md`.

Do not infer the HID report ID, report length, framing, reply, readback, persistence, or supported hardware range from this JavaScript. Do not apply OpenMouse GearHub-V5 `0x54` based on this result. Keep all HID writes disabled until a physical official-hub capture verifies the end-to-end path.

## TODO while PC unavailable

These research and preparation tasks can move forward without the mouse:

- Completed static review of the official hub bundle and added a pure offline
  60-byte X8 SE settings-block encoder plus a synthetic JavaScript reference
  vector on `work/x8se-official-hub-codec`; confirm CI and review the branch.
- Compare any candidate DPI write format with OpenMouse's X8 SE `0xD4` read
  path. Do not assume GearHub-V5's `0x54` command or 64-byte table applies to
  X8 SE.
- Keep the encoder isolated from HID. The current golden fixture is synthetic;
  replace or supplement it with a preserved real capture after hardware is
  available, then compare each byte and the checksum.
- Prepare the detection test matrix for receiver present/mouse off, mouse on,
  and multiple similar devices; ambiguous matches must fail safely.

Hardware and Windows checks to run when the X8 SE and PC are available:

- Capture the official hub changing one DPI stage. Save the preceding read,
  outgoing reports, replies, and checksums; compare the before/after data and
  identify which fields changed.
- Confirm the configuration interface and report access on a standard,
  non-admin Windows account. Record enumeration with the receiver connected
  while the mouse is off and while it is on.
- Verify the new DPI by reading it back where possible and checking actual
  pointer behavior. Test supported DPI limits/steps and persistence after
  disconnecting or power-cycling the mouse.
- Only after these checks should the helper gain a DPI setter; initially write
  one stage while preserving all other settings, and report unverified writes
  honestly if readback is unavailable.

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
