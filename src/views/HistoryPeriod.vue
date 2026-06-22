<template>
  <div class="history-layout">
    <!-- Main Content -->
    <div class="history-main">
      <!-- Title Bar -->
      <v-row class="align-center mb-4">
        <v-col cols="auto">
          <h1 class="text-h5">Payroll History - Pay Period</h1>
        </v-col>
        <v-spacer />
        <v-col cols="auto" class="d-flex ga-2">
          <v-btn color="info" prepend-icon="mdi-file-import" size="small" @click="importCsv" :loading="importing">
            Import CSV
          </v-btn>
          <v-btn variant="outlined" icon="mdi-refresh" size="small" :loading="payrollStore.loading" @click="loadYears" />
        </v-col>
      </v-row>

      <!-- Year Table (Level 1) -->
      <v-card v-if="payrollStore.years.length > 0">
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

            <template #item.employees="{ item }">
              <v-chip size="small" variant="tonal" color="info">{{ item.employees }}</v-chip>
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
            </template>

            <!-- Expanded Row: Period Table (Level 2) -->
            <template #expanded-row="{ columns, item }">
              <td :colspan="columns.length" class="pa-0">
                <v-card variant="flat" class="ml-4 mr-4 mb-2">
                  <v-card-text>
                    <v-progress-linear v-if="loadingYear === item.year" indeterminate color="primary" class="mb-2" />

                    <v-alert v-else-if="!yearPeriods[item.year] || yearPeriods[item.year].length === 0" type="info" variant="tonal" density="compact">
                      No pay periods found for {{ item.year }}.
                    </v-alert>

                    <div v-else>
                      <v-data-table :expanded="getExpandedPeriodKeys(item.year)" @update:expanded="(val: string[]) => { setExpandedPeriodKeys(item.year, val); loadExpandedPeriodPayrolls(val) }"
                        :items="yearPeriods[item.year]" :headers="periodHeaders" density="compact" :items-per-page="-1" hide-default-footer show-expand item-value="pay_date" fixed-header disable-sort>
                        <template #item.period="{ item }">
                          {{ formatDateShort(item.pay_period_start, parseInt(item.pay_period_start?.split('-')[0])) }} – {{ formatDateShort(item.pay_period_end,
                            parseInt(item.pay_period_start?.split('-')[0])) }}
                        </template>

                        <template #item.payDate="{ item }">
                          {{ formatDate(item.pay_date) }}
                        </template>

                        <template #item.employees="{ item }">
                          <v-chip size="small" variant="tonal">{{ item._employeeCount ?? '—' }}</v-chip>
                        </template>

                        <template #item.grossPay="{ item }">
                          {{ formatCurrency(item._grossPay ?? 0) }}
                        </template>

                        <template #item.deductions="{ item }">
                          {{ formatCurrency(item._deductions ?? 0) }}
                        </template>

                        <template #item.netPay="{ item }">
                          <strong>{{ formatCurrency(item._netPay ?? 0) }}</strong>
                        </template>

                        <template #item.actions="{ item }">
                          <v-btn icon="mdi-file-pdf-box" size="small" variant="text" color="primary" @click.stop="generatePeriodReport(item)" :disabled="payrollStore.loading" />
                          <v-btn icon="mdi-export" size="small" variant="text" @click.stop="exportPeriod(item)" :disabled="payrollStore.loading" />
                        </template>

                        <!-- Expanded Row: Payroll Table (Level 3) -->
                        <template #expanded-row="{ columns: innerColumns, item: periodItem }">
              <td :colspan="innerColumns.length" class="pa-0">
                <v-card variant="flat" class="ml-4 mr-4 mb-2">
                  <v-card-text>
                    <v-progress-linear v-if="loadingPeriod === periodItem.pay_date" indeterminate color="primary" class="mb-2" />

                    <v-alert v-else-if="!periodPayrolls[periodItem.pay_date] || periodPayrolls[periodItem.pay_date].length === 0" type="info" variant="tonal" density="compact">
                      No payroll records for this period.
                    </v-alert>

                    <v-data-table v-else :items="periodPayrolls[periodItem.pay_date]" :headers="payrollHeaders" density="compact" :items-per-page="-1" hide-default-footer fixed-header>
                      <template #item.employee="{ item }">
                        {{ getEmployeeDisplay(item.employee_id) }}
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
                        {{ formatCurrency(item.net_pay) }}
                      </template>

                      <template #item.actions="{ item }">
                        <v-btn icon="mdi-eye" size="small" variant="text" @click.stop="viewPayroll(item)" />
                        <template v-if="appStore.devMode">
                          <v-btn icon="mdi-pencil" size="small" variant="text" color="primary" @click.stop="editPayroll(item)" />
                          <v-btn icon="mdi-delete" size="small" variant="text" color="error" @click.stop="confirmDelete(item)" />
                        </template>
                      </template>
                    </v-data-table>
                  </v-card-text>
                </v-card>
              </td>
            </template>
          </v-data-table>
    </div>
    </v-card-text>
    </v-card>
    </td>
