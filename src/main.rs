use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hidapi::{DeviceInfo, HidApi};
use serde::Serialize;

const X8_SE_VID: u16 = 0x1d57;
const X8_SE_PID: u16 = 0xfa60;
const GEARHUB_VID: u16 = 0x25a7;

#[derive(Parser)]
#[command(name = "AttackSharkDPI", version, about = "Read-only HID probe for hardware validation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List known or related HID identifier candidates.
    Detect,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CandidateClass {
    X8SeReportedId,
    GearhubVendorCandidate,
    Other,
}

#[derive(Debug, Serialize)]
struct DeviceRecord {
    vendor_id: u16,
    product_id: u16,
    interface_number: i32,
    usage_page: u16,
    usage: u16,
    manufacturer: Option<String>,
    product: Option<String>,
    candidate_class: CandidateClass,
    model_confirmed: bool,
}

#[derive(Debug, Serialize)]
struct DetectOutput {
    ok: bool,
    read_only: bool,
    devices: Vec<DeviceRecord>,
    error: Option<String>,
}

fn classify(vid: u16, pid: u16) -> CandidateClass {
    if vid == X8_SE_VID && pid == X8_SE_PID {
        CandidateClass::X8SeReportedId
    } else if vid == GEARHUB_VID {
        CandidateClass::GearhubVendorCandidate
    } else {
        CandidateClass::Other
    }
}

fn record(info: &DeviceInfo) -> Option<DeviceRecord> {
    let candidate_class = classify(info.vendor_id(), info.product_id());
    if candidate_class == CandidateClass::Other {
        return None;
    }
    Some(DeviceRecord {
        vendor_id: info.vendor_id(),
        product_id: info.product_id(),
        interface_number: info.interface_number(),
        usage_page: info.usage_page(),
        usage: info.usage(),
        manufacturer: info.manufacturer_string().map(str::to_owned),
        product: info.product_string().map(str::to_owned),
        candidate_class,
        model_confirmed: false,
    })
}

fn detect() -> Result<DetectOutput> {
    let api = HidApi::new().context("could not initialize Windows HID API")?;
    Ok(DetectOutput {
        ok: true,
        read_only: true,
        devices: api.device_list().filter_map(record).collect(),
        error: None,
    })
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Detect => match detect() {
            Ok(output) => println!("{}", serde_json::to_string_pretty(&output).expect("serialize output")),
            Err(error) => {
                println!("{}", serde_json::to_string_pretty(&DetectOutput {
                    ok: false,
                    read_only: true,
                    devices: Vec::new(),
                    error: Some(format!("{error:#}")),
                }).expect("serialize output"));
                std::process::exit(1);
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reported_x8_se_id_is_only_a_candidate() {
        assert_eq!(classify(0x1d57, 0xfa60), CandidateClass::X8SeReportedId);
    }

    #[test]
    fn gearhub_vendor_is_candidate_without_confirming_model() {
        assert_eq!(classify(0x25a7, 0x0001), CandidateClass::GearhubVendorCandidate);
    }

    #[test]
    fn unrelated_ids_are_excluded() {
        assert_eq!(classify(0x046d, 0xc08b), CandidateClass::Other);
    }
}
