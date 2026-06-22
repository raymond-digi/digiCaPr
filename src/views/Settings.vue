<template>
  <div>
    <v-row>
      <v-col cols="12">
        <h1 class="text-h5 mb-4">Settings</h1>
      </v-col>
    </v-row>

    <!-- Company Information -->
    <v-row class="mt-4" xv-if="!appStore.isDatabaseOpen">
      <v-col cols="12">
        <v-card>
          <v-card-title>Company Information</v-card-title>
          <v-card-text>
            <v-alert v-if="!appStore.isDatabaseOpen" type="info" variant="tonal" class="mb-4">
              Open a database to manage company information.
            </v-alert>

            <v-alert v-else-if="companyStore.company" type="success" variant="tonal" class="mb-4">
              <div><strong>Company:</strong> {{ companyStore.company.name }}</div>
              <div><strong>Business Number:</strong> {{ companyStore.company.business_number }}</div>
            </v-alert>

            <v-alert v-else type="warning" variant="tonal" class="mb-4">
              No company information found. Please add company details.
            </v-alert>

            <v-btn color="primary" prepend-icon="mdi-office-building" :disabled="!appStore.isDatabaseOpen" @click="editCompany">
              {{ companyStore.company ? 'Edit' : 'Add' }} Company Info
            </v-btn>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Database Management -->
    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-title>Database Management</v-card-title>
          <v-card-text>
            <v-alert v-if="appStore.isDatabaseOpen" type="success" variant="tonal" class="mb-4">
              <strong>Database Connected:</strong> {{ appStore.databasePath }}
            </v-alert>
            <v-alert v-else type="warning" variant="tonal" class="mb-4">
              No database is currently open. Please create or open a database to continue.
            </v-alert>

            <v-row>
              <v-col cols="12" md="4">
                <v-btn block color="primary" prepend-icon="mdi-database-plus" :disabled="appStore.loading" @click="handleCreateDatabase">
                  Create New Database
                </v-btn>
              </v-col>

              <v-col cols="12" md="4">
                <v-btn block color="info" prepend-icon="mdi-folder-open" :disabled="appStore.loading" @click="handleOpenDatabase">
                  Open Database
                </v-btn>
              </v-col>

              <v-col cols="12" md="4">
                <v-btn v-if="!appStore.isDatabaseOpen" block color="secondary" variant="tonal" prepend-icon="mdi-clock-outline" :disabled="appStore.loading" @click="router.push({ name: 'welcome' })">
                  Recent Databases
                </v-btn>
                <v-btn v-else block color="error" prepend-icon="mdi-database-remove" :disabled="!appStore.isDatabaseOpen || appStore.loading" @click="handleCloseDatabase">
                  Close Database
                </v-btn>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Application Info -->
    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-title>Application Information</v-card-title>
          <v-row>
            <v-col cols="12" sm="4">
              <v-card-text>
                <v-list>
                  <v-list-item>
                    <v-list-item-title>Application Name</v-list-item-title>
                    <v-list-item-subtitle>Canadian Payroll App</v-list-item-subtitle>
                  </v-list-item>
                  <v-list-item>
                    <v-list-item-title>Version</v-list-item-title>
                    <v-list-item-subtitle>{{ appVersion }}</v-list-item-subtitle>
                  </v-list-item>
                </v-list>
              </v-card-text>
            </v-col>

            <!-- App Update -->
            <v-col cols="12" sm="4">
              <v-card-text>
                <div>
                  <div class="text-subtitle-1 font-weight-medium mb-2">App Update</div>
                  <v-btn color="primary" prepend-icon="mdi-update" :loading="checkingAppUpdate" @click="checkAppUpdate">
                    Check for App Updates
                  </v-btn>
                  <v-alert v-if="appUpdateMessage" :type="appUpdateAvailable ? 'success' : 'info'" variant="tonal" class="mt-2">
                    {{ appUpdateMessage }}
                  </v-alert>
                </div>
              </v-card-text>
            </v-col>

            <!-- Config Update -->
            <v-col cols="12" sm="4">
              <v-card-text>
                <div>
                  <div class="text-subtitle-1 font-weight-medium mb-2">Tax Config Update</div>
                  <v-btn color="secondary" prepend-icon="mdi-file-cog" :loading="checkingConfigUpdate" :disabled="!appStore.isDatabaseOpen" @click="checkConfigUpdate">
                    Check for Config Updates
                  </v-btn>
                  <v-alert v-if="configUpdateMessage" :type="configUpdateAvailable ? 'success' : 'info'" variant="tonal" class="mt-2">
                    {{ configUpdateMessage }}
                  </v-alert>
                </div>
              </v-card-text>
            </v-col>
          </v-row>
        </v-card>
      </v-col>
    </v-row>

    <!-- Developer Tools -->
    <v-row class="mt-4">
      <v-col cols="12">
        <v-card>
          <v-card-title>Developer Tools</v-card-title>
          <v-card-text>
            <v-alert type="warning" variant="tonal" class="mb-4">
              <strong>Warning:</strong> Developer Mode allows direct modification of payroll history records without tax calculations. Use with caution for database maintenance only.
            </v-alert>
            <v-switch v-model="appStore.devMode" label="Enable Developer Mode" color="warning" />
            <div class="text-caption text-grey mt-2">
              When enabled, the Payroll Journal (History) page will allow adding, editing, and removing records directly without any tax or deduction calculations.
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Company Form Dialog -->
    <CompanyForm v-model="companyDialog" :company="companyStore.company" @save="handleSaveCompany" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { useCompanyStore } from '@/stores/company'
