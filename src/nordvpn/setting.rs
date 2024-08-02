pub struct NordVpnBooleanSetting {
    key: str,
    nordvpn: NordVPN,
}

impl NordVpnBooleanSetting {
    pub fn new(key: str, nordvpn: NordVPN) -> Self {
        Self {
            key,
            nordvpn,
        }
    }

    pub fn is_enabled(&self) -> bool {
        return self.enabled;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}