</template>
</v-data-table>
</v-card-text>
</v-card>

<!-- Empty State -->
<v-card v-else-if="!payrollStore.loading">
  <v-card-text class="d-flex align-center">
    <v-alert type="info" variant="tonal" class="flex-grow-1 mb-0">
      No payroll history found. Click <strong>Import CSV</strong> to import payroll records.
    </v-alert>
  </v-card-text>
</v-card>
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
          <div class="text-caption">Employee</div>
          <div class="text-body-1">{{ getEmployeeDisplay(viewingPayroll.employee_id) }}</div>
        </v-col>
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

<!-- History Edit Form Dialog -->
<HistoryEditForm v-model="editDialog" :payroll="editingPayroll" :employees="employeeStore.employees" @save="handleSave" />

<!-- Delete Confirmation Dialog -->
<v-dialog v-model="showDeleteDialog" max-width="500">
  <v-card>
    <v-card-title>Confirm Delete</v-card-title>
    <v-card-text>
      <v-alert type="warning" variant="tonal" class="mb-4">
        Are you sure you want to delete this payroll record for <strong>{{ deletingEmployeeName }}</strong>?
        <br /><br />
        This action cannot be undone.
      </v-alert>
    </v-card-text>
    <v-card-actions>
      <v-spacer />
      <v-btn variant="outlined" @click="showDeleteDialog = false">
        Cancel
      </v-btn>
      <v-btn color="error" :loading="payrollStore.loading" @click="confirmDeletePayroll">
        Delete
      </v-btn>
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
import type { Payroll, PayrollPeriod } from '@/types/payroll'
import { getErrorMessage } from '@/utils/error'
import { save, open } from '@tauri-apps/plugin-dialog'
import HistoryEditForm from '@/components/forms/HistoryEditForm.vue'

const payrollStore = usePayrollStore()
const employeeStore = useEmployeeStore()
const appStore = useAppStore()

// --- State ---
const importing = ref(false)

// Year expansion state
const expandedYearItems = ref<string[]>([])
const yearTotals = reactive<Record<number, {
  count: number
  employees: number
  grossPay: number
  deductions: number
  netPay: number
}>>({})
const loadingYear = ref<number | null>(null)

// Period data cache
const yearPeriods = reactive<Record<number, (PayrollPeriod & {
  _employeeCount?: number
  _grossPay?: number
  _deductions?: number
  _netPay?: number
})[]>>({})

// Period expansion state (per-year)
const expandedPeriodKeys = reactive<Record<number, string[]>>({})
const periodPayrolls = reactive<Record<string, Payroll[]>>({})
const loadingPeriod = ref<string | null>(null)

// Dialog state
const showDetailsDialog = ref(false)
const viewingPayroll = ref<Payroll | null>(null)
const editDialog = ref(false)
const editingPayroll = ref<Payroll | null>(null)
const showDeleteDialog = ref(false)
const deletingPayrollId = ref<number | null>(null)
const deletingEmployeeName = ref('')