import { open, save } from '@tauri-apps/plugin-dialog'
import CompanyForm from '@/components/forms/CompanyForm.vue'
import { useRouter } from 'vue-router'
import { getErrorMessage } from '@/utils/error'
import { updateApi, configUpdateApi } from '@/services/api'
declare const __APP_VERSION__: string

const appStore = useAppStore()
const companyStore = useCompanyStore()
const router = useRouter()
const appVersion = __APP_VERSION__

const companyDialog = ref(false)

// Update state
const checkingAppUpdate = ref(false)
const appUpdateMessage = ref('')
const appUpdateAvailable = ref(false)
const checkingConfigUpdate = ref(false)
const configUpdateMessage = ref('')
const configUpdateAvailable = ref(false)

const checkAppUpdate = async () => {
  checkingAppUpdate.value = true
  appUpdateMessage.value = ''
  try {
    const info = await updateApi.checkForUpdates()
    if (info) {
      appUpdateAvailable.value = true
      appUpdateMessage.value = `Version ${info.version} is available. It will be installed on next launch.`
    } else {
      appUpdateAvailable.value = false
      appUpdateMessage.value = 'App is up to date.'
    }
  } catch (error) {
    appUpdateAvailable.value = false
    appUpdateMessage.value = `Update check failed: ${getErrorMessage(error)}`
  } finally {
    checkingAppUpdate.value = false
  }
}

const checkConfigUpdate = async () => {
  checkingConfigUpdate.value = true
  configUpdateMessage.value = ''
  try {
    const currentYear = new Date().getFullYear()
    const newerYear = await configUpdateApi.checkConfigUpdates(currentYear)
    if (newerYear) {
      configUpdateAvailable.value = true
      configUpdateMessage.value = `New tax config available for ${newerYear}. Downloading...`
      await configUpdateApi.downloadConfigUpdate(newerYear)
      configUpdateMessage.value = `Tax config for ${newerYear} downloaded successfully.`
      appStore.showNotification(`Tax config updated to ${newerYear}`, 'success')
    } else {
      configUpdateAvailable.value = false
      configUpdateMessage.value = 'Tax config is up to date.'
    }
  } catch (error) {
    configUpdateAvailable.value = false
    configUpdateMessage.value = `Config update check failed: ${getErrorMessage(error)}`
  } finally {
    checkingConfigUpdate.value = false
  }
}

onMounted(async () => {
  if (appStore.isDatabaseOpen) {
    try {
      await companyStore.fetchCompany()
    } catch (error) {
      console.error('Failed to load company info:', error)
    }
  }
})

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
      await appStore.createDatabase(filePath as string)
      // After creating a new database, show the company form so user can enter company info
      companyDialog.value = true
    }
  } catch (error) {
    appStore.showNotification(`Failed to create database: ${getErrorMessage(error)}`, 'error')
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
      await appStore.openDatabase(selected)
      // Reload company info after opening database
      await companyStore.fetchCompany()
      router.push({ name: 'dashboard' })
    }
  } catch (error) {
    appStore.showNotification(`Failed to open database: ${getErrorMessage(error)}`, 'error')
  }
}

const handleCloseDatabase = async () => {
  try {
    // Update company name in recent list before closing
    if (appStore.databasePath) {
      const companyName = companyStore.company?.name ?? null
      const { useRecentStore } = await import('@/stores/recent')
      const recentStore = useRecentStore()
      await recentStore.updateCompany(appStore.databasePath, companyName)
    }
    await appStore.closeDatabase()
    companyStore.company = null
    router.push({ name: 'welcome' })
  } catch (error) {
    appStore.showNotification(`Failed to close database: ${getErrorMessage(error)}`, 'error')
  }
}

const editCompany = () => {
  companyDialog.value = true
}

const handleSaveCompany = async (company: any) => {
  try {
    await companyStore.saveCompany(company)
    appStore.showNotification('Company information saved successfully', 'success')
  } catch (error) {
    appStore.showNotification(`Failed to save company: ${getErrorMessage(error)}`, 'error')
  }
}
</script>
