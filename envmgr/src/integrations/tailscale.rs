use crate::error::EnvMgrResult;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, Default)]
pub struct TailscaleConfig {
    pub tailnet: String,
}

pub struct Tailscale;

struct TailscaleSwitchListItem {
    pub _id: String,
    pub tailnet: String,
    pub _account: String,
    pub active: bool,
}

impl Tailscale {
    fn tailscale_switch_list() -> EnvMgrResult<Vec<TailscaleSwitchListItem>> {
        let output = std::process::Command::new("tailscale")
            .arg("switch")
            .arg("--list")
            .output()?;
        if !output.status.success() {
            return Err(crate::error::EnvMgrError::Other(
                format!(
                    "tailscale switch --list failed with status: {}",
                    output.status
                )
                .into(),
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = vec![];
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                items.push(TailscaleSwitchListItem {
                    _id: parts[0].to_string(),
                    tailnet: parts[1].to_string(),
                    _account: parts[2].trim_end_matches("*").to_string(),
                    active: parts[2].ends_with("*"),
                });
            }
        }
        Ok(items)
    }

    fn switch_to_tailnet(tailnet: &str) -> EnvMgrResult<()> {
        let status = std::process::Command::new("tailscale")
            .arg("switch")
            .arg(tailnet)
            .status()?;
        if !status.success() {
            return Err(crate::error::EnvMgrError::Other(
                format!(
                    "tailscale switch {} failed with status: {}",
                    tailnet, status
                )
                .into(),
            ));
        }
        Ok(())
    }

    pub fn on_switch_to(config: &TailscaleConfig) -> EnvMgrResult<()> {
        let items = Self::tailscale_switch_list()?;
        if let Some(item) = items.iter().find(|item| item.tailnet == config.tailnet) {
            if item.active {
                // Already on the desired tailnet
                return Ok(());
            } else {
                Self::switch_to_tailnet(&item.tailnet)?;
                return Ok(());
            }
        }
        Err(crate::error::EnvMgrError::Other(
            format!(
                "Tailnet '{}' not found in tailscale switch list",
                config.tailnet
            )
            .into(),
        ))
    }
}
