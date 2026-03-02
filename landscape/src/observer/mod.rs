// Re-export from netlink::observer for backward compatibility
pub use crate::netlink::observer::{
    dev_observer, filter_message_status, handle_address_msg, ip_observer,
};

#[cfg(test)]
mod tests {
    use crate::observer::dev_observer;

    #[tokio::test]
    async fn test_dev_observer() {
        landscape_common::init_tracing!();
        let mut info = dev_observer().await;

        while let Ok(msg) = info.recv().await {
            tracing::debug!("msg: {msg:#?}");
        }
    }
}
