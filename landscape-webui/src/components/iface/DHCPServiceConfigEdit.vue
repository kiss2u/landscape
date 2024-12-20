<script setup lang="ts">
import { DhcpServerConfig } from "@/lib/dhcp";

const dhcp_config = defineModel<DhcpServerConfig>("dhcp_config", {
  required: true,
});
</script>
<template></template>
