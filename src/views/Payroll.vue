<template>
  <div class="fill-height">
    <!-- Step 1: Create New Current Payroll (only if no current Current Payroll) -->
    <v-row v-if="step === 1">
      <v-col>
        <v-card>
          <v-card-title class="text-h5">Create New Payroll</v-card-title>
          <v-card-subtitle>No current payroll found</v-card-subtitle>
          <v-card-text>
            <div class="mb-6 d-flex ga-3">
              <!-- Preset buttons -->
              <v-btn color="info" @click="setLastWeek" variant="outlined">
                Last Week
              </v-btn>
              <v-btn color="warning" @click="setLastTwoWeeks" variant="outlined">
                Last 2 Weeks
              </v-btn>
              <v-btn color="secondary" @click="setLastMonth" variant="outlined">
                Last Month
              </v-btn>
              <v-btn color="success" @click="setLastHalfMonth" variant="outlined">
                Last Half Month
              </v-btn>
            </div>

            <v-form ref="newPayrollForm" v-model="newPayrollFormValid">
              <v-row>
                <v-col cols="12" md="4">
                  <v-text-field v-model="payrollInput.pay_period_start" label="Pay Period Start" type="date" :rules="[rules.required]" variant="outlined" density="compact" />
                </v-col>
                <v-col cols="12" md="4">
                  <v-text-field v-model="payrollInput.pay_period_end" label="Pay Period End" type="date" :rules="[rules.required]" variant="outlined" density="compact" />
                </v-col>
                <v-col cols="12" md="4">
                  <v-text-field v-model="payrollInput.pay_date" label="Pay Date" type="date" :rules="[rules.required]" variant="outlined" density="compact" />
                </v-col>
              </v-row>

              <v-row>
                <v-col cols="12" md="4">
                  <v-checkbox v-model="autoAddSalary" label="Auto add salary employees" color="primary" />
                </v-col>
                <v-col cols="12" md="4">
                  <v-text-field v-model.number="payrollInput.pay_period_number" label="Pay Period Number" type="number" min="1" :rules="[rules.required, rules.positiveInteger]" variant="outlined"
                    density="compact" :hint="calculatedPayPeriodInfo ? `Number: ${calculatedPayPeriodInfo.payPeriodNumber}` : 'N/A'" persistent-hint />
                </v-col>
                <v-col cols="12" md="4">
                  <v-text-field v-model.number="payrollInput.total_pay_periods" label="Total Pay Periods" type="number" min="1" :rules="[rules.required, rules.positiveInteger]" variant="outlined"
                    density="compact" :hint="calculatedPayPeriodInfo ? `Total: ${calculatedPayPeriodInfo.totalPayPeriods} (${calculatedPayPeriodInfo.payFrequency})` : 'N/A'" persistent-hint />
                </v-col>
              </v-row>

              <v-alert v-if="autoAddSalary" type="info" variant="tonal" class="mb-4">
                All active employees paid by salary will be added to this payroll.
              </v-alert>
            </v-form>
          </v-card-text>
          <v-card-actions>
            <v-spacer />
            <v-btn color="primary" :loading="currentStore.loading" :disabled="!newPayrollFormValid" @click="createPayroll">
              Create Payroll
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <!-- Step 2: Review and Edit Current Payroll -->
    <v-row v-if="step === 2" class="flex-grow-1" xstyle="min-height: 0;">
      <v-col cols="12" class="d-flex flex-column" style="height: calc(100vh - 75px);">
        <v-card class="d-flex flex-column" style="height: 100%;">
          <v-card-title class="text-h5 d-flex justify-space-between align-center">
            <span>Payroll</span>
            <div class="d-flex ga-2 align-center">
              <v-chip color="primary" size="small">
                {{ currentStore.payrollTotal.count }} Employees
              </v-chip>
              <v-chip color="grey-darken-3" variant="tonal" size="small" v-if="currentStore.currentPayrollDates || payrollInput.pay_period_start">
                Pay Period {{ currentStore.currentPayrollDates?.pay_period_number || payrollInput.pay_period_number }}
                of {{ currentStore.currentPayrollDates?.total_pay_periods || payrollInput.total_pay_periods }}
                ({{ formatDate(currentStore.currentPayrollDates?.pay_period_start || payrollInput.pay_period_start) }}
                - {{ formatDate(currentStore.currentPayrollDates?.pay_period_end || payrollInput.pay_period_end) }})
              </v-chip>
            </div>
          </v-card-title>
          <v-card-text class="flex-grow-1" style="overflow-y: auto; min-height: 0;">
            <v-row dense class="mb-2">
              <v-col cols="12" md="2">
                <v-card variant="tonal" color="info" class="px-2 py-1">
                  <div class="text-caption">Gross Pay</div>
                  <div class="text-subtitle-1 font-weight-bold">{{ formatCurrency(currentStore.payrollTotal.grossPay) }}</div>
                </v-card>
              </v-col>
              <v-col cols="12" md="2">
                <v-card variant="tonal" color="purple" class="px-2 py-1">
                  <div class="text-caption">CPP</div>
                  <div class="text-subtitle-1 font-weight-bold">{{ formatCurrency(currentStore.payrollTotal.cppTotal) }}</div>
                </v-card>
              </v-col>
              <v-col cols="12" md="2">
                <v-card variant="tonal" color="orange" class="px-2 py-1">
                  <div class="text-caption">EI</div>
                  <div class="text-subtitle-1 font-weight-bold">{{ formatCurrency(currentStore.payrollTotal.eiTotal) }}</div>
                </v-card>
              </v-col>
              <v-col cols="12" md="2">
                <v-card variant="tonal" color="red" class="px-2 py-1">
                  <div class="text-caption">Taxes</div>
                  <div class="text-subtitle-1 font-weight-bold">{{ formatCurrency(currentStore.payrollTotal.federalTaxTotal +
                    currentStore.payrollTotal.provincialTaxTotal) }}</div>
                </v-card>
              </v-col>
              <v-col cols="12" md="2">
                <v-card variant="tonal" color="warning" class="px-2 py-1">
                  <div class="text-caption">Others</div>
                  <div class="text-subtitle-1 font-weight-bold">{{ formatCurrency(currentStore.payrollTotal.additionalDeductionsTotal) }}</div>
                </v-card>
              </v-col>
              <v-col cols="12" md="2">
                <v-card variant="tonal" color="success" class="px-2 py-1">
                  <div class="text-caption">Net Pay</div>
                  <div class="text-subtitle-1 font-weight-bold">{{ formatCurrency(currentStore.payrollTotal.netPay) }}</div>
                </v-card>
              </v-col>
            </v-row>

            <!-- Errors Display -->
            <v-alert v-if="currentStore.errors && currentStore.errors.length > 0" type="warning" variant="tonal" class="mb-4">
              <div class="font-weight-bold mb-2">{{ currentStore.errors.length }} Error(s) occurred:</div>
              <div v-for="error in currentStore.errors" :key="error.employee_id" class="text-body-2">
                • {{ error.employee_name }}: {{ error.error }}
              </div>
            </v-alert>

            <!-- Payroll Table -->
            <v-data-table v-model:expanded="expandedItems" :items="enrichedPayrollData" :headers="payrollHeaders" :loading="currentStore.loading" show-expand density="compact" item-key="id"
              items-per-page="10" :sort-by="[{ key: 'employee_number', order: 'asc' }]">
              <template #item.employee_id="{ item }">
                {{ getEmployeeName(item.employee_id) }}
              </template>

              <template #item.employee_number="{ item }">
                {{ item.employee_number }}
              </template>

              <template #item.gross_pay="{ item }">
                {{ formatCurrency(item.gross_pay) }}
              </template>

              <template #item.additional_earnings="{ item }">
                {{ formatCurrency(item.additional_earnings_total ?? 0) }}
              </template>

              <template #item.deductions="{ item }">
                {{ formatCurrency(
                  Number(item.deductions?.cpp ?? 0) +
                  Number(item.deductions?.ei ?? 0) +
                  Number(item.deductions?.federal_tax ?? 0) +
                  Number(item.deductions?.provincial_tax ?? 0) +
                  Number(item.additional_deductions ?? 0)
                ) }}
              </template>

              <template #item.net_pay="{ item }">
                {{ formatCurrency(item.net_pay) }}
              </template>

              <template #item.base_input="{ item }">
                <div v-if="item.pay_type === 'Hourly'" class="text-end">
                  <span class="text-caption">Hrs: {{ Number(item.regular_hours ?? 0).toFixed(2) }}</span>
                  / <span class="text-caption"> {{ Number(item.overtime_hours ?? 0).toFixed(2) }}</span>
                </div>
                <div v-else class="text-end">
                  {{ formatCurrency(item.gross_pay) }}
                </div>
              </template>

              <template #item.actions="{ item }">
                <v-btn icon="mdi-pencil" size="small" variant="text" color="primary" @click="editPayroll(item)" />
                <v-btn icon="mdi-delete" size="small" variant="text" color="error" @click="confirmDelete(item)" />
              </template>

              <template #expanded-row="{ columns, item }">
                <td :colspan="columns.length">
                  <v-container fluid>
                    <v-row>
                      <v-col cols="12" md="6">
                        <v-card variant="tonal" class="pa-4 mb-4">
                          <div class="text-h6 mb-2">Earnings Breakdown</div>
                          <v-row>
                            <v-col cols="12" sm="6">
                              <div v-if="getCurrentPayrollData(item.id)?.pay_type == 'Hourly'"><strong>Regular Hours:</strong>
                                {{ Number(getCurrentPayrollData(item.id)?.regular_hours ?? 0).toFixed(2) }} hrs </div>
                              <div v-if="getCurrentPayrollData(item.id)?.pay_type == 'Hourly'"><strong>Overtime Hours:</strong>
                                {{ Number(getCurrentPayrollData(item.id)?.overtime_hours ?? 0).toFixed(2) }} hrs</div>
                              <div><strong>Gross Pay:</strong> {{ formatCurrency(getCurrentPayrollData(item.id)?.gross_pay) }}</div>
                            </v-col>
                            <v-col cols="12" sm="6">
                              <div><strong>Additional:</strong> {{ formatCurrency(getAdditionalEarningsTotal(item.id)) }}</div>
                            </v-col>
                          </v-row>
                        </v-card>
                      </v-col>
                      <v-col cols="12" md="6">
                        <v-card variant="tonal" class="pa-4 mb-4">
                          <div class="text-h6 mb-2">Deductions Breakdown</div>
                          <v-row>
                            <v-col cols="12" sm="6">
                              <div><strong>CPP:</strong> {{ formatCurrency(getCurrentPayrollData(item.id)?.deductions?.cpp) }}</div>
                              <div><strong>EI:</strong> {{ formatCurrency(getCurrentPayrollData(item.id)?.deductions?.ei) }}</div>
                              <div><strong>Taxes:</strong> {{ formatCurrency(
                                Number(getCurrentPayrollData(item.id)?.deductions?.federal_tax ?? 0) +
                                Number(getCurrentPayrollData(item.id)?.deductions?.provincial_tax ?? 0)) }}
                              </div>
                            </v-col>
                            <v-col cols="12" sm="6">
                              <!-- <div><strong>Statutory:</strong>{{ formatCurrency(
                                Number(getCurrentPayrollData(item.id)?.deductions?.cpp ?? 0) +
                                Number(getCurrentPayrollData(item.id)?.deductions?.ei ?? 0) +
                                Number(getCurrentPayrollData(item.id)?.deductions?.federal_tax ?? 0) +
                                Number(getCurrentPayrollData(item.id)?.deductions?.provincial_tax ?? 0)) }}
                              </div> -->
                              <div><strong>Additional:</strong> {{ formatCurrency(getAdditionalDeductionsTotal(item.id)) }}</div>
                              <div class="my-5"></div>
                              <div><strong>Net Pay:</strong> {{ formatCurrency(getCurrentPayrollData(item.id)?.net_pay) }}</div>
                            </v-col>
                          </v-row>
                        </v-card>
                      </v-col>
                    </v-row>
                  </v-container>
                </td>
              </template>
            </v-data-table>
          </v-card-text>
          <v-card-actions class="flex-shrink-0">
            <v-btn color="primary" prepend-icon="mdi-plus" @click="showAddEmployeeDialog">
              Add Payroll
            </v-btn>
            <v-btn color="info" prepend-icon="mdi-file-import" @click="importCsv" :loading="importing">
              Import CSV
            </v-btn>
            <v-btn color="success" prepend-icon="mdi-file-export" @click="exportCsv" :loading="exporting" :disabled="currentStore.payrollTotal.count === 0 || currentStore.loading">
              Export CSV
            </v-btn>
            <v-btn variant="outlined" prepend-icon="mdi-file-chart" @click="generateReport" :disabled="currentStore.payrollTotal.count === 0 || currentStore.loading" size="small">
              Report
            </v-btn>
            <v-btn variant="outlined" prepend-icon="mdi-file-pdf-box" @click="generatePaystubs" :disabled="currentStore.payrollTotal.count === 0 || currentStore.loading" size="small">
              Paystubs
            </v-btn>
            <v-spacer />
            <v-btn variant="outlined" color="error" @click="showResetDialog = true">
              Reset
            </v-btn>
            <v-btn color="primary" :disabled="currentStore.payrollTotal.count === 0" @click="showPostDialog = true">
              Post to History
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <!-- Employee Selection Dialog -->
    <v-dialog v-model="employeeSelectDialog" max-width="800">
      <v-card>
        <v-card-title class="pa-4 pb-2">
          Select Employee to Add
          <v-chip color="grey-darken-3" variant="tonal" size="small" class="ml-2" v-if="currentStore.currentPayrollDates">
            Pay Period {{ currentStore.currentPayrollDates?.pay_period_number || payrollInput.pay_period_number }}
            of {{ currentStore.currentPayrollDates?.total_pay_periods || payrollInput.total_pay_periods }}
          </v-chip>
        </v-card-title>
        <v-divider />
        <v-card-text class="pa-4">
          <v-alert v-if="currentStore.availableEmployees.length === 0" type="info" variant="tonal">
            No additional employees available for this pay period.
          </v-alert>
          <v-list v-else>
            <v-list-item v-for="employee in currentStore.availableEmployees" :key="employee.id" @click="selectEmployeeForAdd(employee)" class="mb-2" border rounded>
              <template #prepend>
                <v-avatar color="primary">
                  {{ employee.first_name[0] }}{{ employee.last_name[0] }}
                </v-avatar>
              </template>
              <v-list-item-title>
                {{ employee.employee_number }} - {{ employee.first_name }} {{ employee.last_name }}
              </v-list-item-title>
              <v-list-item-subtitle>
                {{ employee.pay_type }} - {{ formatCurrency(employee.pay_rate) }}
                <span v-if="employee.pay_type === 'Hourly'"> /hr</span>
                <span v-else-if="employee.pay_type === 'Annual'"> /year</span>
                <span v-else-if="employee.pay_type === 'Weekly'"> /week</span>
                <span v-else-if="employee.pay_type === 'Monthly'"> /month</span>
              </v-list-item-subtitle>
              <template #append>
                <v-icon>mdi-chevron-right</v-icon>
              </template>
            </v-list-item>
          </v-list>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="outlined" @click="employeeSelectDialog = false">
            Cancel
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Payroll Form Dialog -->
    <PayrollForm v-model="editDialog" :payroll="editingPayroll" :employee="editingEmployee" :is-add-mode="isAddMode" :pay-period-dates="currentStore.currentPayrollDates" @save="handlePayrollSave" />

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="showDeleteDialog" max-width="500">
      <v-card>
        <v-card-title>Confirm Delete</v-card-title>
        <v-card-text>
          <v-alert type="warning" variant="tonal" class="mb-4">
            Are you sure you want to remove <strong>{{ deletingEmployeeName }}</strong> from this payroll?
            <br><br>
            This action cannot be undone.
          </v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="outlined" @click="showDeleteDialog = false">
            Cancel
          </v-btn>
          <v-btn color="error" :loading="currentStore.loading" @click="confirmDeletePayroll">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Reset Confirmation Dialog -->
    <v-dialog v-model="showResetDialog" max-width="500">
      <v-card>
        <v-card-title>Confirm Reset</v-card-title>
        <v-card-text>
          <v-alert type="warning" variant="tonal" class="mb-4">
            This will clear all current payroll data and return you to the "Create New Payroll" step.
            <br><br>
            This action cannot be undone.
          </v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="outlined" @click="showResetDialog = false">
            Cancel
          </v-btn>
          <v-btn color="error" :loading="currentStore.loading" @click="confirmReset">
            Reset Payroll
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Post to History Confirmation Dialog -->
    <v-dialog v-model="showPostDialog" max-width="600">
      <v-card>
        <v-card-title>Post Payroll to History</v-card-title>
        <v-card-text>
          <v-alert type="warning" variant="tonal" class="mb-4">
            This action will:
            <ul class="mt-2">
              <li>Mark all payrolls as "Paid"</li>
              <li>Update employee YTD amounts</li>
              <li>Make payrolls part of permanent history</li>
            </ul>
            This cannot be undone easily.
          </v-alert>

          <div class="text-h6 mt-4">
            Total: {{ currentStore.payrollTotal.count }} payrolls
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showPostDialog = false">Cancel</v-btn>
          <v-btn color="primary" :loading="currentStore.loading" @click="postToHistory">
            Confirm Post
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Duplicate Dates Warning Dialog -->
    <v-dialog v-model="showDuplicateDatesDialog" max-width="600">
      <v-card>
        <v-card-title>Warning: Duplicate Payroll Dates</v-card-title>
        <v-card-text>
          <v-alert type="warning" variant="tonal" class="mb-4">
            Payroll records with the same dates already exist in history:
            <ul class="mt-2">
              <li><strong>Pay Period Start:</strong> {{ formatDate(payrollInput.pay_period_start) }}</li>
              <li><strong>Pay Period End:</strong> {{ formatDate(payrollInput.pay_period_end) }}</li>
              <li><strong>Pay Date:</strong> {{ formatDate(payrollInput.pay_date) }}</li>
            </ul>
            <br>
            Do you want to continue creating a new payroll with these dates?
          </v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="outlined" @click="cancelDuplicateDatesCreation">
            Cancel
          </v-btn>
          <v-btn color="warning" :loading="currentStore.loading" @click="confirmDuplicateDatesCreation">
            Continue Anyway
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useCurrentPayrollStore } from '@/stores/currentPayroll'
import { useEmployeeStore } from '@/stores/employee'
import { useAppStore } from '@/stores/app'
import { payrollApi, reportsApi } from '@/services/api'
import { getErrorMessage } from '@/utils/error'
import { calculatePayPeriodInfo } from '@/utils/pay-period'
import { formatDateLocal, toDateString } from '@/utils/date'
import type { Payroll, CurrentPayrollInput } from '@/types/payroll'
import type { Employee } from '@/types/employee'
import PayrollForm from '@/components/forms/PayrollForm.vue'
import { save, open } from '@tauri-apps/api/dialog'

