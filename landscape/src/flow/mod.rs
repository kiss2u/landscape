use std::collections::HashMap;

use landscape_common::flow::{config::FlowConfig, FlowEbpfMatchPair, FlowEntryRule};

fn convert_mark_map_to_vec_mark(value: HashMap<FlowEntryRule, u32>) -> Vec<FlowEbpfMatchPair> {
    let mut result = Vec::with_capacity(value.len());
    for (match_rule, flow_id) in value.into_iter() {
        result.push(FlowEbpfMatchPair::new(match_rule, flow_id));
    }
    result
}

pub async fn update_flow_matchs(rules: Vec<FlowConfig>, old_rules: Vec<FlowConfig>) {
    // let net_ifindexs: Vec<(u32, String)> = rules
    //     .iter()
    //     .map(|e| (e.flow_id, e.packet_handle_iface_name.get(0)))
    //     .filter_map(|e| match e.1 {
    //         Some(FlowTarget::Interface { name }) => Some((e.0, name.clone())),
    //         _ => None,
    //     })
    //     .collect();
    let mut rules = flow_rule_into_hash(rules);
    let old_rules = flow_rule_into_hash(old_rules);
    tracing::debug!("rules: {:?}", rules);
    tracing::debug!("old_rules: {:?}", old_rules);

    let delete_keys = find_delete_rule_keys(&mut rules, old_rules);
    tracing::debug!("update_config: {:?}", rules);
    tracing::debug!("delete_keys: {:?}", delete_keys);

    landscape_ebpf::map_setting::flow::update_flow_match_rule(convert_mark_map_to_vec_mark(rules));
    landscape_ebpf::map_setting::flow::del_flow_match_rule(delete_keys);

    // TODO: 应当只清理当前 Flow 的缓存
    landscape_ebpf::map_setting::route::cache::recreate_route_lan_cache_inner_map();
}

fn flow_rule_into_hash(rules: Vec<FlowConfig>) -> HashMap<FlowEntryRule, u32> {
    let mut new_mark_infos = HashMap::new();

    for ip_rule in rules.into_iter() {
        if !ip_rule.enable {
            continue;
        }
        for item in ip_rule.flow_match_rules.into_iter() {
            new_mark_infos.insert(item, ip_rule.flow_id);
        }
    }
    new_mark_infos
}

fn find_delete_rule_keys(
    new_rules: &mut HashMap<FlowEntryRule, u32>,
    old_rules: HashMap<FlowEntryRule, u32>,
) -> Vec<FlowEntryRule> {
    let mut delete_keys = vec![];
    for (key, old_mark) in old_rules.into_iter() {
        if let Some(mark) = new_rules.get(&key) {
            if *mark == old_mark {
                new_rules.remove(&key);
            } else {
                continue;
            }
        } else {
            delete_keys.push(key);
        }
    }
    delete_keys
}
