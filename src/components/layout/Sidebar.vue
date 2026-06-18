<template>
  <v-navigation-drawer v-model="appStore.sidebarOpen" app :rail="rail" @click="rail = false">
    <v-list>
      <v-list-item prepend-icon="mdi-chevron-left" title="Collapse" @click.stop="rail = !rail" />
    </v-list>

    <v-divider />

    <v-list density="compact" nav>
      <v-list-item v-for="item in menuItems" :key="item.to" :to="item.to" :prepend-icon="item.icon" :title="item.title" :disabled="item.requiresDatabase && !appStore.isDatabaseOpen" rounded="xl" />
    </v-list>
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

const rail = ref(false)

const menuItems = [
  {
    title: 'Dashboard',
    icon: 'mdi-view-dashboard',
    to: '/',
    requiresDatabase: true
  },
  {
    title: 'Employees',
    icon: 'mdi-account-multiple',
    to: '/employees',
    requiresDatabase: true
  },
  {
    title: 'Payroll',
    icon: 'mdi-folder-multiple',
    to: '/payroll',
    requiresDatabase: true
  },
  {
    title: 'Vacation',
    icon: 'mdi-beach',
    to: '/vacation',
    requiresDatabase: true
  },
  {
    title: 'Remittance',
    icon: 'mdi-bank-transfer',
    to: '/remittance',
    requiresDatabase: true
  },
  {
    title: 'T4 Year End',
    icon: 'mdi-file-document-outline',
    to: '/t4',
    requiresDatabase: true
  },
  {
    title: 'History (Period)',
    icon: 'mdi-cash-multiple',
    to: '/history-period',
    requiresDatabase: true
  },
  {
    title: 'History (Employee)',
    icon: 'mdi-account-clock',
    to: '/history-employee',
    requiresDatabase: true
  },
  {
    title: 'Settings',
    icon: 'mdi-cog',
    to: '/settings',
    requiresDatabase: false
  }
]
</script>