const currentStore = useCurrentPayrollStore()
const employeeStore = useEmployeeStore()
const appStore = useAppStore()

const step = ref(1)
const newPayrollFormValid = ref(false)
const autoAddSalary = ref(true)
const employeeSelectDialog = ref(false)
const showPostDialog = ref(false)
const showDuplicateDatesDialog = ref(false)

const editDialog = ref(false)
const isAddMode = ref(false)
const editingPayroll = ref<Payroll | null>(null)
const expandedItems = ref<string[]>([])
const editingEmployee = ref<Employee | null>(null)
const showResetDialog = ref(false)
const showDeleteDialog = ref(false)
const deletingPayrollId = ref<number | null>(null)
const deletingEmployeeName = ref('')
const importing = ref(false)
const exporting = ref(false)

const payrollInput = ref<CurrentPayrollInput>({
  pay_period_start: '',
  pay_period_end: '',
  pay_date: '',
  pay_period_number: 0,
  total_pay_periods: 0,
  employee_ids: undefined
})

const rules = {
  required: (v: any) => !!v || 'Required',
  positive: (v: number) => v > 0 || 'Must be positive',
  positiveInteger: (v: number) => (v > 0 && Number.isInteger(v)) || 'Must be positive integer'
}

const formatCurrency = (value: any): string => {
  const num = Number(value ?? 0);
  if (Number.isNaN(num)) {
    return '$0.00';
  }
  return num.toLocaleString('en-CA', {
    style: 'currency',
    currency: 'CAD'
  });
};