// --- Table Headers ---
const yearHeaders = [
  { title: 'Year', key: 'year' },
  { title: 'Periods', key: 'count', sortable: false },
  { title: 'Employees', key: 'employees', sortable: false },
  { title: 'Gross Pay', key: 'grossPay' },
  { title: 'Deductions', key: 'deductions' },
  { title: 'Net Pay', key: 'netPay' },
  { title: '', key: 'actions', sortable: false }
]

const periodHeaders = [
  { title: 'Pay Date', key: 'payDate' },
  { title: 'Period', key: 'period' },
  { title: 'Emp.', key: 'employees', sortable: false },
  { title: 'Gross Pay', key: 'grossPay' },
  { title: 'Deductions', key: 'deductions' },
  { title: 'Net Pay', key: 'netPay' },
  { title: '', key: 'actions', sortable: false }
]

const payrollHeaders = [
  { title: 'Employee', key: 'employee', sortable: false },
  { title: 'Pay Date', key: 'pay_date' },
  { title: 'Gross Pay', key: 'grossPay' },
  { title: 'Deductions', key: 'deductions', sortable: false },
  { title: 'Net Pay', key: 'netPay' },
  { title: '', key: 'actions', sortable: false }
]

// --- Computed ---
const yearTableItems = computed(() => {
  return payrollStore.years.map(year => {
    const totals = yearTotals[year]
    return {
      year,
      count: totals?.count ?? 0,
      employees: totals?.employees ?? 0,
      grossPay: totals?.grossPay ?? 0,
      deductions: totals?.deductions ?? 0,
      netPay: totals?.netPay ?? 0
    }
  })
})

// --- Watchers ---
// Watch year expansion to load periods and compute totals on demand
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

// Watch period expansion to load payrolls on demand (per-year)
watch(expandedPeriodKeys, async (newVal) => {
  for (const yearStr of Object.keys(newVal)) {
    const year = Number(yearStr)
    const keys = newVal[year]
    if (!keys || keys.length === 0) continue
    for (const key of keys) {
      const payDate = String(key)
      if (!periodPayrolls[payDate]) {
        await loadPeriodPayrolls(payDate)
      }
    }
  }
}, { deep: true })

// --- Period Expansion Helpers ---
const getExpandedPeriodKeys = (year: number): string[] => {
  if (!expandedPeriodKeys[year]) {
    expandedPeriodKeys[year] = []
  }
  return expandedPeriodKeys[year]
}

const setExpandedPeriodKeys = (year: number, val: string[]) => {
  expandedPeriodKeys[year] = val
}

/** Load payrolls for any newly expanded period rows */
const loadExpandedPeriodPayrolls = async (expandedKeys: string[]) => {
  for (const key of expandedKeys) {
    const payDate = String(key)
    if (!periodPayrolls[payDate]) {
      await loadPeriodPayrolls(payDate)
    }
  }
}

