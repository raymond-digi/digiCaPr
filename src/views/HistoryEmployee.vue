<template>
  <div class="history-layout">
    <div class="history-main">
      <!-- Title Bar -->
      <v-row class="align-center mb-4">
        <v-col cols="auto">
          <h1 class="text-h5">Payroll History - Employee</h1>
        </v-col>
        <v-spacer />
        <v-col cols="auto" class="d-flex ga-2">
          <v-btn variant="outlined" icon="mdi-refresh" size="small" :loading="payrollStore.loading" @click="loadYears" />
        </v-col>
      </v-row>

      <div class="history-body">
        <!-- Employee List (Left) -->
        <div class="history-employee-panel">
          <v-card>
            <v-text-field v-model="employeeSearch" density="compact" variant="outlined" placeholder="Search employees..." prepend-inner-icon="mdi-magnify" clearable hide-details class="mx-3 mt-3" />
            <v-list density="compact" nav class="py-0">
              <v-list-item v-for="emp in filteredEmployees" :key="emp.id" :active="selectedEmployeeId === emp.id" :title="`${emp.first_name} ${emp.last_name}`" :subtitle="emp.employee_number"
                @click="selectEmployee(emp.id!)" />
            </v-list>
          </v-card>
        </div>

        <!-- Content (Right) -->
        <div class="history-content">
          <!-- No Employee Selected -->
          <v-card v-if="!selectedEmployeeId">
            <v-card-text>
              <v-alert type="info" variant="tonal" class="mb-0">
                Select an employee from the list to view their payroll history.
              </v-alert>
            </v-card-text>
          </v-card>

          <!-- Year Table (Level 1) with Payroll Expanded Row (Level 2) -->
          <v-card v-else-if="payrollStore.years.length > 0">
            <v-card-text class="pa-0">
              <v-data-table v-model:expanded="expandedYearItems" :items="yearTableItems" :headers="yearHeaders" show-expand density="compact" item-value="year" :items-per-page="-1" hide-default-footer
                disable-sort>
                <template #item.year="{ item }">
                  <v-icon class="mr-2" size="small">mdi-calendar</v-icon>
                  <strong>{{ item.year }}</strong>
                </template>

                <template #item.count="{ item }">
                  <v-chip size="small" variant="tonal">{{ item.count }}</v-chip>
                </template>

                <template #item.grossPay="{ item }">
                  {{ formatCurrency(item.grossPay) }}
                </template>

                <template #item.deductions="{ item }">
                  {{ formatCurrency(item.deductions) }}
                </template>

                <template #item.netPay="{ item }">
                  <strong class="text-success">{{ formatCurrency(item.netPay) }}</strong>
                </template>

                <template #item.actions="{ item }">
                  <v-btn icon="mdi-export" size="small" variant="text" @click.stop="exportYear(item.year)" :disabled="payrollStore.loading" />
                  <v-btn icon="mdi-file-pdf-box" size="small" variant="text" color="primary" @click.stop="exportYearPdf(item.year)" :disabled="payrollStore.loading" />
                </template>

                <!-- Expanded Row: Payroll Table (Level 2) -->
                <template #expanded-row="{ columns, item }">
                  <td :colspan="columns.length" class="pa-0">
                    <v-card variant="flat" class="ml-4 mr-4 mb-2">
                      <v-card-text>
                        <v-progress-linear v-if="loadingYear === item.year" indeterminate color="primary" class="mb-2" />

                        <v-alert v-else-if="!yearPayrolls[item.year] || yearPayrolls[item.year].length === 0" type="info" variant="tonal" density="compact">
                          No payroll records found for {{ item.year }}.
                        </v-alert>

                        <v-data-table v-else :items="yearPayrolls[item.year]" :headers="payrollHeaders" density="compact" :items-per-page="-1" hide-default-footer fixed-header
                          disable-sort>
                          <template #item.payDate="{ item }">
                            {{ formatDate(item.pay_date) }}
                          </template>

                          <template #item.period="{ item }">
                            {{ formatDateShort(item.pay_period_start) }} – {{ formatDateShort(item.pay_period_end) }}
                          </template>

                          <template #item.grossPay="{ item }">
                            {{ formatCurrency(item.gross_pay) }}
                          </template>

                          <template #item.deductions="{ item }">
                            {{ formatCurrency(
                              Number(item.deductions?.cpp ?? 0) +
                              Number(item.deductions?.cpp2 ?? 0) +
                              Number(item.deductions?.ei ?? 0) +
                              Number(item.deductions?.federal_tax ?? 0) +
                              Number(item.deductions?.provincial_tax ?? 0)
                            ) }}
                          </template>

                          <template #item.netPay="{ item }">
                            <strong>{{ formatCurrency(item.net_pay) }}</strong>
                          </template>

                          <template #item.actions="{ item }">
                            <v-btn icon="mdi-eye" size="small" variant="text" @click.stop="viewPayroll(item)" />
                          </template>
                        </v-data-table>
                      </v-card-text>
                    </v-card>
                  </td>
                </template>
              </v-data-table>
            </v-card-text>
          </v-card>

          <!-- Empty State -->
          <v-card v-else-if="selectedEmployeeId && !payrollStore.loading">
            <v-card-text class="d-flex align-center">
              <v-alert type="info" variant="tonal" class="flex-grow-1 mb-0">
                No payroll history found for this employee.
              </v-alert>
            </v-card-text>
          </v-card>
        </div>
      </div>
    </div>

    <!-- View Payroll Details Dialog -->
    <v-dialog v-model="showDetailsDialog" max-width="700">
      <v-card v-if="viewingPayroll">
        <v-card-title class="d-flex align-center">
          Payroll Details
          <v-spacer />
          <v-btn icon="mdi-close" size="small" variant="text" @click="showDetailsDialog = false" />
        </v-card-title>
        <v-card-text>
          <v-row>
            <v-col cols="6">
              <div class="text-caption">Pay Date</div>
              <div class="text-body-1">{{ formatDate(viewingPayroll.pay_date) }}</div>
            </v-col>
            <v-col cols="6">
              <div class="text-caption">Pay Period</div>
              <div class="text-body-1">{{ formatDate(viewingPayroll.pay_period_start) }} – {{ formatDate(viewingPayroll.pay_period_end) }}</div>
            </v-col>
            <v-col cols="6">
              <div class="text-caption">Period #</div>
              <div class="text-body-1">{{ viewingPayroll.pay_period_number || 'N/A' }} of {{ viewingPayroll.total_pay_periods }}</div>
            </v-col>
          </v-row>

          <v-divider class="my-3" />

          <!-- Earnings -->
          <div class="text-subtitle-1 font-weight-bold mb-2">Earnings</div>
          <v-list density="compact">
            <v-list-item v-if="viewingPayroll.regular_hours">
              <v-list-item-title>Regular Hours</v-list-item-title>
              <template #append>{{ Number(viewingPayroll.regular_hours).toFixed(2) }} hrs</template>
            </v-list-item>
            <v-list-item v-if="viewingPayroll.overtime_hours">
              <v-list-item-title>Overtime Hours</v-list-item-title>
              <template #append>{{ Number(viewingPayroll.overtime_hours).toFixed(2) }} hrs</template>
            </v-list-item>
            <v-list-item>
              <v-list-item-title>Gross Pay</v-list-item-title>
              <template #append><strong>{{ formatCurrency(viewingPayroll.gross_pay) }}</strong></template>
            </v-list-item>
            <v-list-item v-for="earning in viewingPayroll.additional_earnings" :key="earning.earning_type">
              <v-list-item-title>{{ earning.earning_type }}</v-list-item-title>
              <template #append>{{ formatCurrency(earning.amount) }}</template>
            </v-list-item>
          </v-list>

          <v-divider class="my-3" />

          <!-- Deductions -->
          <div class="text-subtitle-1 font-weight-bold mb-2">Deductions</div>
          <v-list density="compact">
            <v-list-item>
              <v-list-item-title>CPP</v-list-item-title>
              <template #append>{{ formatCurrency(viewingPayroll.deductions?.cpp) }}</template>
            </v-list-item>
            <v-list-item>
              <v-list-item-title>EI</v-list-item-title>
              <template #append>{{ formatCurrency(viewingPayroll.deductions?.ei) }}</template>
            </v-list-item>
            <v-list-item>
              <v-list-item-title>Federal Tax</v-list-item-title>
              <template #append>{{ formatCurrency(viewingPayroll.deductions?.federal_tax) }}</template>
            </v-list-item>
            <v-list-item>
              <v-list-item-title>Provincial Tax</v-list-item-title>
              <template #append>{{ formatCurrency(viewingPayroll.deductions?.provincial_tax) }}</template>
            </v-list-item>
            <v-list-item v-for="deduction in viewingPayroll.deductions?.additional" :key="deduction.name">
              <v-list-item-title>{{ deduction.name }}</v-list-item-title>
              <template #append>{{ formatCurrency(deduction.amount) }}</template>
            </v-list-item>
          </v-list>

          <v-divider class="my-3" />

          <!-- Net Pay -->
          <v-list density="compact">
            <v-list-item class="bg-success">
              <v-list-item-title class="text-white font-weight-bold">Net Pay</v-list-item-title>
              <template #append>
                <strong class="text-white text-h6">{{ formatCurrency(viewingPayroll.net_pay) }}</strong>
              </template>
            </v-list-item>
          </v-list>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showDetailsDialog = false">Close</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { usePayrollStore } from '@/stores/historyPayroll'