const calculatedPayPeriodInfo = computed(() => {
  const start = payrollInput.value.pay_period_start
  const end = payrollInput.value.pay_period_end
  if (!start || !end) return null
  try {
    return calculatePayPeriodInfo(start, end)
  } catch (e) {
    return null
  }
})

// Enrich payroll data with employee information for sorting
const enrichedPayrollData = computed(() => {
  return currentStore.payrolls.map(payroll => {
    const employee = employeeStore.employees.find(e => e.id === payroll.employee_id)
    return {
      ...payroll,
      employee_number: employee?.employee_number || 'N/A',
      employee_name: employee ? `${employee.first_name} ${employee.last_name}` : `Employee #${payroll.employee_id}`,
      pay_type: employee?.pay_type || 'Unknown'
    }
  })
})


const payrollHeaders = [
  { key: 'data-table-expand' },
  {
    title: 'Emp #',
    key: 'employee_number',
    sortable: true,
    sort: (a: string, b: string) => {
      // Handle non-numeric employee numbers
      if (a === 'N/A') return 1
      if (b === 'N/A') return -1
      return a.localeCompare(b, undefined, { numeric: true })
    }
  },
  {
    title: 'Employee',
    key: 'employee_id',
    sortable: true,
    sort: (a: number, b: number) => {
      const nameA = getEmployeeName(a).toLowerCase()
      const nameB = getEmployeeName(b).toLowerCase()
      return nameA.localeCompare(nameB)
    }
  },
  { title: 'Hours / Gross', key: 'base_input', sortable: false, align: 'end' as const },
  { title: 'Gross Pay', key: 'gross_pay', sortable: false, align: 'end' as const },
  { title: 'Additional', key: 'additional_earnings', sortable: false, align: 'end' as const },
  { title: 'Deductions', key: 'deductions', sortable: false, align: 'end' as const },
  { title: 'Net Pay', key: 'net_pay', sortable: false, align: 'end' as const },
  { title: 'Actions', key: 'actions', sortable: false }
]