// --- Data Loading ---
const loadYearData = async (year: number) => {
  if (isNaN(year) || !year) return
  if (yearPeriods[year]) return // already loaded
  loadingYear.value = year
  try {
    // Load periods for this year
    await payrollStore.fetchPeriods(year)
    const periods = [...payrollStore.periods].sort((a, b) => b.pay_date.localeCompare(a.pay_date))

    // Load all payrolls for the year to compute totals
    await payrollStore.fetchPayrolls({
      pay_date_from: `${year}-01-01`,
      pay_date_to: `${year}-12-31`,
      employee_id: null,
      limit: null,
      offset: null
    })

    const payrolls = payrollStore.payrolls

    // Compute year-level totals
    const employeeSet = new Set<number>()
    let grossPay = 0, totalDeductions = 0, netPay = 0
    for (const p of payrolls) {
      employeeSet.add(p.employee_id)
      grossPay += Number(p.gross_pay ?? 0)
      const deduction = Number(p.deductions?.cpp ?? 0) + Number(p.deductions?.cpp2 ?? 0) + Number(p.deductions?.ei ?? 0) + Number(p.deductions?.federal_tax ?? 0) + Number(p.deductions?.provincial_tax ?? 0)
      totalDeductions += deduction
      netPay += Number(p.net_pay ?? 0)
    }

    yearTotals[year] = {
      count: periods.length,
      employees: employeeSet.size,
      grossPay,
      deductions: totalDeductions,
      netPay
    }

    // Compute per-period totals from the year's payrolls
    const periodTotalsMap: Record<string, { employees: Set<number>; grossPay: number; deductions: number; netPay: number }> = {}
    for (const p of payrolls) {
      const key = p.pay_date
      if (!periodTotalsMap[key]) {
        periodTotalsMap[key] = { employees: new Set(), grossPay: 0, deductions: 0, netPay: 0 }
      }
      const pt = periodTotalsMap[key]
      pt.employees.add(p.employee_id)
      pt.grossPay += Number(p.gross_pay ?? 0)
      pt.deductions += Number(p.deductions?.cpp ?? 0) + Number(p.deductions?.cpp2 ?? 0) + Number(p.deductions?.ei ?? 0) + Number(p.deductions?.federal_tax ?? 0) + Number(p.deductions?.provincial_tax ?? 0)
      pt.netPay += Number(p.net_pay ?? 0)
    }

    yearPeriods[year] = periods.map(p => ({
      ...p,
      _employeeCount: periodTotalsMap[p.pay_date]?.employees.size ?? 0,
      _grossPay: periodTotalsMap[p.pay_date]?.grossPay ?? 0,
      _deductions: periodTotalsMap[p.pay_date]?.deductions ?? 0,
      _netPay: periodTotalsMap[p.pay_date]?.netPay ?? 0
    }))

    // Initialize expanded state for this year's period table if not yet set
    if (!expandedPeriodKeys[year]) {
      expandedPeriodKeys[year] = []
    }
  } catch (error) {
    appStore.showNotification(`Failed to load data for ${year}: ${getErrorMessage(error)}`, 'error')
  } finally {
    loadingYear.value = null
  }
}

const loadPeriodPayrolls = async (payDate: string) => {
  if (periodPayrolls[payDate]) return // already loaded
  loadingPeriod.value = payDate
  try {
    await payrollStore.fetchPayrolls({
      pay_date_from: payDate,
      pay_date_to: payDate,
      employee_id: null,
      limit: null,
      offset: null
    })
    periodPayrolls[payDate] = [...payrollStore.payrolls]
  } catch (error) {
    appStore.showNotification(`Failed to load payrolls: ${getErrorMessage(error)}`, 'error')
  } finally {
    loadingPeriod.value = null
  }
}