import { useEmployeeStore } from '@/stores/employee'
import { useAppStore } from '@/stores/app'
import { reportsApi } from '@/services/api'
import type { Payroll } from '@/types/payroll'
import { getErrorMessage } from '@/utils/error'
import { save, open } from '@tauri-apps/api/dialog'

const payrollStore = usePayrollStore()
const employeeStore = useEmployeeStore()
const appStore = useAppStore()

// --- State ---
const selectedEmployeeId = ref<number | null>(null)
const employeeSearch = ref('')

// Year expansion state
const expandedYearItems = ref<string[]>([])
const yearTotals = reactive<Record<number, {
  count: number
  grossPay: number
  deductions: number
  netPay: number
}>>({})
const loadingYear = ref<number | null>(null)

// Payroll data cache per year
const yearPayrolls = reactive<Record<number, Payroll[]>>({})

// Dialog state
const showDetailsDialog = ref(false)
const viewingPayroll = ref<Payroll | null>(null)

// --- Table Headers ---
const yearHeaders = [
  { title: 'Year', key: 'year' },
  { title: 'Payrolls', key: 'count', sortable: false },
  { title: 'Gross Pay', key: 'grossPay' },
  { title: 'Deductions', key: 'deductions' },
  { title: 'Net Pay', key: 'netPay' },
  { title: '', key: 'actions', sortable: false }
]

