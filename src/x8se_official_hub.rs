//! Static reconstruction of the X8 SE DPI block built by ATTACK SHARK HUB.
//!
//! This module only encodes bytes in memory. It does not implement the
//! browser's local-helper transport and must not be used to write to a mouse
//! until a real X8 SE capture verifies the HID framing and device behavior.

use std::fmt;

pub const DPI_BLOCK_LEN: usize = 60;
pub const STAGE_COUNT: usize = 8;
const DPI3311: [u8; 220] = [
    1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 21, 22, 23, 24, 25, 27, 28, 29, 30,
    31, 32, 34, 35, 36, 37, 38, 39, 41, 42, 43, 44, 45, 47, 48, 49, 50, 51, 52, 54, 55, 56, 57, 58,
    59, 61, 62, 63, 64, 65, 67, 68, 69, 70, 71, 72, 74, 75, 76, 77, 78, 79, 81, 82, 83, 84, 85, 87,
    88, 89, 90, 91, 92, 94, 95, 96, 97, 98, 99, 101, 102, 103, 104, 105, 107, 108, 109, 110, 111,
    112, 114, 115, 116, 117, 118, 119, 121, 122, 123, 124, 125, 127, 128, 129, 130, 131, 132, 134,
    135, 136, 137, 138, 139, 141, 142, 143, 144, 145, 147, 148, 149, 150, 151, 152, 154, 155, 156,
    157, 158, 159, 161, 162, 163, 164, 165, 167, 168, 169, 170, 171, 172, 174, 175, 176, 177, 178,
    179, 181, 182, 183, 184, 185, 187, 188, 189, 190, 191, 192, 194, 195, 196, 197, 198, 199, 201,
    202, 203, 204, 205, 207, 208, 209, 210, 211, 212, 214, 215, 216, 217, 218, 219, 221, 222, 223,
    224, 225, 227, 228, 229, 230, 231, 232, 234, 235, 118, 119, 121, 122, 123, 124, 125, 127, 128,
    129, 130, 131, 132, 134, 135, 136, 137, 138, 139, 141,
];

const DEFAULT_INDICATORS: [[u8; 3]; STAGE_COUNT] = [
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255],
    [255, 255, 0],
    [0, 255, 255],
    [255, 0, 255],
    [255, 64, 0],
    [255, 255, 255],
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct X8SeDpiSettings {
    pub profile_id: u8,
    pub sensitivity_x: u8,
    pub sensitivity_y: u8,
    pub enabled_stages: u8,
    pub stages: [Option<u16>; STAGE_COUNT],
    pub active_stage: u8,
    pub indicators: [[u8; 3]; STAGE_COUNT],
    pub indicator_type: u8,
}

