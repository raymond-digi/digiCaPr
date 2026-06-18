<template>
  <v-app-bar color="primary" elevation="2">
    <v-app-bar-nav-icon @click="appStore.toggleSidebar" />
    
    <v-toolbar-title class="text-h6">
      Digi Canadian Payroll App
    </v-toolbar-title>
    
    <v-spacer />
    
    <!-- Database Status -->
    <v-chip 
      v-if="appStore.isDatabaseOpen" 
      color="success" 
      variant="flat"
      prepend-icon="mdi-database-check"
      class="mr-2"
    >
      {{ databaseName }}
    </v-chip>
    <v-chip 
      v-else 
      color="warning" 
      variant="flat"
      prepend-icon="mdi-database-alert"
      class="mr-2"
    >
      No Database
    </v-chip>
    
    <!-- Company Name -->
    <v-chip 
      v-if="companyStore.hasCompany"
      variant="outlined"
      prepend-icon="mdi-office-building"
    >
      {{ companyStore.companyName }}
    </v-chip>
  </v-app-bar>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'
import { useCompanyStore } from '@/stores/company'

const appStore = useAppStore()
const companyStore = useCompanyStore()

const databaseName = computed(() => {
  if (!appStore.databasePath) return 'Connected'
  const path = appStore.databasePath
  const parts = path.split(/[\\/]/)
  return parts[parts.length - 1]
})
</script>