const payrollHeaders = [
  { title: 'Pay Date', key: 'payDate' },
  { title: 'Period', key: 'period', sortable: false },
  { title: 'Gross Pay', key: 'grossPay' },
  { title: 'Deductions', key: 'deductions', sortable: false },
  { title: 'Net Pay', key: 'netPay' },
  { title: '', key: 'actions', sortable: false }
]

// --- Computed ---
const filteredEmployees = computed(() => {
  const list = employeeStore.employees
  if (!employeeSearch.value) return list
  const search = employeeSearch.value.toLowerCase()
  return list.filter(e =>
    e.first_name.toLowerCase().includes(search) ||
    e.last_name.toLowerCase().includes(search) ||
    e.employee_number.toLowerCase().includes(search)
  )
})

const yearTableItems = computed(() => {
  return payrollStore.years.map(year => {
    const totals = yearTotals[year]
    return {
      year,
      count: totals?.count ?? 0,
      grossPay: totals?.grossPay ?? 0,
      deductions: totals?.deductions ?? 0,
      netPay: totals?.netPay ?? 0
    }
  })
})

// --- Watchers ---
watch(expandedYearItems, async (newVal, oldVal) => {
  const oldSet = new Set(oldVal.map(String))
  const opened = newVal.filter(y => !oldSet.has(String(y)))
  for (const yearVal of opened) {
    const year = typeof yearVal === 'number' ? yearVal : Number(String(yearVal))
    if (!isNaN(year)) {
      await loadYearData(year)
    }
  }
}, { deep: true })

// --- Data Loading ---
const loadYearData = async (year: number) => {
  if (isNaN(year) || !year) return
  if (yearPayrolls[year]) return
  loadingYear.value = year
  try {
    await payrollStore.fetchPayrolls({
      pay_date_from: `${year}-01-01`,
      pay_date_to: `${year}-12-31`,
      employee_id: selectedEmployeeId.value ?? null,
      limit: null,
      offset: null
    })

    const payrolls = payrollStore.payrolls

    let grossPay = 0, totalDeductions = 0, netPay = 0
    for (const p of payrolls) {
      grossPay += Number(p.gross_pay ?? 0)
      totalDeductions += Number(p.deductions?.cpp ?? 0) + Number(p.deductions?.cpp2 ?? 0) + Number(p.deductions?.ei ?? 0) + Number(p.deductions?.federal_tax ?? 0) + Number(p.deductions?.provincial_tax ?? 0)
      netPay += Number(p.net_pay ?? 0)
    }

    yearTotals[year] = {
      count: payrolls.length,
      grossPay,
      deductions: totalDeductions,
      netPay
    }

    yearPayrolls[year] = [...payrolls].sort((a, b) => b.pay_date.localeCompare(a.pay_date))
  } catch (error) {
    appStore.showNotification(`Failed to load data for ${year}: ${getErrorMessage(error)}`, 'error')
  } finally {
    loadingYear.value = null
  }
}