const getEmployeeName = (employeeId: number) => {
  const employee = employeeStore.employees.find(e => e.id === employeeId)
  return employee ? `${employee.first_name} ${employee.last_name}` : `Employee #${employeeId}`
}

// Create a reactive getter that computes payroll data dynamically
const getCurrentPayrollData = (payrollId: number | undefined) => {
  if (!payrollId) return null;
  // Always fetch fresh from the store to ensure reactivity
  const freshPayroll = currentStore.payrolls.find(p => p.id === payrollId)
  if (!freshPayroll) return null;

  // Enrich with employee data
  const employee = employeeStore.employees.find(e => e.id === freshPayroll.employee_id)
  return {
    ...freshPayroll,
    employee_number: employee?.employee_number || 'N/A',
    employee_name: employee ? `${employee.first_name} ${employee.last_name}` : `Employee #${freshPayroll.employee_id}`,
    pay_type: employee?.pay_type || 'Unknown'
  }
}

// Computed property for additional earnings to ensure reactivity
const getAdditionalEarningsTotal = (payrollId: number | undefined) => {
  if (!payrollId) return 0
  const payroll = currentStore.payrolls.find(p => p.id === payrollId)
  if (!payroll || !payroll.additional_earnings) return 0
  return payroll.additional_earnings.reduce((sum, e) => sum + Number(e.amount || 0), 0)
}

