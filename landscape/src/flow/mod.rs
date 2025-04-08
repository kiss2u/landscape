use std::collections::HashMap;

use landscape_common::flow::{FlowConfig, FlowMathPair, PacketMatchMark};

fn convert_mark_map_to_vec_mark(value: HashMap<PacketMatchMark, u32>) -> Vec<FlowMathPair> {
    let mut result = Vec::with_capacity(value.len());
    for (match_rule, flow_id) in value.into_iter() {
        result.push(FlowMathPair { match_rule, flow_id });
    }
    result
}

pub fn update_flow_matchs(rules: Vec<FlowConfig>, old_rules: Vec<FlowConfig>) {
    let mut rules = flow_rule_into_hash(rules);
    let old_rules = flow_rule_into_hash(old_rules);
    tracing::debug!("rules: {:?}", rules);
    tracing::debug!("old_rules: {:?}", old_rules);

    let delete_keys = find_delete_rule_keys(&mut rules, old_rules);
    tracing::debug!("update_config: {:?}", rules);
    tracing::debug!("delete_keys: {:?}", delete_keys);

    landscape_ebpf::map_setting::flow::update_flow_match_rule(convert_mark_map_to_vec_mark(rules));
    landscape_ebpf::map_setting::flow::del_flow_match_rule(delete_keys);
}

fn flow_rule_into_hash(rules: Vec<FlowConfig>) -> HashMap<PacketMatchMark, u32> {
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
    new_rules: &mut HashMap<PacketMatchMark, u32>,
    old_rules: HashMap<PacketMatchMark, u32>,
) -> Vec<PacketMatchMark> {
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
