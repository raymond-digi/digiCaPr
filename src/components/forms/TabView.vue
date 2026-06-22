<template>
  <v-card flat class="tab-card">
    <v-tabs v-model="activeTab" color="primary" grow class="tab-nav">
      <v-tab value="personal">
        <v-icon v-if="tabStatus?.personal === 'valid'" size="small" color="success" class="mr-1">mdi-check-circle</v-icon>
        <v-icon v-else-if="tabStatus?.personal === 'invalid'" size="small" color="error" class="mr-1">mdi-alert-circle</v-icon>
        Personal Info
      </v-tab>
      <v-tab value="employment">
        <v-icon v-if="tabStatus?.employment === 'valid'" size="small" color="success" class="mr-1">mdi-check-circle</v-icon>
        <v-icon v-else-if="tabStatus?.employment === 'invalid'" size="small" color="error" class="mr-1">mdi-alert-circle</v-icon>
        Employment
      </v-tab>
      <v-tab value="payroll">
        <v-icon v-if="tabStatus?.payroll === 'valid'" size="small" color="success" class="mr-1">mdi-check-circle</v-icon>
        <v-icon v-else-if="tabStatus?.payroll === 'invalid'" size="small" color="error" class="mr-1">mdi-alert-circle</v-icon>
        Payroll
      </v-tab>
      <v-tab value="history">
        <v-icon size="small" color="grey" class="mr-1">mdi-history</v-icon>
        History
      </v-tab>
    </v-tabs>

    <v-window v-model="activeTab" class="tab-content">
      <v-window-item value="personal">
        <div class="pa-4">
          <slot name="personal"></slot>
        </div>
      </v-window-item>

      <v-window-item value="employment">
        <div class="pa-4">
          <slot name="employment"></slot>
        </div>
      </v-window-item>

      <v-window-item value="payroll">
        <div class="pa-4">
          <slot name="payroll"></slot>
        </div>
      </v-window-item>

      <v-window-item value="history">
        <div class="pa-4">
          <slot name="history"></slot>
        </div>
      </v-window-item>
    </v-window>
  </v-card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  modelValue?: string
  tabStatus?: Record<string, 'pending' | 'valid' | 'invalid'>
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const activeTab = ref<string>('personal')

// Sync with v-model
watch(() => props.modelValue, (val) => {
  if (val) activeTab.value = val
})

watch(activeTab, (val) => {
  emit('update:modelValue', val)
})
</script>

<style scoped>
.tab-card {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.tab-nav {
  flex-shrink: 0;
}

.tab-nav :deep(.v-tab) {
  text-transform: none;
  font-weight: 500;
}

.tab-content {
  flex: 1 1 auto;
  overflow: hidden;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.tab-content :deep(.v-window) {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.tab-content :deep(.v-window__container) {
  height: 100%;
  flex: 1 1 auto;
  min-height: 0;
}

.tab-content :deep(.v-window-item) {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

:deep(.v-window-item > div.pa-4) {
  flex: 1 1 auto;
  overflow-y: auto;
  overflow-x: hidden;
  min-height: 0;
  padding-bottom: 1.5rem;
  scrollbar-gutter: stable;
  scrollbar-width: thin;
  scrollbar-color: #9e9e9e #f5f5f5;
}

:deep(.v-window-item > div.pa-4::-webkit-scrollbar) {
  width: 8px;
}

:deep(.v-window-item > div.pa-4::-webkit-scrollbar-track) {
  background: #f5f5f5;
}

:deep(.v-window-item > div.pa-4::-webkit-scrollbar-thumb) {
  background: #9e9e9e;
  border-radius: 4px;
}

:deep(.v-window-item > div.pa-4::-webkit-scrollbar-thumb:hover) {
  background: #757575;
}

@media (max-width: 960px) {
  .tab-nav {
    flex-direction: column;
  }
}
</style>