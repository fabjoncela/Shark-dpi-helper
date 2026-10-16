//! Offline reconstruction of OpenMouse's 0x25a7 GearHub/MU DPI read codec.
//!
//! This encodes the read request and parses the response shape used by
//! OpenMouse's Attack Shark driver for the 0x25a7 family. It is protocol
//! research only: it does not open HID devices or send reports. In particular,
//! OpenMouse does not implement a DPI setter for the X8 SE on this path.

use std::fmt;

pub const REPORT_ID: u8 = 0x00;
pub const REPORT_LEN: usize = 64;
pub const COMMAND_LEN: usize = 9;
pub const GET_DPI_COMMAND: u8 = 0xd4;
pub const STAGE_CAPACITY: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DpiProfile {
    pub active_stage_index: usize,
    pub stages: Vec<u16>,
}

impl DpiProfile {
    pub fn active_dpi(&self) -> Option<u16> {
        self.stages.get(self.active_stage_index).copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    WrongReportLength(usize),
    WrongResponseCommand(u8),
    TooManyStages(u8),
    TruncatedStageTable { expected: usize, actual: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongReportLength(actual) => {
                write!(f, "expected a {REPORT_LEN}-byte report, got {actual} bytes")
            }
            Self::WrongResponseCommand(command) => {
                write!(f, "expected GET_DPI response 0x{GET_DPI_COMMAND:02x}, got 0x{command:02x}")
            }
            Self::TooManyStages(count) => {
                write!(f, "response reports {count} DPI stages; maximum is {STAGE_CAPACITY}")
            }
            Self::TruncatedStageTable { expected, actual } => {
                write!(f, "DPI stage table needs {expected} bytes, response has {actual}")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Build OpenMouse's 64-byte GET_DPI feature-report payload for one profile.
///
/// The command is [0xd4, profile], padded to nine bytes. The checksum is
/// stored at byte 7 and the remaining report bytes are zero-filled. Report ID
/// 0x00 is separate from this payload in WebHID; native HID framing still
/// needs to be verified before sending this from the helper.
pub fn encode_get_dpi_report(profile: u8) -> [u8; REPORT_LEN] {
    let mut report = [0u8; REPORT_LEN];
    report[0] = GET_DPI_COMMAND;
    report[1] = profile;

    let sum = report[..7]
        .iter()
        .fold(0u8, |sum, byte| sum.wrapping_add(*byte));
    report[7] = 0xffu8.wrapping_sub(sum);
    report
}

/// Parse the response shape currently read by OpenMouse's GearHub path.
///
/// OpenMouse reads slot_count little-endian DPI values from byte 8 onward
/// and uses byte 2 as the active slot index (falling back to slot zero when
/// the byte is greater than eight). This preserves those semantics for
/// comparison; it is not yet confirmed against a successful X8 SE capture.
pub fn decode_get_dpi_response(bytes: &[u8]) -> Result<DpiProfile, DecodeError> {
    if bytes.len() != REPORT_LEN {
        return Err(DecodeError::WrongReportLength(bytes.len()));
    }
    if bytes[0] != GET_DPI_COMMAND {
        return Err(DecodeError::WrongResponseCommand(bytes[0]));
    }

    let reported_count = bytes[3];
    if usize::from(reported_count) > STAGE_CAPACITY {
        return Err(DecodeError::TooManyStages(reported_count));
    }

    let end = 8 + usize::from(reported_count) * 2;
    if end > bytes.len() {
        return Err(DecodeError::TruncatedStageTable {
            expected: end,
            actual: bytes.len(),
        });
    }

    let stages = (0..usize::from(reported_count))
        .map(|stage| {
            let offset = 8 + stage * 2;
            u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
        })
        .collect();

    let active_stage_index = if bytes[2] > STAGE_CAPACITY as u8 {
        0
    } else {
        usize::from(bytes[2])
    };

    Ok(DpiProfile {
        active_stage_index,
        stages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_openmouse_get_dpi_request_shape() {
        let report = encode_get_dpi_report(1);
        assert_eq!(report.len(), REPORT_LEN);
        assert_eq!(&report[..9], &[0xd4, 1, 0, 0, 0, 0, 0, 0x2a, 0]);
        assert!(report[9..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn decodes_little_endian_stage_values_and_active_index() {
        let mut response = [0u8; REPORT_LEN];
        response[0] = GET_DPI_COMMAND;
        response[2] = 1;
        response[3] = 3;
        response[8..14].copy_from_slice(&[0x20, 0x03, 0x40, 0x06, 0x60, 0x09]);

        let profile = decode_get_dpi_response(&response).unwrap();
        assert_eq!(profile.stages, [800, 1600, 2400]);
        assert_eq!(profile.active_stage_index, 1);
        assert_eq!(profile.active_dpi(), Some(1600));
    }

    #[test]
    fn rejects_wrong_command_and_invalid_stage_count() {
        let mut response = [0u8; REPORT_LEN];
        assert_eq!(
            decode_get_dpi_response(&response),
            Err(DecodeError::WrongResponseCommand(0))
        );

        response[0] = GET_DPI_COMMAND;
        response[3] = 9;
        assert_eq!(
            decode_get_dpi_response(&response),
            Err(DecodeError::TooManyStages(9))
        );
    }

    #[test]
    fn rejects_a_report_with_the_wrong_length() {
        assert_eq!(
            decode_get_dpi_response(&[0; REPORT_LEN - 1]),
            Err(DecodeError::WrongReportLength(REPORT_LEN - 1))
        );
    }
}
