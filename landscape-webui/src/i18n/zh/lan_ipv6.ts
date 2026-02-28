export default {
  title: "LAN IPv6 配置",

  enable: "是否启用",
  enabled: "启用",
  disabled: "禁用",

  mode: "模式",
  mode_slaac: "纯 RA (SLAAC)",
  mode_stateful: "纯 DHCPv6 (Stateful)",
  mode_slaac_dhcpv6: "RA + DHCPv6",

  mode_slaac_desc:
    "设备将通过路由通告自动获取 IPv6 地址，无需额外的地址管理服务。适合大多数家庭和小型网络。缺点是部分设备无法固定地址后缀。",
  mode_stateful_desc:
    "由 DHCPv6 服务器统一分配地址，可精确控制每台设备的地址。部分老旧设备可能不支持 DHCPv6，但若无此类设备建议默认使用此模式。",
  mode_slaac_dhcpv6_desc:
    "路由通告提供本地私有地址 (ULA)，保障不支持 DHCPv6 的设备基础上网条件；DHCPv6 尽量分配公网地址 (GUA)。设备访问外网时将自动优先使用公网地址。",

  ra_prefix_source: "RA 前缀配置",
  ra_prefix_source_desc:
    "设置路由通告的地址前缀。支持手动指定静态前缀，或自动使用从上游获取的前缀委派 (PD)。设备将基于这些前缀自动生成地址。",

  dhcpv6_prefix_source: "DHCPv6 前缀源",
  dhcpv6_prefix_source_desc:
    "设置 DHCPv6 服务器可分配的地址范围。支持手动指定静态前缀，或自动使用从上游获取的前缀委派 (PD)。",

  ra_prefix_source_ula: "RA 前缀源（ULA）",
  ra_prefix_source_ula_desc:
    "本地私有前缀 (ULA)，用于保障局域网设备之间的互通。以及基础的上网能力。",
  dhcpv6_prefix_source_combo_desc:
    "DHCPv6 分配的地址前缀，通常为公网地址 (GUA)。设备访问外网时将优先使用此地址。",

  add: "增加",
  no_prefix: "暂无前缀配置",
  no_dhcpv6_prefix: "暂无 DHCPv6 前缀配置",
  no_ra_prefix: "暂无 RA 前缀配置",

  ra_config: "RA 配置",
  ad_interval: "路由通告间隔",
  ad_interval_desc: "路由器定期发送通告的时间间隔（秒），默认 300 秒。",

  m_flag: "通过 DHCPv6 获取地址 (M)",
  m_flag_desc: "开启后，设备将通过 DHCPv6 服务器获取地址，而非自动生成。",
  o_flag: "通过 DHCPv6 获取配置 (O)",
  o_flag_desc: "开启后，设备将通过 DHCPv6 获取 DNS 等网络配置信息。",

  ra_flags_auto: "RA 标志（自动）",
  ra_flags_auto_desc:
    "当前模式已自动启用 M 和 O 标志，引导设备使用 DHCPv6 服务。",

  route_priority: "默认路由优先级",
  priority_low: "低",
  priority_medium: "中（默认）",
  priority_high: "高",

  update: "更新",
  cancel: "取消",
  confirm: "确定",
  form_validation_failed: "请检查配置项是否填写正确",
  cross_source_conflict: "RA 前缀源与 DHCPv6 前缀源的子网索引 {idx} 重复",

  // Source binding modal
  source_edit_title: "前缀来源编辑",
  service_kind_ra: "RA",
  service_kind_na: "DHCPv6 NA",
  service_kind_pd: "DHCPv6 PD",
  source_type_static: "静态前缀",
  source_type_pd: "IPv6 PD",

  source_base_prefix: "基础前缀地址",
  source_base_prefix_cidr: "父前缀 (CIDR)",
  source_base_prefix_hint:
    "注意! 最多只可自定义到 /60, 格式需要保持 ::xxx0, 因为低位 0 不可省略",
  source_depend_iface: "关联网卡（须开启 DHCPv6-PD）",
  source_depend_iface_placeholder: "选择进行前缀申请的网卡",
  source_no_iface: "未选择 PD 网卡",

  source_pool_index: "池块序号",
  source_pool_index_desc:
    "在父前缀内的块序号。RA/NA 对应第几个 /64 子网，PD 对应以池块前缀长度划分的第几块。",
  source_pool_len: "池块前缀长度",
  source_pool_len_desc:
    "池块的前缀长度。例如：父前缀=/48，池块长度=56，则每个池块为 /56，可容纳多个委派子前缀。",
  source_max_source_prefix_len: "最大源前缀长度",
  source_max_source_prefix_len_desc:
    "过滤上游 PD 前缀：只有前缀长度不超过此值的上游前缀才用于委派。",
  source_preferred_lifetime: "首选生存期 (s)",
  source_preferred_lifetime_desc:
    "主机会优先使用在首选时间内的 IP，相对于超过首选时间但未超过有效时间的 IP。",
  source_valid_lifetime: "有效生存期 (s)",

  // DHCPv6 Server Card
  dhcpv6_server: "DHCPv6 服务器",
  ia_na: "IA_NA（地址分配）",
  ia_pd: "IA_PD（前缀委派）",
  enable_ia_na: "启用 IA_NA",
  enable_ia_pd: "启用 IA_PD",

  ia_na_max_prefix_len: "过滤前缀长度",
  ia_na_max_prefix_len_desc:
    "只从前缀长度不超过此值的前缀源中分配地址。例如设为 64，则长度大于 /64 的前缀源会被跳过。",
  ia_na_pool_start: "地址池起始",
  ia_na_pool_start_desc:
    "地址后缀的起始编号。例如设为 256，则分配的第一个地址后缀为 ::100。",
  ia_na_preferred_lifetime: "首选生存期",
  ia_na_preferred_lifetime_desc:
    "设备优先使用该地址的时长（秒）。到期后地址仍可用但不再优先选择。",
  ia_na_valid_lifetime: "有效生存期",
  ia_na_valid_lifetime_desc:
    "地址的最长有效时间（秒）。到期后设备必须重新申请地址。应大于首选生存期。",

  ia_pd_delegate_prefix_len: "委派前缀长度",
  ia_pd_delegate_prefix_len_desc:
    "分配给每台设备的子前缀长度。例如源前缀为 /48、委派长度为 /64，则可委派 2^16 个 /64 子前缀。",
  ia_pd_preferred_lifetime: "首选生存期",
  ia_pd_preferred_lifetime_desc:
    "设备优先使用该委派前缀的时长（秒）。到期后前缀仍可用但不再优先选择。",
  ia_pd_valid_lifetime: "有效生存期",
  ia_pd_valid_lifetime_desc:
    "委派前缀的最长有效时间（秒）。到期后设备必须重新申请。应大于首选生存期。",
};