// Computed property for additional deductions to ensure reactivity
const getAdditionalDeductionsTotal = (payrollId: number | undefined) => {
  if (!payrollId) return 0
  const payroll = currentStore.payrolls.find(p => p.id === payrollId)
  if (!payroll || !payroll.deductions?.additional) return 0
  return payroll.deductions.additional.reduce((sum, d) => sum + Number(d.amount || 0), 0)
}

const formatDate = formatDateLocal

// Preset date functions

const setLastWeek = () => {
  const today = new Date()
  const day = today.getDay()
  const sunday = new Date(today)
  sunday.setDate(today.getDate() - day - 7)
  const saturday = new Date(sunday)
  saturday.setDate(sunday.getDate() + 6)
  payrollInput.value.pay_period_start = toDateString(sunday)
  payrollInput.value.pay_period_end = toDateString(saturday)
  payrollInput.value.pay_date = toDateString(saturday)
}

const setLastTwoWeeks = () => {
  const today = new Date()
  const day = today.getDay()
  const sunday = new Date(today)
  sunday.setDate(today.getDate() - day - 14)
  const saturday = new Date(sunday)
  saturday.setDate(sunday.getDate() + 13) // 14 days Sun-Sat
  payrollInput.value.pay_period_start = toDateString(sunday)
  payrollInput.value.pay_period_end = toDateString(saturday)
  payrollInput.value.pay_date = toDateString(saturday)
}

