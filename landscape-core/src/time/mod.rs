mod sync;

pub use sync::{
    get_time_sync_status, set_system_time, start_ntp_sync_thread, start_time_sync_service,
    update_time_sync_config,
};
