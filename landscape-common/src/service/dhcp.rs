use crate::dhcp::DHCPv4OfferInfo;
use crate::service::ServiceStatus;
use crate::service::Watchable;
use serde::Serialize;

use super::service_code::WatchService;

pub type DHCPv4ServiceWatchStatus = WatchService<DHCPv4ServiceStatus>;

#[derive(Serialize, Debug, Clone, Default)]
pub struct DHCPv4ServiceStatus {
    pub status: ServiceStatus,
    pub data: DHCPv4OfferInfo,
}

impl Watchable for DHCPv4ServiceStatus {
    type HoldData = DHCPv4OfferInfo;
    fn get_current_status_code(&self) -> ServiceStatus {
        self.status.clone()
    }

    fn modify_curent_status(&mut self, status: ServiceStatus) {
        self.status = status;
    }

    fn change_status(&mut self, new_status: ServiceStatus, data: Option<DHCPv4OfferInfo>) -> bool {
        let _ = data;
        if self.status.can_transition_to(&new_status) {
            tracing::debug!("change to new status: {new_status:?}");
            self.status = new_status;
        }
        true
    }
}