const setLastMonth = () => {
  const now = new Date();
  const lastMonthDate = new Date(now.getFullYear(), now.getMonth() - 1, 1);
  const endOfLastMonth = new Date(now.getFullYear(), now.getMonth(), 0);
  payrollInput.value.pay_period_start = toDateString(lastMonthDate);
  payrollInput.value.pay_period_end = toDateString(endOfLastMonth);
  payrollInput.value.pay_date = toDateString(endOfLastMonth);
}

const setLastHalfMonth = () => {
  const now = new Date();
  const todayDay = now.getDate();
  const currentMonth = now.getMonth();
  const currentYear = now.getFullYear();
  let startDate, endDate;
  if (todayDay <= 15) {
    // Previous month second half: 16 - end
    let lastMonth = currentMonth - 1;
    let year = currentYear;
    if (lastMonth < 0) {
      lastMonth = 11;
      year--;
    }
    startDate = new Date(year, lastMonth, 16);
    endDate = new Date(year, lastMonth + 1, 0);
  } else {
    // Current month first half: 1 - 15
    startDate = new Date(currentYear, currentMonth, 1);
    endDate = new Date(currentYear, currentMonth, 15);
  }
  payrollInput.value.pay_period_start = toDateString(startDate);
  payrollInput.value.pay_period_end = toDateString(endDate);
  payrollInput.value.pay_date = toDateString(endDate);
}

watch([() => payrollInput.value.pay_period_start, () => payrollInput.value.pay_period_end], () => {
  if (calculatedPayPeriodInfo.value) {
    payrollInput.value.pay_period_number = calculatedPayPeriodInfo.value.payPeriodNumber
    payrollInput.value.total_pay_periods = calculatedPayPeriodInfo.value.totalPayPeriods
  }
}, { immediate: true })

const createPayroll = async () => {
  try {
    // Check if history payroll records exist for the same dates
    const datesExist = await payrollApi.checkHistoryPayrollDatesExist(
      payrollInput.value.pay_period_start,
      payrollInput.value.pay_period_end,
      payrollInput.value.pay_date
    )

    if (datesExist) {
      // Show warning dialog and wait for user confirmation
      showDuplicateDatesDialog.value = true
      return
    }

    await proceedWithPayrollCreation()
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    console.error('Payroll creation error:', error)
    appStore.showNotification(`Failed to create current payroll: ${errorMsg}`, 'error')
  }
}

