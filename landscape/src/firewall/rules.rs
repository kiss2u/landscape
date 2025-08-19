use std::{collections::HashMap, str::FromStr};

use landscape_common::{
    firewall::{FirewallRuleConfig, FirewallRuleItem, FirewallRuleMark},
    flow::mark::FlowMark,
    utils::range::NumberRange,
};

fn convert_mark_map_to_vec_mark(
    value: HashMap<FirewallRuleItem, FlowMark>,
) -> Vec<FirewallRuleMark> {
    let mut result = Vec::with_capacity(value.len());
    for (item, mark) in value.into_iter() {
        result.push(FirewallRuleMark { mark, item });
    }
    result
}

pub fn update_firewall_rules(
    mut rules: Vec<FirewallRuleConfig>,
    mut old_rules: Vec<FirewallRuleConfig>,
) {
    rules.sort_by(|a, b| a.index.cmp(&b.index));
    old_rules.sort_by(|a, b| a.index.cmp(&b.index));

    let mut rules = firewall_rule_into_hash(rules);
    let old_rules = firewall_rule_into_hash(old_rules);
    tracing::debug!("rules: {:?}", rules);
    tracing::debug!("old_rules: {:?}", old_rules);

    let delete_keys = find_delete_rule_keys(&mut rules, old_rules);
    tracing::debug!("update_config: {:?}", rules);
    tracing::debug!("delete_keys: {:?}", delete_keys);

    landscape_ebpf::map_setting::add_firewall_rule(convert_mark_map_to_vec_mark(rules));
    landscape_ebpf::map_setting::del_firewall_rule(delete_keys);
}

fn firewall_rule_into_hash(rules: Vec<FirewallRuleConfig>) -> HashMap<FirewallRuleItem, FlowMark> {
    let mut new_mark_infos = HashMap::new();

    for ip_rule in rules.into_iter() {
        if !ip_rule.enable {
            continue;
        }
        for item in ip_rule.items.into_iter() {
            if let Some(port_str) = item.local_port {
                let range = NumberRange::from_str(&port_str).unwrap();
                if range.start <= range.end {
                    for insert_port in range.start..=range.end {
                        new_mark_infos.insert(
                            FirewallRuleItem {
                                ip_protocol: item.ip_protocol.clone(),
                                local_port: Some(insert_port),
                                address: item.address,
                                ip_prefixlen: item.ip_prefixlen,
                            },
                            ip_rule.mark,
                        );
                    }
                } else {
                    tracing::error!("port range error: {port_str:?}");
                }
            } else {
                new_mark_infos.insert(
                    FirewallRuleItem {
                        ip_protocol: item.ip_protocol,
                        local_port: None,
                        address: item.address,
                        ip_prefixlen: item.ip_prefixlen,
                    },
                    ip_rule.mark,
                );
            }
        }
    }
    new_mark_infos
}

fn find_delete_rule_keys(
    new_rules: &mut HashMap<FirewallRuleItem, FlowMark>,
    old_rules: HashMap<FirewallRuleItem, FlowMark>,
) -> Vec<FirewallRuleItem> {
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
