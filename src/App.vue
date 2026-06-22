<template>
  <AppLayout>
    <router-view />
  </AppLayout>
  <UpdateNotification />
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import AppLayout from '@/components/layout/AppLayout.vue'
import UpdateNotification from '@/components/UpdateNotification.vue'
import { useAppStore } from '@/stores/app'
import { useCompanyStore } from '@/stores/company'
import { useRecentStore } from '@/stores/recent'

const appStore = useAppStore()
const companyStore = useCompanyStore()
const recentStore = useRecentStore()

onMounted(async () => {
  // Check database status on app load
  await appStore.checkDatabaseStatus()

  // Load company info and save to recent list if database is open
  if (appStore.isDatabaseOpen) {
    try {
      await companyStore.fetchCompany()
      // Save to recent list with company name
      if (appStore.databasePath) {
        const companyName = companyStore.company?.name ?? null
        await recentStore.addRecent(appStore.databasePath, companyName)
      }
    } catch (error) {
      console.error('Failed to load company info:', error)
    }
  }
})
</script>

<style scoped>
/* Global app styles */
</style>
