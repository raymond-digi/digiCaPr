<template>
  <v-snackbar
    v-model="appStore.notification.show"
    :color="appStore.notification.type"
    :timeout="4000"
    location="top right"
    @update:model-value="onClose"
  >
    {{ appStore.notification.message }}
    
    <template #actions>
      <v-btn
        variant="text"
        @click="appStore.hideNotification()"
      >
        Close
      </v-btn>
    </template>
  </v-snackbar>
</template>

<script setup lang="ts">
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

const onClose = (value: boolean) => {
  if (!value) {
    appStore.hideNotification()
  }
}
</script>