const proceedWithPayrollCreation = async () => {
  try {
    const input: CurrentPayrollInput = {
      pay_period_start: payrollInput.value.pay_period_start,
      pay_period_end: payrollInput.value.pay_period_end,
      pay_date: payrollInput.value.pay_date,
      pay_period_number: payrollInput.value.pay_period_number,
      total_pay_periods: payrollInput.value.total_pay_periods,
      employee_ids: autoAddSalary.value ? undefined : []
    }

    await currentStore.createPayroll(input)

    if (currentStore.payrollTotal.count > 0) {
      appStore.showNotification(
        `Created with ${currentStore.payrollTotal.count} payroll(s)`,
        'success'
      )
      step.value = 2
    } else if (currentStore.errors && currentStore.errors.length > 0) {
      appStore.showNotification('Payroll created with errors - review and fix', 'warning')
      step.value = 2
    } else {
      // Empty payroll - allow manual employee addition
      if (!autoAddSalary.value) {
        appStore.showNotification('Empty Payroll created - add employees manually', 'info')
        step.value = 2
      } else {
        appStore.showNotification('No active employees found to add', 'warning')
      }
    }
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    console.error('Payroll creation error:', error)
    appStore.showNotification(`Failed to create current payroll: ${errorMsg}`, 'error')
  }
}

const confirmDuplicateDatesCreation = async () => {
  showDuplicateDatesDialog.value = false
  await proceedWithPayrollCreation()
}

const cancelDuplicateDatesCreation = () => {
  showDuplicateDatesDialog.value = false
}

const showAddEmployeeDialog = async () => {
  try {
    await currentStore.fetchAvailableEmployees(
      currentStore.currentPayrollDates?.pay_period_start || payrollInput.value.pay_period_start,
      currentStore.currentPayrollDates?.pay_period_end || payrollInput.value.pay_period_end
    )

    if (currentStore.availableEmployees.length === 0) {
      appStore.showNotification('No additional employees available for this pay period', 'info')
      return
    }

    employeeSelectDialog.value = true
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(`Failed to fetch employees: ${errorMsg}`, 'error')
  }
}

const selectEmployeeForAdd = (employee: Employee) => {
  // Close employee selection dialog
  employeeSelectDialog.value = false

  // Set add mode
  isAddMode.value = true
  editingEmployee.value = employee
  editingPayroll.value = null

  // Open edit dialog in add mode
  editDialog.value = true
}

const confirmDelete = (item: Payroll) => {
  deletingPayrollId.value = item.id!
  deletingEmployeeName.value = getEmployeeName(item.employee_id)
  showDeleteDialog.value = true
}

const confirmDeletePayroll = async () => {
  if (!deletingPayrollId.value) return

  try {
    await currentStore.removeFromPayroll(deletingPayrollId.value)
    appStore.showNotification('Removed from payroll', 'success')
    showDeleteDialog.value = false
    deletingPayrollId.value = null
    deletingEmployeeName.value = ''
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(`Failed to remove: ${errorMsg}`, 'error')
  }
}

const postToHistory = async () => {
  try {
    const payrollIds = currentStore.payrolls
      .filter(p => p.id)
      .map(p => p.id!)

    const newPayrollIds = await currentStore.postCurrentToHistory(payrollIds)

    appStore.showNotification(
      `Posted ${newPayrollIds.length} payroll(s) to history`,
      'success'
    )

    showPostDialog.value = false
    await resetPayroll()
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(`Failed to post payroll: ${errorMsg}`, 'error')
  }
}

const resetPayroll = async () => {
  try {
    await currentStore.resetCurrentPayroll()
    appStore.showNotification('Current payroll reset successfully', 'success')
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(`Reset failed: ${errorMsg}`, 'error')
    return
  }
  // Reset UI state
  step.value = 1
  payrollInput.value = {
    pay_period_start: '',
    pay_period_end: '',
    pay_date: '',
    pay_period_number: 0,
    total_pay_periods: 0,
    employee_ids: undefined
  }
  autoAddSalary.value = true
}

const confirmReset = async () => {
  await resetPayroll()
  showResetDialog.value = false
}

