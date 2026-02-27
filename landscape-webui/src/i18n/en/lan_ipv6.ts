export default {
  title: "LAN IPv6 Configuration",

  enable: "Enable",
  enabled: "Enabled",
  disabled: "Disabled",

  mode: "Mode",
  mode_slaac: "RA Only (SLAAC)",
  mode_stateful: "DHCPv6 Only (Stateful)",
  mode_slaac_dhcpv6: "RA + DHCPv6",

  mode_slaac_desc:
    "Devices automatically obtain IPv6 addresses via Router Advertisement. No additional address management needed. Suitable for most home and small networks. Downside: some devices cannot have a fixed address suffix.",
  mode_stateful_desc:
    "The DHCPv6 server centrally assigns addresses, giving you precise control over each device's address. Router Advertisement only guides devices to contact the DHCPv6 server. Some older devices may not support DHCPv6, but if you have no such devices this is the recommended mode.",
  mode_slaac_dhcpv6_desc:
    "Router Advertisement provides local private addresses (ULA) to ensure basic connectivity for devices that don't support DHCPv6. DHCPv6 assigns public addresses (GUA) when possible. Devices automatically prefer public addresses for internet access.",

  ra_prefix_source: "RA Prefix Configuration",
  ra_prefix_source_desc:
    "Configure the address prefixes announced by Router Advertisement. You can specify static prefixes manually or use prefixes obtained from upstream Prefix Delegation (PD). Devices will generate addresses based on these prefixes.",

  dhcpv6_prefix_source: "DHCPv6 Prefix Source",
  dhcpv6_prefix_source_desc:
    "Configure the address ranges the DHCPv6 server can assign. You can specify static prefixes manually or use prefixes obtained from upstream Prefix Delegation (PD).",

  ra_prefix_source_ula: "RA Prefix Source (ULA)",
  ra_prefix_source_ula_desc:
    "Local private prefixes (ULA) that ensure connectivity between LAN devices and provide basic internet access.",
  dhcpv6_prefix_source_combo_desc:
    "Prefixes assigned by DHCPv6, typically public addresses (GUA). Devices will prefer these addresses for internet access.",

  add: "Add",
  no_prefix: "No prefixes configured",
  no_dhcpv6_prefix: "No DHCPv6 prefixes configured",
  no_ra_prefix: "No RA prefixes configured",

  ra_config: "RA Configuration",
  ad_interval: "Advertisement Interval",
  ad_interval_desc:
    "How often the router sends periodic advertisements (in seconds). Default is 300 seconds.",

  m_flag: "Obtain Address via DHCPv6 (M)",
  m_flag_desc:
    "When enabled, devices will obtain addresses from the DHCPv6 server instead of generating them automatically.",
  o_flag: "Obtain Config via DHCPv6 (O)",
  o_flag_desc:
    "When enabled, devices will obtain DNS and other network settings via DHCPv6.",

  ra_flags_auto: "RA Flags (Auto)",
  ra_flags_auto_desc:
    "M and O flags are automatically enabled in this mode to direct devices to the DHCPv6 server.",

  route_priority: "Default Route Priority",
  priority_low: "Low",
  priority_medium: "Medium (Default)",
  priority_high: "High",

  update: "Update",
  form_validation_failed: "Please check the configuration fields",
  cross_source_conflict:
    "Subnet index {idx} conflicts between RA and DHCPv6 prefix sources",

  // DHCPv6 Server Card
  dhcpv6_server: "DHCPv6 Server",
  ia_na: "IA_NA (Address Assignment)",
  ia_pd: "IA_PD (Prefix Delegation)",
  enable_ia_na: "Enable IA_NA",
  enable_ia_pd: "Enable IA_PD",

  ia_na_max_prefix_len: "Filter Prefix Length",
  ia_na_max_prefix_len_desc:
    "Only assign addresses from prefix sources with a prefix length no greater than this value. For example, set to 64 to skip any prefix source longer than /64.",
  ia_na_pool_start: "Address Pool Start",
  ia_na_pool_start_desc:
    "The starting number for address suffixes. For example, set to 256 and the first assigned address suffix will be ::100.",
  ia_na_preferred_lifetime: "Preferred Lifetime",
  ia_na_preferred_lifetime_desc:
    "How long (in seconds) the device will prefer using this address. After expiry the address is still valid but no longer preferred.",
  ia_na_valid_lifetime: "Valid Lifetime",
  ia_na_valid_lifetime_desc:
    "Maximum time (in seconds) the address remains valid. After expiry the device must request a new address. Should be greater than the preferred lifetime.",

  ia_pd_max_source_prefix_len: "Filter Source Prefix Length",
  ia_pd_max_source_prefix_len_desc:
    "Only delegate sub-prefixes from sources with a prefix length no greater than this value. For example, set to 56 to skip any source prefix longer than /56.",
  ia_pd_delegate_prefix_len: "Delegated Prefix Length",
  ia_pd_delegate_prefix_len_desc:
    "The prefix length assigned to each device. For example, with a /48 source prefix and /64 delegation length, up to 2^16 sub-prefixes can be delegated.",
  ia_pd_pool_start_index: "Sub-prefix Pool Start",
  ia_pd_pool_start_index_desc:
    "Starting index for sub-prefix allocation. Use this to skip the first few sub-prefixes, reserving them for static assignments or other purposes.",
  ia_pd_preferred_lifetime: "Preferred Lifetime",
  ia_pd_preferred_lifetime_desc:
    "How long (in seconds) the device will prefer using this delegated prefix. After expiry the prefix is still valid but no longer preferred.",
};
