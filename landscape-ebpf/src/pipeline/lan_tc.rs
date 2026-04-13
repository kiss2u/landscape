use std::{
    collections::HashMap,
    os::fd::{AsFd, AsRawFd},
    path::PathBuf,
    sync::{Arc, Mutex, Weak},
};

use libbpf_rs::{
    skel::{OpenSkel, SkelBuilder},
    MapCore, MapFlags, Program, TC_EGRESS, TC_INGRESS,
};
use once_cell::sync::Lazy;

use crate::{
    bpf_error::LdEbpfResult,
    landscape::{OwnedOpenObject, TcHookProxy},
    map_setting::reuse_pinned_map_or_recreate,
    LAN_ROUTE_EGRESS_PRIORITY, LAN_ROUTE_INGRESS_PRIORITY, MAP_PATHS,
};

pub(crate) mod lan_tc_pipeline {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/lan_tc_pipeline.skel.rs"));
}

use lan_tc_pipeline::*;

pub(crate) const LAN_INGRESS_STAGE_ROUTE: u32 = 0;
pub(crate) const LAN_INGRESS_STAGE_COUNT: u32 = 1;
pub(crate) const LAN_EGRESS_STAGE_ROUTE: u32 = 0;
pub(crate) const LAN_EGRESS_STAGE_COUNT: u32 = 1;

static PIPELINES: Lazy<Mutex<HashMap<u32, Weak<LanTcPipelineInner>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub(crate) fn lan_tc_pipeline_ingress_path(ifindex: u32) -> PathBuf {
    MAP_PATHS.lan_tc_pipeline_ingress.with_file_name(format!("lan_tc_pipeline_ingress_{ifindex}"))
}

pub(crate) fn lan_tc_pipeline_egress_path(ifindex: u32) -> PathBuf {
    MAP_PATHS.lan_tc_pipeline_egress.with_file_name(format!("lan_tc_pipeline_egress_{ifindex}"))
}

pub struct LanTcPipelineHandle {
    inner: Arc<LanTcPipelineInner>,
}

struct LanTcPipelineInner {
    _backing: OwnedOpenObject,
    skel: LanTcPipelineSkel<'static>,
    ingress_hook: Mutex<Option<TcHookProxy>>,
    egress_hook: Mutex<Option<TcHookProxy>>,
    ifindex: u32,
}

unsafe impl Send for LanTcPipelineHandle {}
unsafe impl Sync for LanTcPipelineHandle {}
unsafe impl Send for LanTcPipelineInner {}
unsafe impl Sync for LanTcPipelineInner {}

impl LanTcPipelineHandle {
    pub fn acquire(ifindex: u32) -> LdEbpfResult<Self> {
        let mut pipelines = PIPELINES.lock().expect("lan tc pipeline registry poisoned");
        if let Some(existing) = pipelines.get(&ifindex).and_then(Weak::upgrade) {
            return Ok(Self { inner: existing });
        }

        let inner = Arc::new(LanTcPipelineInner::new(ifindex)?);
        pipelines.insert(ifindex, Arc::downgrade(&inner));
        Ok(Self { inner })
    }

    pub fn register_route_lan(
        &self,
        ingress_prog: &Program,
        egress_prog: &Program,
    ) -> LdEbpfResult<()> {
        register_stage(
            &self.inner.skel.maps.lan_ingress_stage_progs,
            LAN_INGRESS_STAGE_ROUTE,
            ingress_prog,
        )?;
        register_stage(
            &self.inner.skel.maps.lan_egress_stage_progs,
            LAN_EGRESS_STAGE_ROUTE,
            egress_prog,
        )?;
        Ok(())
    }

    pub fn unregister_route_lan(&self) {
        let _ = self
            .inner
            .skel
            .maps
            .lan_ingress_stage_progs
            .delete(&LAN_INGRESS_STAGE_ROUTE.to_ne_bytes());
        let _ = self
            .inner
            .skel
            .maps
            .lan_egress_stage_progs
            .delete(&LAN_EGRESS_STAGE_ROUTE.to_ne_bytes());
    }
}

impl LanTcPipelineInner {
    fn new(ifindex: u32) -> LdEbpfResult<Self> {
        let builder = LanTcPipelineSkelBuilder::default();
        let (backing, open_object) = OwnedOpenObject::new();
        let mut open_skel =
            crate::bpf_ctx!(builder.open(open_object), "lan_tc_pipeline open skeleton failed")?;
        let ingress_path = lan_tc_pipeline_ingress_path(ifindex);
        let egress_path = lan_tc_pipeline_egress_path(ifindex);
        reuse_pinned_map_or_recreate(&mut open_skel.maps.lan_ingress_stage_progs, &ingress_path);
        reuse_pinned_map_or_recreate(&mut open_skel.maps.lan_egress_stage_progs, &egress_path);
        let skel = crate::bpf_ctx!(open_skel.load(), "lan_tc_pipeline load skeleton failed")?;
        clear_stage_slots(&skel.maps.lan_ingress_stage_progs, LAN_INGRESS_STAGE_COUNT);
        clear_stage_slots(&skel.maps.lan_egress_stage_progs, LAN_EGRESS_STAGE_COUNT);

        let mut ingress_hook = TcHookProxy::new(
            &skel.progs.lan_tc_pipeline_ingress_root,
            ifindex as i32,
            TC_INGRESS,
            LAN_ROUTE_INGRESS_PRIORITY,
        );
        let mut egress_hook = TcHookProxy::new(
            &skel.progs.lan_tc_pipeline_egress_root,
            ifindex as i32,
            TC_EGRESS,
            LAN_ROUTE_EGRESS_PRIORITY,
        );

        ingress_hook.attach();
        egress_hook.attach();

        Ok(Self {
            _backing: backing,
            skel,
            ingress_hook: Mutex::new(Some(ingress_hook)),
            egress_hook: Mutex::new(Some(egress_hook)),
            ifindex,
        })
    }
}

impl Drop for LanTcPipelineInner {
    fn drop(&mut self) {
        let mut pipelines = PIPELINES.lock().expect("lan tc pipeline registry poisoned");
        pipelines.remove(&self.ifindex);
        self.ingress_hook.lock().expect("lan tc pipeline ingress hook poisoned").take();
        self.egress_hook.lock().expect("lan tc pipeline egress hook poisoned").take();
    }
}

fn register_stage<M: MapCore>(map: &M, slot: u32, prog: &Program) -> LdEbpfResult<()> {
    let prog_fd = prog.as_fd().as_raw_fd();
    crate::bpf_ctx!(
        map.update(&slot.to_ne_bytes(), &prog_fd.to_ne_bytes(), MapFlags::ANY),
        "lan_tc_pipeline register stage {slot} failed"
    )?;
    Ok(())
}

fn clear_stage_slots<M: MapCore>(map: &M, count: u32) {
    for slot in 0..count {
        let _ = map.delete(&slot.to_ne_bytes());
    }
}