onMounted(async () => {
  try {
    await employeeStore.fetchEmployees()
    await currentStore.loadCurrentPayroll()

    if (currentStore.currentPayrollDates) {
      payrollInput.value.pay_period_start = currentStore.currentPayrollDates.pay_period_start
      payrollInput.value.pay_period_end = currentStore.currentPayrollDates.pay_period_end
      payrollInput.value.pay_date = currentStore.currentPayrollDates.pay_date
      payrollInput.value.pay_period_number = currentStore.currentPayrollDates.pay_period_number || 0
      payrollInput.value.total_pay_periods = currentStore.currentPayrollDates.total_pay_periods || 0
      step.value = 2
    } else {
      // Set default dates (current week Sunday-Saturday)
      const today = new Date()
      const day = today.getDay()
      const sunday = new Date(today)
      sunday.setDate(today.getDate() - day)
      const saturday = new Date(sunday)
      saturday.setDate(sunday.getDate() + 6)

      payrollInput.value.pay_period_start = toDateString(sunday)
      payrollInput.value.pay_period_end = toDateString(saturday)
      payrollInput.value.pay_date = toDateString(saturday)
    }
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(`Failed to load data: ${errorMsg}`, 'error')
  }
});

const editPayroll = (item: Payroll) => {
  isAddMode.value = false
  editingPayroll.value = JSON.parse(JSON.stringify(item))
  editingEmployee.value = employeeStore.employees.find(e => e.id === item.employee_id) || null
  editDialog.value = true
}

const handlePayrollSave = async (payroll: Payroll) => {
  try {
    // Use different API calls for add vs edit
    if (isAddMode.value) {
      await payrollApi.addToCurrentPayroll(payroll)
      appStore.showNotification(
        `Added ${editingEmployee.value?.first_name} ${editingEmployee.value?.last_name} to payroll`,
        'success'
      )
    } else {
      await payrollApi.updateCurrentPayroll(payroll)
      appStore.showNotification('Payroll updated successfully', 'success')
    }

    // Reload the entire payroll to ensure totals are recalculated properly
    await currentStore.loadCurrentPayroll()
    // Refresh employee store to ensure enriched payroll data has up-to-date employee info
    await employeeStore.fetchEmployees()
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    console.error('Error saving payroll:', error)
    appStore.showNotification(`Failed to ${isAddMode.value ? 'add' : 'update'} payroll: ${errorMsg}`, 'error')
  }
}

const generatePaystubs = async () => {
  if (currentStore.payrollTotal.count === 0) {
    appStore.showNotification('No payrolls to generate paystubs for', 'warning')
    return
  }

  try {
    const payrollIds = currentStore.payrolls
      .filter(p => p.id)
      .map(p => p.id!)

    const outputDir = 'reports/paystubs'
    const files = await reportsApi.generateCurrentPayrollPaystubs(payrollIds, outputDir)
    appStore.showNotification(
      `Generated ${files.length} pay stubs`,
      'success'
    )
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(
      `Pay stub generation failed: ${errorMsg}`,
      'error'
    )
  }
}

const generateReport = async () => {
  if (currentStore.payrollTotal.count === 0) {
    appStore.showNotification('No payrolls to generate report for', 'warning')
    return
  }

  try {
    const payrollIds = currentStore.payrolls
      .filter(p => p.id)
      .map(p => p.id!)

    const outputDir = 'reports'
    await reportsApi.generateCurrentPayrollReport(payrollIds, outputDir)
    appStore.showNotification(
      `Generated summary report`,
      'success'
    )
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(
      `Report generation failed: ${errorMsg}`,
      'error'
    )
  }
}

const exportCsv = async () => {
  if (currentStore.payrollTotal.count === 0) {
    appStore.showNotification('No payrolls to export', 'warning')
    return
  }

  try {
    const filePath = await save({
      defaultPath: 'payroll.csv',
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }]
    })

    if (filePath) {
      exporting.value = true
      const result = await currentStore.exportCurrentPayrollCsv(filePath)
      appStore.showNotification(
        `Exported payroll to ${result}`,
        'success'
      )
    }
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(
      `Export failed: ${errorMsg}`,
      'error'
    )
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
      // Use current payroll dates if available
      const dates = currentStore.currentPayrollDates
      const result = await currentStore.importCurrentPayrollCsv(
        filePath,
        dates?.pay_period_start,
        dates?.pay_period_end,
        dates?.pay_date
      )

      if (result.errors.length > 0) {
        appStore.showNotification(
          `Imported ${result.payrolls.length} payroll(s) with ${result.errors.length} error(s)`,
          'warning'
        )
      } else {
        appStore.showNotification(
          `Successfully imported ${result.payrolls.length} payroll(s)`,
          'success'
        )
      }
    }
  } catch (error) {
    const errorMsg = getErrorMessage(error)
    appStore.showNotification(
      `Import failed: ${errorMsg}`,
      'error'
    )
  } finally {
    importing.value = false
  }
}

</script>
