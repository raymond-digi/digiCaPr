<template>
  <div>
    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-title class="d-flex align-center">
            <span class="text-h5">Employees</span>
            <v-spacer />
            <v-checkbox v-model="showOnlyCurrentlyEmployed" label="Employed" density="compact" hide-details class="mr-4" />
            <v-text-field v-model="search" density="compact" prepend-inner-icon="mdi-magnify" label="Search" single-line hide-details variant="outlined" class="mr-4" style="max-width: 300px" />
          </v-card-title>

          <!-- Import Errors Display -->
          <v-alert v-if="importErrors && importErrors.length > 0" type="warning" variant="tonal" class="mb-4">
            <div class="font-weight-bold mb-2">{{ importErrors.length }} Error(s) occurred:</div>
            <div v-for="error in importErrors" :key="error.employee_number" class="text-body-2">
              • {{ error.employee_number }} - {{ error.employee_name }}: {{ error.error }}
            </div>
          </v-alert>

          <v-card-text>
            <v-data-table :items="filteredEmployees" :headers="headers" :search="search" :loading="employeeStore.loading" density="compact" items-per-page="10" class="employee-table">
              <template #header.employment_status="{ column }">
                <span class="text-caption">{{ column.title }}</span>
              </template>
              <template #item.employment_status="{ item }">
                <v-chip :color="item.termination_date ? 'error' : 'success'" size="small">
                  {{ item.termination_date ? 'Terminated' : 'Employed' }}
                </v-chip>
              </template>

              <template #item.is_active="{ item }">
                <v-chip :color="item.is_active ? 'success' : 'error'" size="small">
                  {{ item.is_active ? 'Active' : 'Inactive' }}
                </v-chip>
              </template>

              <template #item.province="{ item }">
                {{ item.address?.province || '-' }}
              </template>

              <template #item.pay_rate="{ item }">
                ${{ Number(item.pay_rate).toFixed(2) }}{{ item.pay_type === 'Hourly' ? '/hr' : item.pay_type ===
                  'Weekly' ? '/wk' : item.pay_type === 'Monthly' ? '/mo' : '/yr' }}
              </template>

              <template #item.actions="{ item }">
                <v-btn icon="mdi-pencil" size="small" variant="text" @click="editEmployee(item)" />
                <v-btn icon="mdi-delete" size="small" variant="text" color="error" @click="confirmDelete(item)" />
              </template>
            </v-data-table>
          </v-card-text>

          <v-card-actions>
            <v-btn color="primary" prepend-icon="mdi-plus" @click="openAddDialog">
              Add Employee
            </v-btn>
            <v-btn color="info" prepend-icon="mdi-file-import" @click="importCsv" :loading="importing" class="mr-2">
              Import CSV
            </v-btn>
            <v-btn color="success" prepend-icon="mdi-file-export" @click="exportCsv" :loading="exporting" class="mr-2">
              Export CSV
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <!-- Employee Form Dialog -->
    <EmployeeForm v-model="employeeDialog" :employee="selectedEmployee" @save="handleSaveEmployee" />

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Confirm Delete</v-card-title>
        <v-card-text>
          Are you sure you want to delete {{ employeeToDelete?.first_name }} {{ employeeToDelete?.last_name }}?
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" @click="deleteEmployee">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useEmployeeStore } from '@/stores/employee'
import { useAppStore } from '@/stores/app'
import type { Employee } from '@/types/employee'
import EmployeeForm from '@/components/forms/EmployeeForm.vue'
import { save, open } from '@tauri-apps/plugin-dialog'
import { getErrorMessage } from '@/utils/error'

const employeeStore = useEmployeeStore()
const appStore = useAppStore()

const search = ref('')
const showOnlyCurrentlyEmployed = ref(true)
const employeeDialog = ref(false)
const selectedEmployee = ref<Employee | null>(null)
const deleteDialog = ref(false)
const employeeToDelete = ref<Employee | null>(null)
const importErrors = ref<Array<{ employee_number: string; employee_name: string; error: string }>>([])
const importing = ref(false)
const exporting = ref(false)

const headers = [
  { title: 'Emp #', key: 'employee_number', align: 'start' as const },
  { title: 'First Name', key: 'first_name' },
  { title: 'Last Name', key: 'last_name' },
  { title: 'Province', key: 'province' },
  { title: 'Pay Rate', key: 'pay_rate' },
  { title: 'Employment', key: 'employment_status' },
  { title: 'Status', key: 'is_active' },
  { title: 'Actions', key: 'actions', sortable: false }
]

const employees = computed(() => employeeStore.employees)

const filteredEmployees = computed(() => {
  if (showOnlyCurrentlyEmployed.value) {
    return employees.value.filter(emp => !emp.termination_date)
  }
  return employees.value
})

onMounted(async () => {
  try {
    await employeeStore.fetchEmployees()
  } catch (error) {
    appStore.showNotification(`Failed to load employees: ${getErrorMessage(error)}`, 'error')
  }
})

const openAddDialog = () => {
  selectedEmployee.value = null
  employeeDialog.value = true
}

const editEmployee = (employee: Employee) => {
  selectedEmployee.value = { ...employee }
  employeeDialog.value = true
}

const handleSaveEmployee = async (_employee: Employee) => {
  try {
    await employeeStore.fetchEmployees()
    appStore.showNotification('Employee saved successfully', 'success')
  } catch (error) {
    console.error('Employee list refresh error:', error)
    appStore.showNotification('Employee saved but failed to refresh list', 'warning')
  }
}

const confirmDelete = (employee: Employee) => {
  employeeToDelete.value = employee
  deleteDialog.value = true
}

const deleteEmployee = async () => {
  if (!employeeToDelete.value?.id) return

  try {
    await employeeStore.deleteEmployee(employeeToDelete.value.id)
    appStore.showNotification('Employee deleted successfully', 'success')
    deleteDialog.value = false
    employeeToDelete.value = null
  } catch (error) {
    appStore.showNotification(`Failed to delete employee: ${getErrorMessage(error)}`, 'error')
  }
}

const exportCsv = async () => {
  try {
    const filePath = await save({
      defaultPath: 'employees.csv',
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }]
    })

    if (filePath) {
      exporting.value = true
      const count = await employeeStore.exportEmployeesCsv(filePath)
      appStore.showNotification(`Exported ${count} employees to CSV`, 'success')
    }
  } catch (error) {
    appStore.showNotification(`Failed to export CSV: ${getErrorMessage(error)}`, 'error')
  } finally {
    exporting.value = false
  }
}

const importCsv = async () => {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }]
    })

    if (filePath && typeof filePath === 'string') {
      importing.value = true
      const result = await employeeStore.importEmployeesCsv(filePath)
      importErrors.value = result.errors
      let message = `Imported ${result.imported} employees`
      if (result.skipped > 0) {
        message += `, skipped ${result.skipped}`
      }
      if (result.errors.length > 0) {
        message += ` with ${result.errors.length} errors`
        console.error('Import errors:', result.errors)
      }
      appStore.showNotification(message, result.errors.length > 0 ? 'warning' : 'success')
    }
  } catch (error) {
    appStore.showNotification(`Failed to import CSV: ${getErrorMessage(error)}`, 'error')
  } finally {
    importing.value = false
  }
}
</script>