const loadYears = async () => {
  try {
    await payrollStore.fetchYears()
    // Clear cached data
    Object.keys(yearPeriods).forEach(k => delete yearPeriods[Number(k)])
    Object.keys(yearTotals).forEach(k => delete yearTotals[Number(k)])
    Object.keys(periodPayrolls).forEach(k => delete periodPayrolls[k])
    expandedYearItems.value = []
    Object.keys(expandedPeriodKeys).forEach(k => delete expandedPeriodKeys[Number(k)])

    // Reload year totals and period data for all years so the year table shows correct values
    for (const year of payrollStore.years) {
      await loadYearData(year)
    }
  } catch (error) {
    appStore.showNotification(`Failed to load years: ${getErrorMessage(error)}`, 'error')
  }
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
const formatDateShort = (dateStr: string, parentYear?: number) => {
  if (!dateStr) return ''
  const parts = dateStr.split('-')
  if (parts.length === 3) {
    const year = parseInt(parts[0], 10)
    const month = parseInt(parts[1], 10) - 1
    const day = parseInt(parts[2], 10)
    const hideYear = parentYear !== undefined && year === parentYear
    return new Date(year, month, day).toLocaleDateString('en-CA', hideYear
      ? { month: 'short', day: 'numeric' }
      : { year: 'numeric', month: 'short', day: 'numeric' })
  }
  return new Date(dateStr).toLocaleDateString('en-CA', { year: 'numeric', month: 'short', day: 'numeric' })
}

const formatCurrency = (value: any): string => {
  const num = Number(value ?? 0)
  if (Number.isNaN(num)) return '$0.00'
  return num.toLocaleString('en-CA', { style: 'currency', currency: 'CAD' })
}

const getEmployeeDisplay = (employeeId: number) => {
  const emp = employeeStore.employees.find(e => e.id === employeeId)
  if (emp) return `(${emp.employee_number}) ${emp.first_name} ${emp.last_name}`
  return `Employee #${employeeId}`
}

// --- Actions ---
const viewPayroll = (payroll: Payroll) => {
  viewingPayroll.value = payroll
  showDetailsDialog.value = true
}

const editPayroll = (payroll: Payroll) => {
  editingPayroll.value = JSON.parse(JSON.stringify(payroll))
  editDialog.value = true
}

const confirmDelete = (payroll: Payroll) => {
  deletingPayrollId.value = payroll.id!
  const emp = employeeStore.employees.find(e => e.id === payroll.employee_id)
  deletingEmployeeName.value = emp ? `${emp.first_name} ${emp.last_name}` : `Employee #${payroll.employee_id}`
  showDeleteDialog.value = true
}

const confirmDeletePayroll = async () => {
  if (!deletingPayrollId.value) return
  try {
    await payrollStore.deleteHistoryPayroll(deletingPayrollId.value)
    appStore.showNotification('Payroll record deleted', 'success')
    showDeleteDialog.value = false
    deletingPayrollId.value = null
    deletingEmployeeName.value = ''
    // Invalidate caches
    Object.keys(yearPeriods).forEach(k => delete yearPeriods[Number(k)])
    Object.keys(yearTotals).forEach(k => delete yearTotals[Number(k)])
    Object.keys(periodPayrolls).forEach(k => delete periodPayrolls[k])
    expandedYearItems.value = []
    Object.keys(expandedPeriodKeys).forEach(k => delete expandedPeriodKeys[Number(k)])
    await loadYears()
  } catch (error) {
    appStore.showNotification(`Failed to delete: ${getErrorMessage(error)}`, 'error')
  }
}

const handleSave = async (payroll: Payroll) => {
  try {
    if (payroll.id) {
      await payrollStore.updateHistoryPayroll(payroll)
      appStore.showNotification('Payroll record updated', 'success')
    } else {
      await payrollStore.saveRawPayroll(payroll)
      appStore.showNotification('Payroll record created (raw)', 'success')
    }
    // Invalidate caches
    Object.keys(yearPeriods).forEach(k => delete yearPeriods[Number(k)])
    Object.keys(yearTotals).forEach(k => delete yearTotals[Number(k)])
    Object.keys(periodPayrolls).forEach(k => delete periodPayrolls[k])
    expandedYearItems.value = []
    Object.keys(expandedPeriodKeys).forEach(k => delete expandedPeriodKeys[Number(k)])
    await loadYears()
  } catch (error) {
    appStore.showNotification(`Failed to save: ${getErrorMessage(error)}`, 'error')
  }
}

// --- Export / Import ---
const exportYear = async (year: number) => {
  try {
    const filePath = await save({
      defaultPath: `payroll_history_${year}.csv`,
      filters: [{ name: 'CSV', extensions: ['csv'] }]
    })
    if (filePath) {
      const result = await payrollStore.exportHistoryPayrollCsv(
        filePath,
        null,
        `${year}-01-01`,
        `${year}-12-31`,
        null
      )
      appStore.showNotification(`Exported payroll history for ${year} to ${result}`, 'success')
    }
  } catch (error) {
    appStore.showNotification(`Export failed: ${getErrorMessage(error)}`, 'error')
  }
}

const exportPeriod = async (period: PayrollPeriod) => {
  try {
    const filePath = await save({
      defaultPath: `payroll_${period.pay_date}.csv`,
      filters: [{ name: 'CSV', extensions: ['csv'] }]
    })
    if (filePath) {
      await payrollStore.exportHistoryPayrollCsv(
        filePath,
        null,
        period.pay_date,
        period.pay_date,
        null
      )
      appStore.showNotification(`Exported payroll for period ${period.pay_date}`, 'success')
    }
  } catch (error) {
    appStore.showNotification(`Export failed: ${getErrorMessage(error)}`, 'error')
  }
}

const importCsv = async () => {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: 'CSV', extensions: ['csv'] }]
    })

    if (filePath && typeof filePath === 'string') {
      importing.value = true
      const result = await payrollStore.importHistoryPayrollCsv(filePath)
      if (result.errors.length > 0) {
        appStore.showNotification(`Imported ${result.imported} payroll(s) with ${result.errors.length} error(s)`, 'warning')
      } else {
        appStore.showNotification(`Successfully imported ${result.imported} payroll(s)`, 'success')
      }

      // Clear all caches and reload
      Object.keys(yearPeriods).forEach(k => delete yearPeriods[Number(k)])
      Object.keys(yearTotals).forEach(k => delete yearTotals[Number(k)])
      Object.keys(periodPayrolls).forEach(k => delete periodPayrolls[k])
      expandedYearItems.value = []
      Object.keys(expandedPeriodKeys).forEach(k => delete expandedPeriodKeys[Number(k)])
      await loadYears()
    }
  } catch (error) {
    appStore.showNotification(`Import failed: ${getErrorMessage(error)}`, 'error')
  } finally {
    importing.value = false
  }
}

