<script lang="ts" setup>
import { HelpFilled } from "@vicons/carbon";
</script>
<template>
  <n-popover trigger="hover">
    <template #trigger>
      <n-flex justify="center" align="center">
        <slot></slot>
        <n-button text>
          <template #icon>
            <n-icon><HelpFilled /></n-icon>
          </template>
        </n-button>
      </n-flex>
    </template>
    <slot name="msg"></slot>
  </n-popover>
</template>
