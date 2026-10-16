# Attack Shark X8 SE DPI Helper — Project Context

Last reviewed: 2026-10-16  
Status: repository created; planning only; no helper implementation exists yet.

## Purpose

Build a small, portable Windows command-line helper that lets CafeLauncher apply the selected profile's existing DPI value to an Attack Shark X8 SE. Keep it in a separate repository from CafeLauncher and keep v1 limited to DPI control for this mouse.

The helper should run without installation and, if hardware access permits, without administrator rights. CafeLauncher should download it only when a supported X8 SE is detected, verify a pinned SHA-256 checksum, and continue applying the rest of the profile if the download or DPI operation fails.

## Decisions already made

- Proposed repository: `fabjoncela/AttackShark-DPI-Helper`.
- Proposed executable: `AttackSharkDPI.exe` (Windows x64, portable).
- Reuse CafeLauncher's existing profile DPI field and its better-logihub hybrid download/execution pattern; inspect the current implementation before integrating.
- Do not change Logitech Superlight 1/2 behavior.
- Do not change polling rate, lighting, macros, or other mouse settings in v1.
- Do not merge CafeLauncher integration until the helper is tested with a physical X8 SE.
- A receiver being plugged in does not prove that its paired mouse is powered on or responsive. Never select a device by enumeration order when multiple mouse families are present.

## Upstream review: important limitation

OpenMouse's public `session.md` describes an X8 SE capture with VID `0x1d57`, PID `0xfa60`, and four HID interfaces. It says the initial family detection routed this device to the wrong protocol and records a fix that identifies a vendor-control collection and uses the GearHub (`25a7`) protocol. The file lists feature report `0x00` with 64-byte commands, command `0x80` for firmware, and command `0xD4` for DPI slots. The current `mouse-protocol` Attack Shark driver contains a GearHub DPI read path, but the inspected `setDpi()` path only supports its separate `1d57-x11` family. Therefore the current upstream code does **not** establish that X8 SE DPI writes work. Treat it as useful protocol and detection evidence, not as proof that the requested feature is already solved.

The upstream driver also records model IDs inside receiver messages, including `0x02` for X8 SE, and notes that several Attack Shark models share USB product IDs. Do not identify or target a mouse from VID/PID alone. Confirm the correct HID interface and model evidence on the actual X8 SE hardware.

The upstream UI's listed DPI values are candidate presets, not confirmed X8 SE write limits. Verify accepted DPI values and how the mouse responds to unsupported values on hardware.

## Phase 1 must answer before implementation

1. **Can Windows user-mode HID APIs reach the X8 SE configuration interface?** Confirm the exact interface/collection and report length on a standard, non-admin account. Do not assume browser WebHID access implies Windows native access, or vice versa.
2. **What command writes DPI?** Locate the relevant GearHub command in upstream history, the vendor's WebHID bundle, or other reliable protocol evidence. Confirm its request/response framing and checksum. The currently inspected OpenMouse driver reads DPI slots but does not expose a GearHub DPI setter.
3. **What does a DPI write change?** Establish whether it updates the active DPI stage, a stored device profile, or persistent onboard memory. Preserve other DPI stages and settings. Do not claim changes are temporary until tested across mouse power cycles and PC restarts.
4. **How can the helper prove the correct mouse is active?** Test receiver-present/mouse-off and mouse-on cases. If safe, use a bounded read command or receiver model message as a responsiveness check; otherwise report ambiguity and skip.
5. **Which DPI values are actually accepted?** Check the X8 SE's real limits and steps. Validate inputs before sending commands; do not silently map a request unless the mapping is tested and documented.

Phase 1 is complete only after a physical X8 SE can be identified, its DPI changed safely, and the result independently confirmed where readback is available. If reliable readback is not supported, report “command sent; not independently verified,” not a fabricated value.

## Command-line interface target

Keep output machine-readable and exit codes stable. Suggested initial commands:

```text
AttackSharkDPI.exe detect
AttackSharkDPI.exe dpi get
AttackSharkDPI.exe dpi set 800
```

`dpi get` should return an explicit unsupported result if the device cannot report current DPI. `dpi set` should validate the measured range/steps, write only the intended DPI value, use bounded HID operations, and return a clear error for unsupported or ambiguous hardware. Prefer JSON output with fields such as `ok`, `device`, `dpi`, `verified`, and `error`.

## CafeLauncher integration requirements

- On profile selection, read the latest requested DPI from the existing profile field.
- Keep Logitech handling on the existing better-logihub path. Run the X8 SE helper only for a confidently identified X8 SE; never run both helpers indiscriminately.
- Do not download the Attack Shark helper on Logitech-only or unsupported systems.
- Cache under `%LOCALAPPDATA%\CafeLauncher\Tools\`; download to a temporary file, verify the pinned checksum, then atomically move it into place.
- Reuse existing async download, locking, retry, and logging conventions. Do not build a second general download framework.
- Continue all other profile work while the helper downloads. If profile buttons are clicked rapidly, apply only the newest queued DPI after the helper is ready.
- Bound helper execution with a timeout. On failure, log a useful reason and continue the launcher.
- Keep the tested release version and SHA-256 pinned in CafeLauncher; never execute an unverified or arbitrary latest binary.
- Deep Freeze may clear the local helper cache on reboot. Re-download on the next detection when needed.

## Acceptance checks

- X8 SE connected and responsive: requested DPI is applied and verified where the protocol supports readback.
- Receiver connected but mouse off: helper exits safely; CafeLauncher continues.
- Logitech SL1/SL2 only: existing behavior is unchanged and no Attack Shark helper is downloaded.
- No supported mouse: no helper download or DPI action.
- Both device families present: no arbitrary targeting; skip or use a verified explicit selection.
- Slow/failed download, checksum mismatch, helper timeout, or HID error: no unverified execution; remaining profile actions continue.
- Multiple quick profile selections: only the latest pending DPI request is sent.
- Standard non-admin Windows account and a clean Deep Freeze cache: behavior is tested and documented.

## Repository bootstrap and first work

The GitHub repository `fabjoncela/Shark-dpi-helper` now exists and contains this initial context file. Start with Phase 1 rather than implementing a guessed DPI write command. Keep the helper's protocol tests runnable without hardware; add real-device validation as a separate required release gate.

## References

- [OpenMouse X8 SE support investigation](https://github.com/OpenMouse-Project/openmouse/blob/main/session.md)
- [OpenMouse Attack Shark HID driver](https://github.com/OpenMouse-Project/mouse-protocol/blob/main/src/drivers/attackshark/hid.ts)
- [OpenMouse mouse-protocol repository](https://github.com/OpenMouse-Project/mouse-protocol)
- [CafeLauncher repository](https://github.com/fabjoncela/Cs-script)
