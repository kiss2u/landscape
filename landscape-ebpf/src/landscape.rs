use std::os::fd::AsFd;

use libbpf_rs::{Program, TcAttachPoint, TcHook, TcHookBuilder};

pub struct TcHookProxy {
    hook: Option<TcHook>,
}

impl TcHookProxy {
    pub fn new(prog: &Program, ifindex: i32, attach: TcAttachPoint, priority: u32) -> TcHookProxy {
        let mut tc_builder = TcHookBuilder::new(prog.as_fd());
        tc_builder.ifindex(ifindex).replace(true).handle(1).priority(priority);
        let ingress = tc_builder.hook(attach);
        Self { hook: Some(ingress) }
    }

    pub fn attach(&mut self) {
        if let Some(hook) = self.hook.as_mut() {
            let result = hook.query();
            tracing::debug!("1 - the hook is exist? {:?}", result);
            hook.create().unwrap();

            let result = hook.query();
            tracing::debug!("2 - the hook is exist? {:?}", result);
            hook.attach().unwrap();

            let result = hook.query();
            tracing::debug!("3 - the hook is exist? {:?}", result);
        }
    }
}

impl Drop for TcHookProxy {
    fn drop(&mut self) {
        if let Some(mut hook) = self.hook {
            tracing::debug!("detach hook");
            if let Ok(_) = hook.query() {
                tracing::debug!("start detach success");
                if let Err(e) = hook.detach() {
                    tracing::debug!("detach error: {:?}", e);
                } else {
                    tracing::debug!("detach success");
                }
            }
            // if let Err(e) = hook.destroy() {
            //     tracing::debug!("destroy error: {:?}", e);
            // }
        }
        // if let Ok(_) = self.query() {
        //     self.detach().unwrap();
        // }
    }
}
