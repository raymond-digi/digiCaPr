<template>
  <AppLayout>
    <router-view />
  </AppLayout>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import AppLayout from '@/components/layout/AppLayout.vue'
import { useAppStore } from '@/stores/app'
import { useCompanyStore } from '@/stores/company'

const appStore = useAppStore()
const companyStore = useCompanyStore()

onMounted(async () => {
  // Check database status on app load
  await appStore.checkDatabaseStatus()
  
  // Load company info if database is open
  if (appStore.isDatabaseOpen) {
    try {
      await companyStore.fetchCompany()
    } catch (error) {
      console.error('Failed to load company info:', error)
    }
  }
})
</script>

<style scoped>
/* Global app styles */
</style>