const generatePeriodReport = async (period: PayrollPeriod) => {
  try {
    // Load payrolls for this period first
    await payrollStore.fetchPayrolls({
      pay_date_from: period.pay_date,
      pay_date_to: period.pay_date,
      employee_id: null,
      limit: null,
      offset: null
    })

    if (payrollStore.payrolls.length === 0) {
      appStore.showNotification('No payroll records for this period', 'warning')
      return
    }

    const payrollIds = payrollStore.payrolls.filter(p => p.id).map(p => p.id!)
    await reportsApi.generateHistoryPayrollReport(payrollIds, 'reports')
    appStore.showNotification('Generated payroll report', 'success')
  } catch (error) {
    appStore.showNotification(`Report generation failed: ${getErrorMessage(error)}`, 'error')
  }
}

// --- Init ---
onMounted(async () => {
  try {
    await employeeStore.fetchEmployees()
    await payrollStore.fetchYears()

    // Eagerly load all years' data so totals show immediately
    for (const year of payrollStore.years) {
      await loadYearData(year)
    }

    // Auto-expand all years
    expandedYearItems.value = payrollStore.years.map(String)
  } catch (error) {
    appStore.showNotification(`Failed to load data: ${getErrorMessage(error)}`, 'error')
  }
})
</script>

<style scoped>
.history-layout {
  display: flex;
  gap: 12px;
  height: calc(100vh - 140px);
  overflow: hidden;
}

.history-main {
  flex: 1 1 75%;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
}

/* Prevent horizontal scrollbar on year table */
:deep(.v-data-table) {
  overflow-x: auto;
}

:deep(.v-data-table__wrapper) {
  overflow-x: auto;
}

/* Responsive: stack vertically on small screens */
@media (max-width: 959px) {
  .history-layout {
    flex-direction: column;
    height: auto;
    overflow: visible;
  }

  .history-main {
    flex: none;
    max-width: 100%;
    min-height: 0;
    overflow-y: visible;
    overflow-x: hidden;
  }
}
</style>
