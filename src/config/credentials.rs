use crate::Result;

use std::path::Path;

pub fn copy_credentials(backup: &str) -> Result<()> {
    let org_path = Path::new(&super::config_dir()).join("credentials");
    let backup_path = Path::new(&super::config_dir()).join(backup);
    std::fs::copy(org_path, backup_path)
        .map(drop)
        .map_err(anyhow::Error::new)
}
