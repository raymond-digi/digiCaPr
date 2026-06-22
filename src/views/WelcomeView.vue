<template>
  <div class="welcome-container d-flex align-center justify-center" style="min-height: 90vh;">
    <div style="width: 100%; max-width: 600px;">
      <!-- App Logo / Name -->
      <div class="text-center mb-8">
        <v-icon size="64" color="primary" class="mb-4">mdi-leaf-maple</v-icon>
        <h1 class="text-h4 font-weight-bold mb-1">Digi Canadian Payroll</h1>
        <p class="text-subtitle-1 text-medium-emphasis">Select a database to get started</p>
      </div>

      <!-- Recent Databases List -->
      <v-card v-if="recentStore.hasRecent" class="mb-6">
        <v-card-title class="text-subtitle-1 font-weight-medium">
          <v-icon start size="small">mdi-clock-outline</v-icon>
          Recent Databases
        </v-card-title>
        <v-divider />
        <v-list lines="three" class="py-0">
          <v-list-item v-for="db in recentStore.recentDatabases" :key="db.path" class="py-3">
            <template #prepend>
              <v-icon color="primary">mdi-database</v-icon>
            </template>

            <v-list-item-title class="font-weight-medium">
              <template v-if="db.company_name">
                {{ db.company_name }} — {{ db.file_name }}
              </template>
              <template v-else>
                {{ db.file_name }}
              </template>
            </v-list-item-title>

            <v-list-item-subtitle class="mt-1">
              <span class="text-caption text-medium-emphasis">{{ db.path }}</span>
            </v-list-item-subtitle>

            <template #append>
              <div class="d-flex align-center ga-1">
                <v-btn size="small" color="primary" variant="tonal" prepend-icon="mdi-open-in-app" :disabled="loading" @click="handleOpenRecent(db)">
                  Open
                </v-btn>
                <v-btn size="small" variant="text" color="grey" icon="mdi-close" @click="confirmRemove(db)" />
              </div>
            </template>
          </v-list-item>
        </v-list>
      </v-card>

      <!-- Empty State -->
      <v-card v-else class="mb-6">
        <v-card-text class="text-center py-8">
          <v-icon size="48" color="grey-lighten-1" class="mb-4">mdi-database-outline</v-icon>
          <p class="text-body-1 text-medium-emphasis mb-0">No recent databases found.</p>
          <p class="text-caption text-medium-emphasis">Open or create a database to get started.</p>
        </v-card-text>
      </v-card>

      <!-- Action Buttons -->
      <div class="d-flex justify-center ga-4">
        <v-btn color="primary" size="large" prepend-icon="mdi-folder-open" :disabled="loading" @click="handleOpenDatabase">
          Open Database
        </v-btn>
        <v-btn color="secondary" size="large" variant="tonal" prepend-icon="mdi-database-plus" :disabled="loading" @click="handleCreateDatabase">
          Create New Database
        </v-btn>
      </div>

      <!-- Settings Link -->
      <div class="text-center my-6">
        <router-link :to="{ name: 'settings' }" class="text-caption text-medium-emphasis text-decoration-none">
          <v-icon start size="x-small">mdi-cog</v-icon>
          Settings
        </router-link>
      </div>
    </div>

    <!-- Remove Confirmation Dialog -->
    <v-dialog v-model="removeDialog" max-width="420" persistent>
      <v-card>
        <v-card-title class="text-h6">Remove from Recent</v-card-title>
        <v-card-text>
          <template v-if="dbToRemove">
            <p>
              Remove
              <strong>
                <template v-if="dbToRemove.company_name">
                  {{ dbToRemove.company_name }} — {{ dbToRemove.file_name }}
                </template>
                <template v-else>
                  {{ dbToRemove.file_name }}
                </template>
              </strong>
              from the recent databases list?
            </p>
            <p class="text-caption text-medium-emphasis mt-2">
              This will not delete the actual database file.
            </p>
          </template>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="cancelRemove">Cancel</v-btn>
          <v-btn color="error" variant="flat" @click="doRemove">Remove</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useAppStore } from '@/stores/app'
import { useCompanyStore } from '@/stores/company'
import { useRecentStore } from '@/stores/recent'
import type { RecentDatabase } from '@/types/recent'
import { getErrorMessage } from '@/utils/error'

const router = useRouter()
const appStore = useAppStore()
const companyStore = useCompanyStore()
const recentStore = useRecentStore()

const loading = ref(false)
const removeDialog = ref(false)
const dbToRemove = ref<RecentDatabase | null>(null)

onMounted(async () => {
  await recentStore.fetchRecent()
})

const handleOpenRecent = async (db: RecentDatabase) => {
  loading.value = true
  try {
    await appStore.openDatabase(db.path)
    await companyStore.fetchCompany()
    const companyName = companyStore.company?.name ?? null
    await recentStore.addRecent(db.path, companyName)
    router.push({ name: 'dashboard' })
  } catch (error) {
    appStore.showNotification(`Failed to open database: ${getErrorMessage(error)}`, 'error')
  } finally {
    loading.value = false
  }
}

const handleOpenDatabase = async () => {
  try {
    const selected = await open({
      title: 'Open Database',
      multiple: false,
      filters: [{
        name: 'Database',
        extensions: ['db', 'sqlite']
      }]
    })

    if (selected && typeof selected === 'string') {
      loading.value = true
      await appStore.openDatabase(selected)
      await companyStore.fetchCompany()
      const companyName = companyStore.company?.name ?? null
      await recentStore.addRecent(selected, companyName)
      router.push({ name: 'dashboard' })
    }
  } catch (error) {
    appStore.showNotification(`Failed to open database: ${getErrorMessage(error)}`, 'error')
  } finally {
    loading.value = false
  }
}

const handleCreateDatabase = async () => {
  try {
    const filePath = await save({
      title: 'Create New Database',
      defaultPath: 'payroll.db',
      filters: [{
        name: 'Database',
        extensions: ['db', 'sqlite']
      }]
    })

    if (filePath) {
      loading.value = true
      await appStore.createDatabase(filePath as string)
      const companyName = companyStore.company?.name ?? null
      await recentStore.addRecent(filePath as string, companyName)
      router.push({ name: 'settings' })
    }
  } catch (error) {
    appStore.showNotification(`Failed to create database: ${getErrorMessage(error)}`, 'error')
  } finally {
    loading.value = false
  }
}

const confirmRemove = (db: RecentDatabase) => {
  dbToRemove.value = db
  removeDialog.value = true
}

const cancelRemove = () => {
  dbToRemove.value = null
  removeDialog.value = false
}

const doRemove = async () => {
  if (dbToRemove.value) {
    await recentStore.removeRecent(dbToRemove.value.path)
  }
  dbToRemove.value = null
  removeDialog.value = false
}
</script>

<style scoped>
.welcome-container {
  margin: -16px;
  /* Counteract the container padding from AppLayout */
}
</style>
