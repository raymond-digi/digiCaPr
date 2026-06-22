<template>
  <v-snackbar v-model="showUpdate" :timeout="-1" color="info" location="bottom right">
    Update available: v{{ updateInfo?.version }}
    <template v-slot:actions>
      <v-btn variant="text" @click="installUpdate" :loading="installing">
        Install Now
      </v-btn>
      <v-btn variant="text" @click="showUpdate = false">
        Dismiss
      </v-btn>
    </template>
  </v-snackbar>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { updateApi, type UpdateInfo } from '@/services/api'

const showUpdate = ref(false)
const installing = ref(false)
const updateInfo = ref<UpdateInfo | null>(null)

onMounted(async () => {
  try {
    const info = await updateApi.checkForUpdates()
    if (info) {
      updateInfo.value = info
      showUpdate.value = true
    }
  } catch (e) {
    console.log('Update check failed:', e)
  }
})

const installUpdate = async () => {
  installing.value = true
  try {
    await updateApi.installUpdate()
  } finally {
    installing.value = false
    showUpdate.value = false
  }
}
</script>
