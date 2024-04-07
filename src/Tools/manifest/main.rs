use pretty::pass;
use shared::{ais_data::AisInfo, errors::UnifiedError};
use system::{create_hash, truncate};

fn main() -> Result<(), UnifiedError> {
    // Create an instance of AisInfo
    let mut ais_info: AisInfo = AisInfo::new()?;

    ais_info.machine_id = Some(
        truncate(
            &create_hash(format!(
                "{}{}",
                &ais_info
                    .clone()
                    .machine_ip
                    .unwrap_or(String::from("Uninitialized")),
                &ais_info
                    .clone()
                    .machine_id
                    .unwrap_or(String::from("Uninitialized"))
            )),
            16,
        )
        .to_owned(),
    );
    ais_info.system_version = AisInfo::current_version();
    // Generate the manifest file
    ais_info.create_manifest()?;

    pass("Manifest file created successfully");

    Ok(())
}