impl Default for X8SeDpiSettings {
    fn default() -> Self {
        Self {
            profile_id: 1,
            sensitivity_x: 0,
            sensitivity_y: 1,
            enabled_stages: 63,
            stages: [
                Some(800),
                Some(1_600),
                Some(2_400),
                Some(3_200),
                Some(8_000),
                Some(25_000),
                Some(0),
                Some(0),
            ],
            active_stage: 2,
            indicators: DEFAULT_INDICATORS,
            indicator_type: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncodeError {
    ActiveStageOutOfRange(u8),
    DpiOutsideObservedUiRange { stage: usize, dpi: u16 },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActiveStageOutOfRange(stage) => {
                write!(f, "active stage must be in 1..=8, got {stage}")
            }
            Self::DpiOutsideObservedUiRange { stage, dpi } => write!(
                f,
                "stage {} DPI {dpi} is outside the ATTACK SHARK HUB UI steps",
                stage + 1
            ),
        }
    }
}

impl std::error::Error for EncodeError {}

/// Encode one in-memory 60-byte settings block from the official hub's
/// observed X8 SE serializer.
///
/// `stages` follows the hub's one-based DPI slots. `None` and `Some(0)` both
/// encode a disabled slot. `active_stage` is one-based, matching the hub UI.
/// Accepted values mirror the displayed UI increments: 50..=10,000 by 50,
/// then 10,100..=25,000 by 100.
pub fn encode_settings_block(
    settings: &X8SeDpiSettings,
) -> Result<[u8; DPI_BLOCK_LEN], EncodeError> {
    if !(1..=STAGE_COUNT as u8).contains(&settings.active_stage) {
        return Err(EncodeError::ActiveStageOutOfRange(settings.active_stage));
    }

    let mut x_slots = [0u8; STAGE_COUNT];
    let mut y_slots = [0u8; STAGE_COUNT];
    let mut x_double = 0u8;
    let mut y_double = 0u8;

    for (stage, dpi) in settings.stages.iter().enumerate() {
        let Some(dpi) = *dpi else {
            continue;
        };
        if dpi == 0 {
            continue;
        }
        validate_ui_dpi(stage, dpi)?;
        let (x, y) = encode_dpi_value(dpi);
        x_slots[stage] = x;
        y_slots[stage] = y;

        if (24_100..=25_000).contains(&dpi) {
            y_double |= 1 << stage;
        } else if (12_001..=24_000).contains(&dpi) {
            x_double |= 1 << stage;
        }
    }

    let mut block = [0u8; DPI_BLOCK_LEN];
    block[0..8].copy_from_slice(&[
        4,  // fixed
        56, // sizeTotalData
        settings.profile_id,
        settings.sensitivity_x,
        settings.sensitivity_y,
        settings.enabled_stages,
        x_double,
        y_double,
    ]);
    block[8..16].copy_from_slice(&x_slots);
    block[16..24].copy_from_slice(&y_slots);
    block[24] = settings.active_stage;

    // Reproduce the deployed bundle's first indicator-loop indexing as
    // written (`n+++d`): it leaves a gap and the next loop overwrites byte 29.
    // Keep this quirk visible for capture comparison instead of silently
    // normalizing the vendor's observed output.
    let mut indicator_offset = 25usize;
    for (component, value) in settings.indicators[0].iter().copied().enumerate() {
        block[indicator_offset + component] = value;
        indicator_offset += 1;
    }
    for color in settings.indicators.iter().skip(1) {
        for value in color {
            block[indicator_offset] = *value;
            indicator_offset += 1;
        }
    }
    block[49] = settings.indicator_type;

    let checksum = block[3..50]
        .iter()
        .fold(0u16, |sum, byte| sum.wrapping_add(u16::from(*byte)));
    block[50] = (checksum >> 8) as u8;
    block[51] = checksum as u8;
    // Bytes 52..60 remain zero, as in a newly allocated ArrayBuffer(60).
    Ok(block)
}

fn validate_ui_dpi(stage: usize, dpi: u16) -> Result<(), EncodeError> {
    let valid = (50..=10_000).contains(&dpi) && dpi % 50 == 0
        || (10_100..=25_000).contains(&dpi) && dpi % 100 == 0;
    if valid {
        Ok(())
    } else {
        Err(EncodeError::DpiOutsideObservedUiRange { stage, dpi })
    }
}

fn encode_dpi_value(dpi: u16) -> (u8, u8) {
    if dpi == 0 {
        return (0, 0);
    }

    let original = u32::from(dpi);
    let mut transformed = if (24_100..=25_000).contains(&dpi) {
        (f64::from(original) / 3.0).round() as u32
    } else if dpi > 24_000 {
        8_500
    } else if dpi > 12_000 {
        (f64::from(original) / 2.0).round() as u32
    } else {
        original
    };

    if transformed <= 10_000 {
        transformed = transformed.max(50);
        transformed = ((transformed + 25) / 50) * 50;
        transformed = transformed.min(10_000);
    } else {
        transformed = ((transformed + 50) / 100) * 100;
        transformed = transformed.max(10_100);
    }

    let (index, y_marker) = if transformed <= 10_000 {
        ((transformed - 50) / 50, 0)
    } else {
        (199 + (transformed - 10_000) / 100, 1)
    };
    (DPI3311[index as usize], y_marker)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_official_hub_default_stage_values() {
        let block = encode_settings_block(&X8SeDpiSettings::default()).unwrap();

        // Synthetic golden vector generated by the deployed JavaScript
        // serializer from the same default stage values; not a HID capture.
        const EXPECTED: [u8; DPI_BLOCK_LEN] = [
            0x04, 0x38, 0x01, 0x00, 0x01, 0x3f, 0x00, 0x20, 0x12, 0x25, 0x38, 0x4b, 0xbc, 0xc4,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0x00, 0x00,
            0x00, 0xff, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00,
            0xff, 0xff, 0x40, 0x00, 0xff, 0xff, 0xff, 0x02, 0x0f, 0xd1, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(block, EXPECTED);
        assert_eq!(&block[0..8], &[4, 56, 1, 0, 1, 63, 0, 32]);
        assert_eq!(&block[8..16], &[18, 37, 56, 75, 188, 196, 0, 0]);
        assert_eq!(&block[16..24], &[0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(block[24], 2);
        assert_eq!(&block[52..], &[0; 8]);
        assert_eq!(u16::from_be_bytes([block[50], block[51]]), checksum(&block));
    }

    #[test]
    fn mirrors_vendor_indicator_loop_bytes() {
        let settings = X8SeDpiSettings {
            stages: [None; STAGE_COUNT],
            active_stage: 1,
            ..X8SeDpiSettings::default()
        };
        let block = encode_settings_block(&settings).unwrap();
        assert_eq!(&block[25..31], &[255, 0, 0, 0, 255, 0]);
        assert_eq!(&block[31..34], &[0, 0, 255]);
    }

    #[test]
    fn rejects_non_ui_steps_and_invalid_stage_ids() {
        let mut settings = X8SeDpiSettings {
            stages: [None; STAGE_COUNT],
            active_stage: 1,
            ..X8SeDpiSettings::default()
        };
        settings.stages[0] = Some(12_050);
        assert!(matches!(
            encode_settings_block(&settings),
            Err(EncodeError::DpiOutsideObservedUiRange { .. })
        ));
        let mut settings = settings;
        settings.stages[0] = None;
        settings.active_stage = 0;
        assert!(matches!(
            encode_settings_block(&settings),
            Err(EncodeError::ActiveStageOutOfRange(0))
        ));
    }

    #[test]
    fn flags_high_ranges_per_stage() {
        let stages = [
            Some(12_100),
            Some(24_000),
            Some(24_100),
            Some(25_000),
            None,
            None,
            None,
            None,
        ];
        let settings = X8SeDpiSettings {
            stages,
            active_stage: 1,
            ..X8SeDpiSettings::default()
        };
        let block = encode_settings_block(&settings).unwrap();
        assert_eq!(block[6], 0b0000_0011);
        assert_eq!(block[7], 0b0000_1100);
    }

    fn checksum(block: &[u8; DPI_BLOCK_LEN]) -> u16 {
        block[3..50]
            .iter()
            .fold(0u16, |sum, byte| sum.wrapping_add(u16::from(*byte)))
    }
}