const loadYears = async () => {
  try {
    await payrollStore.fetchYears(selectedEmployeeId.value ?? undefined)
    clearAllCaches()

    // Eagerly load all years' data
    for (const year of payrollStore.years) {
      await loadYearData(year)
    }
    expandedYearItems.value = payrollStore.years.map(String)
  } catch (error) {
    appStore.showNotification(`Failed to load years: ${getErrorMessage(error)}`, 'error')
  }
}

// --- Cache Management ---
const clearAllCaches = () => {
  Object.keys(yearPayrolls).forEach(k => delete yearPayrolls[Number(k)])
  Object.keys(yearTotals).forEach(k => delete yearTotals[Number(k)])
  expandedYearItems.value = []
}

// --- Employee Selection ---
const selectEmployee = async (employeeId: number) => {
  selectedEmployeeId.value = employeeId
  await loadYears()
}

// --- Formatting Helpers ---
const formatDate = (dateStr: string) => {
  if (!dateStr) return ''
  const parts = dateStr.split('-')
  if (parts.length === 3) {
    const year = parseInt(parts[0], 10)
    const month = parseInt(parts[1], 10) - 1
    const day = parseInt(parts[2], 10)
    return new Date(year, month, day).toLocaleDateString('en-CA', { year: 'numeric', month: 'short', day: 'numeric' })
  }
  return new Date(dateStr).toLocaleDateString('en-CA', { year: 'numeric', month: 'short', day: 'numeric' })
}

/** Format date, hiding year if it matches the parent year */
const formatDateShort = (dateStr: string) => {
  if (!dateStr) return ''
  const parts = dateStr.split('-')
  if (parts.length === 3) {
    const year = parseInt(parts[0], 10)
    const month = parseInt(parts[1], 10) - 1
    const day = parseInt(parts[2], 10)
    return new Date(year, month, day).toLocaleDateString('en-CA', { month: 'short', day: 'numeric' })
  }
  return new Date(dateStr).toLocaleDateString('en-CA', { month: 'short', day: 'numeric' })
}

const formatCurrency = (value: any): string => {
  const num = Number(value ?? 0)
  if (Number.isNaN(num)) return '$0.00'
  return num.toLocaleString('en-CA', { style: 'currency', currency: 'CAD' })
}

// --- Actions ---
const viewPayroll = (payroll: Payroll) => {
  viewingPayroll.value = payroll
  showDetailsDialog.value = true
}

const exportYearPdf = async (year: number) => {
  try {
    const outputDir = await open({
      directory: true,
      title: 'Select output directory for PDF report'
    })
    if (!outputDir || typeof outputDir !== 'string') return

    const payrolls = yearPayrolls[year]
    if (!payrolls || payrolls.length === 0) {
      appStore.showNotification('No payroll records to export', 'warning')
      return
    }

    const payrollIds = payrolls.filter(p => p.id).map(p => p.id!)
    if (payrollIds.length === 0) {
      appStore.showNotification('No valid payroll records found', 'warning')
      return
    }

    await reportsApi.generateHistoryPayrollReport(payrollIds, outputDir)
    appStore.showNotification(`Generated PDF report for ${year}`, 'success')
  } catch (error) {
    appStore.showNotification(`PDF export failed: ${getErrorMessage(error)}`, 'error')
  }
}

const exportYear = async (year: number) => {
  try {
    const filePath = await save({
      defaultPath: `employee_${selectedEmployeeId.value}_payroll_${year}.csv`,
      filters: [{ name: 'CSV', extensions: ['csv'] }]
    })
    if (filePath) {
      await payrollStore.exportHistoryPayrollCsv(
        filePath,
        selectedEmployeeId.value ?? null,
        `${year}-01-01`,
        `${year}-12-31`,
        null
      )
      appStore.showNotification(`Exported payroll history for ${year}`, 'success')
    }
  } catch (error) {
    appStore.showNotification(`Export failed: ${getErrorMessage(error)}`, 'error')
  }
}

// --- Init ---
onMounted(async () => {
  await employeeStore.fetchEmployees()
})
</script>

<style scoped>
.history-layout {
  height: calc(100vh - 140px);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.history-body {
  display: flex;
  gap: 12px;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.history-employee-panel {
  flex: 0 0 220px;
  min-height: 0;
  overflow-y: auto;
}

.history-content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
}

/* Responsive: stack vertically on small screens */
@media (max-width: 959px) {
  .history-body {
    flex-direction: column;
    overflow: visible;
  }

  .history-employee-panel,
  .history-content {
    flex: none;
    max-width: 100%;
    min-height: 0;
    overflow-y: visible;
  }
}
</style>
