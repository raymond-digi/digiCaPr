<template>
  <div>
    <!-- Title Bar -->
    <v-row class="align-center mb-4">
      <v-col cols="auto">
        <h1 class="text-h5">T4 Slips</h1>
      </v-col>
      <v-spacer />
      <v-col cols="auto" class="d-flex ga-2">
        <v-btn color="primary" prepend-icon="mdi-plus" size="small" @click="showNewT4Dialog = true">
          New
        </v-btn>
        <v-btn variant="outlined" icon="mdi-refresh" size="small" :loading="t4Store.loading" @click="loadYears" />
      </v-col>
    </v-row>

    <!-- T4 Years Table -->
    <v-card v-if="t4Store.availableYears.length > 0">
      <v-card-text class="pa-0">
        <v-data-table v-model:expanded="expandedYearItems" :items="yearTableItems" :headers="yearHeaders" show-expand density="compact" item-key="year" :items-per-page="-1" hide-default-footer
          disable-sort>
          <template #item.year="{ item }">
            <v-icon class="mr-2" size="small">mdi-calendar</v-icon>
            <strong>{{ item.year }}</strong>
          </template>

          <template #item.slip_count="{ item }">
            <v-chip size="small" variant="tonal">{{ item.slip_count }}</v-chip>
          </template>

          <template #item.employment_income="{ item }">
            ${{ formatAmount(item.employment_income) }}
          </template>

          <template #item.total_deductions="{ item }">
            <span :class="Math.abs(Number(item.total_deductions) - Number(item.remittances_paid)) > 0.01 ? 'text-error font-weight-bold' : ''">
              ${{ formatAmount(item.total_deductions) }}
            </span>
          </template>

          <template #item.remittances_paid="{ item }">
            ${{ formatAmount(item.remittances_paid) }}
          </template>

          <template #item.actions="{ item }">
            <v-btn icon="mdi-file-pdf-box" size="small" variant="text" color="primary" :loading="loadingYear === item.year" @click="generateSummaryPdf(item.year)" />
            <v-btn icon="mdi-calculator" size="small" variant="text" color="primary" :loading="loadingYear === item.year" @click="openTotalsDialog(item)" />
          </template>

          <template #expanded-row="{ columns, item }">
            <td :colspan="columns.length" class="pa-0">
              <v-card variant="flat" class="ml-4 mr-4 mb-2">
                <v-card-text>
                  <v-progress-linear v-if="loadingYear === item.year" indeterminate color="primary" class="mb-2" />

                  <v-alert v-else-if="!yearSlips[item.year] || yearSlips[item.year].length === 0" type="info" variant="tonal" density="compact">
                    No T4 slips found for {{ item.year }}.
                  </v-alert>

                  <div v-else>
                    <div class="d-flex align-center mb-2">
                      <span class="text-subtitle-2">T4 Slips ({{ yearSlips[item.year].length }})</span>
                      <v-spacer />
                      <v-btn size="x-small" variant="outlined" prepend-icon="mdi-file-pdf-box" class="mr-2" :loading="t4Store.loading" @click="generateAllPdfs(item.year)">
                        Generate All PDFs
                      </v-btn>
                      <v-menu :close-on-content-click="!exporting">
                        <template #activator="{ props }">
                          <v-btn size="x-small" variant="outlined" prepend-icon="mdi-download" v-bind="props" :loading="exporting">
                            Export
                          </v-btn>
                        </template>
                        <v-list density="compact">
                          <v-tooltip :disabled="canExportXml" location="bottom">
                            <template #activator="{ props: tooltipProps }">
                              <v-list-item prepend-icon="mdi-xml" title="Export XML (T619)" @click="exportXml" :disabled="exporting || !canExportXml" v-bind="tooltipProps" />
                            </template>
                            <span>{{ xmlExportDisabledReason }}</span>
                          </v-tooltip>
                          <v-list-item prepend-icon="mdi-file-delimited" title="Export CSV" @click="exportCsv" :disabled="exporting" />
                          <v-divider />
                          <v-list-item prepend-icon="mdi-cog" title="Transmitter Settings..." @click="showTransmitterDialog = true" />
                        </v-list>
                      </v-menu>
                    </div>

                    <v-data-table :items="yearSlips[item.year]" :headers="slipHeaders" density="compact" :items-per-page="-1" hide-default-footer fixed-header disable-sort>
                      <template #item.employee_name="{ item }">
                        {{ item.employee.first_name }} {{ item.employee.last_name }}
                      </template>

                      <template #item.employee_number="{ item }">
                        {{ item.employee.employee_number }}
                      </template>

                      <template #item.employment_income="{ item }">
                        {{ formatCurrency(item.employment_income) }}
                      </template>

                      <template #item.cpp_contributions="{ item }">
                        <span :class="hasCppDiscrepancy(item) ? 'text-error font-weight-bold' : ''">
                          {{ formatCurrency(item.cpp_contributions) }}
                        </span>
                      </template>

                      <template #item.ei_premiums="{ item }">
                        <span :class="hasEiDiscrepancy(item) ? 'text-error font-weight-bold' : ''">
                          {{ formatCurrency(item.ei_premiums) }}
                        </span>
                      </template>

                      <template #item.income_tax_deducted="{ item }">
                        {{ formatCurrency(item.income_tax_deducted) }}
                      </template>

                      <template #item.computed_net_pay="{ item }">
                        <span :class="hasDiscrepancy(item) ? 'text-error font-weight-bold' : ''">
                          {{ formatCurrency(item.computed_net_pay) }}
                        </span>
                      </template>

                      <template #item.actions="{ item }">
                        <v-btn icon="mdi-file-pdf-box" size="x-small" variant="text" color="primary" :loading="t4Store.loading" @click="generateSinglePdf(item)" />
                        <v-btn icon="mdi-cog" size="x-small" variant="text" @click="openAdjustmentDialog(item)" />
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
    <v-card v-else-if="!t4Store.loading">
      <v-card-text class="d-flex align-center">
        <v-alert type="info" variant="tonal" class="flex-grow-1 mb-0">
          No T4 records found. Click <strong>New</strong> to calculate T4 slips for a year.
        </v-alert>
        <v-btn color="primary" prepend-icon="mdi-plus" class="ml-4" @click="showNewT4Dialog = true">
          New T4s
        </v-btn>
      </v-card-text>
    </v-card>

    <!-- Loading -->
    <v-card v-if="t4Store.loading && t4Store.availableYears.length === 0" class="mt-4">
      <v-card-text class="text-center">
        <v-progress-circular indeterminate color="primary" />
        <div class="mt-2 text-body-2">Loading...</div>
      </v-card-text>
    </v-card>

    <!-- New T4 Dialog (Year Selection + Calculate) -->
    <v-dialog v-model="showNewT4Dialog" max-width="500">
      <v-card>
        <v-card-title class="d-flex align-center">
          <v-icon class="mr-2">mdi-plus-circle</v-icon>
          Calculate T4 Slips
          <v-spacer />
          <v-btn icon="mdi-close" size="small" variant="text" @click="showNewT4Dialog = false" />
        </v-card-title>
        <v-card-text>
          <p class="text-body-2 mb-4">
            Calculate year-end T4 values for all employees based on payroll history.
          </p>
          <v-select v-model="selectedYear" :items="yearOptions" label="Tax Year" variant="outlined" density="compact" class="mb-4" />
          <v-btn color="primary" block :loading="t4Store.loading" :disabled="!selectedYear" @click="calculateT4s">
            Calculate T4s
          </v-btn>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showNewT4Dialog = false">Cancel</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Totals Dialog (Calculator icon) -->
    <v-dialog v-model="showTotalsDialog" max-width="700">
      <v-card v-if="selectedYearTotals">
        <v-card-title class="d-flex align-center">
          <v-icon class="mr-2">mdi-calculator</v-icon>
          T4 Summary - {{ selectedYearTotals.year }}
          <v-spacer />
          <v-tooltip text="Recalculate T4 values from payroll history (overwrites current data)" location="bottom">
            <template #activator="{ props }">
              <v-btn variant="tonal" color="warning" prepend-icon="mdi-calculator" size="small" :loading="t4Store.loading" v-bind="props"
                @click="showTotalsDialog = false; showRecalculateConfirm = true">
                Recalculate
              </v-btn>
            </template>
          </v-tooltip>
        </v-card-title>
        <v-card-text>
          <v-row>
            <v-col cols="6" sm="4">
              <v-card variant="tonal" color="primary" class="px-2 py-1">
                <div class="text-caption">Slips</div>
                <div class="text-subtitle-1 font-weight-bold">{{ selectedYearTotals.slip_count }}</div>
              </v-card>
            </v-col>
            <v-col cols="6" sm="4">
              <v-card variant="tonal" :color="Math.abs(Number(selectedYearTotals.total_deductions) - Number(selectedYearTotals.remittances_paid)) > 0.01 ? 'error' : 'warning'" class="px-2 py-1">
                <div class="text-caption">Total Deductions</div>
                <div class="text-subtitle-1 font-weight-bold">${{ formatAmount(selectedYearTotals.total_deductions) }}</div>
              </v-card>
            </v-col>
            <v-col cols="6" sm="4">
              <v-card variant="tonal" color="success" class="px-2 py-1">
                <div class="text-caption">Remittances Paid</div>
                <div class="text-subtitle-1 font-weight-bold">${{ formatAmount(selectedYearTotals.remittances_paid) }}</div>
              </v-card>
            </v-col>
          </v-row>

          <v-divider class="my-4" />

          <!-- Summary Table -->
          <v-table v-if="yearSummaryData[selectedYearTotals.year]" density="compact">
            <thead>
              <tr>
                <th>Box</th>
                <th>Description</th>
                <th class="text-right">Amount</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>14</td>
                <td>Employment income</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_employment_income) }}</td>
              </tr>
              <tr>
                <td>16</td>
                <td>Employee's CPP contributions</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_employee_cpp) }}</td>
              </tr>
              <tr>
                <td>16a</td>
                <td>Employee's CPP2 contributions</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_employee_cpp2) }}</td>
              </tr>
              <tr>
                <td>27</td>
                <td>Employer's CPP contributions</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_employer_cpp) }}</td>
              </tr>
              <tr>
                <td>27a</td>
                <td>Employer's CPP2 contributions</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_employer_cpp2) }}</td>
              </tr>
              <tr>
                <td>18</td>
                <td>Employee's EI premiums</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_employee_ei) }}</td>
              </tr>
              <tr>
                <td>19</td>
                <td>Employer's EI premiums</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_employer_ei) }}</td>
              </tr>
              <tr>
                <td>20</td>
                <td>RPP contributions</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_rpp_contributions) }}</td>
              </tr>
              <tr>
                <td>52</td>
                <td>Pension adjustment</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_pension_adjustment) }}</td>
              </tr>
              <tr>
                <td>22</td>
                <td>Income tax deducted</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_income_tax) }}</td>
              </tr>
              <tr>
                <td>80</td>
                <td>Total deductions</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_deductions_reported) }}</td>
              </tr>
              <tr>
                <td>82</td>
                <td>Remittances paid</td>
                <td class="text-right">${{ formatAmount(yearSummaryData[selectedYearTotals.year].total_remittances_paid) }}</td>
              </tr>
            </tbody>
          </v-table>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showTotalsDialog = false">Close</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Recalculate Confirmation Dialog -->
    <v-dialog v-model="showRecalculateConfirm" max-width="500">
      <v-card>
        <v-card-title class="d-flex align-center">
          <v-icon icon="mdi-alert" color="warning" class="mr-2" />
          Recalculate T4 Data
        </v-card-title>
        <v-card-text>
          <p>
            This will recalculate all T4 values for <strong>{{ t4Store.selectedYear }}</strong> from payroll history data.
          </p>
          <v-alert type="warning" variant="tonal" class="mt-3">
            This will overwrite any existing values. Are you sure?
          </v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showRecalculateConfirm = false">Cancel</v-btn>
          <v-btn color="warning" :loading="t4Store.loading" @click="confirmRecalculate">
            <v-icon icon="mdi-calculator" class="mr-1" />
            Recalculate
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Adjustment Dialog -->
    <T4AdjustmentForm v-model="showAdjustmentDialog" :slip="selectedSlip" @save="saveAdjustment" />

    <!-- Transmitter Settings Dialog -->
    <TransmitterForm v-model="showTransmitterDialog" @save="onTransmitterSaved" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useDisplay } from 'vuetify'
import { useT4Store } from '@/stores/t4'
import { useAppStore } from '@/stores/app'
import { payrollApi, reportsApi, registryApi, employeeApi } from '@/services/api'
import { getErrorMessage } from '@/utils/error'
import type { T4SlipLegacy as T4Slip, T4SummaryData } from '@/types/t4'
import { save } from '@tauri-apps/plugin-dialog'
import T4AdjustmentForm from '@/components/forms/T4AdjustmentForm.vue'
import TransmitterForm from '@/components/forms/TransmitterForm.vue'

const t4Store = useT4Store()
const appStore = useAppStore()

// Dialog states
const showNewT4Dialog = ref(false)
const showTotalsDialog = ref(false)
const showAdjustmentDialog = ref(false)
const showRecalculateConfirm = ref(false)
const showTransmitterDialog = ref(false)

// Selection states
const selectedYear = ref<number | null>(null)
const selectedSlip = ref<T4Slip | null>(null)
const selectedYearTotals = ref<{ year: number; slip_count: number; employment_income: number; total_deductions: number; remittances_paid: number } | null>(null)
const exporting = ref(false)
const transmitterInfoValid = ref<boolean | null>(null)

// Expandable years state
const expandedYearItems = ref<string[]>([])
const yearSlips = reactive<Record<number, T4Slip[]>>({})
const yearSummaryData = reactive<Record<number, T4SummaryData>>({})
const loadingYear = ref<number | null>(null)

// Payroll history years that don't have T4 data yet (for the "New" dropdown)
const payrollHistoryYears = ref<number[]>([])
const yearOptions = computed(() => {
  return payrollHistoryYears.value.filter(year => !t4Store.availableYears.includes(year))
})

const { lgAndUp } = useDisplay()

// Year table headers
const yearHeaders = [
  { title: 'Year', key: 'year' },
  { title: 'Slips', key: 'slip_count', sortable: false },
  { title: 'Employment Income', key: 'employment_income' },
  { title: 'Total Deductions', key: 'total_deductions' },
  { title: 'Remittances Paid', key: 'remittances_paid' },
  { title: 'Actions', key: 'actions', sortable: false }
]

// Computed year table items
const yearTableItems = computed(() => {
  return t4Store.availableYears.map(year => {
    const slips = yearSlips[year] ?? []
    const summary = yearSummaryData[year]
    return {
      year,
      slip_count: slips.length,
      employment_income: slips.reduce((sum, s) => sum + Number(s.employment_income ?? 0), 0),
      total_deductions: summary?.total_deductions_reported ?? 0,
      remittances_paid: summary?.total_remittances_paid ?? 0
    }
  })
})

// Slip table headers (responsive)
const slipHeaders = computed(() => {
  const headers = [
    { title: 'Code', key: 'employee_number' },
    { title: 'Name', key: 'employee_name', sortable: false },
    { title: 'Earnings', key: 'employment_income', align: 'end' as const }
  ]
  // if (xlAndUp.value) {
  headers.push(
    { title: 'CPP', key: 'cpp_contributions', align: 'end' as const },
    { title: 'CPP2', key: 'cpp2_contributions', align: 'end' as const },
    { title: 'EI', key: 'ei_premiums', align: 'end' as const },
    // { title: 'Box 22', key: 'income_tax_deducted', align: 'end' as const }
  )
  // }
  if (lgAndUp.value) {
    headers.push(
      { title: 'Net Pay', key: 'computed_net_pay', align: 'end' as const }
    )
  }
  headers.push(
    { title: 'Actions', key: 'actions', sortable: false }
  )
  return headers
})

// Computed: check if any slips have net pay errors
const hasNetPayErrors = computed(() => {
  for (const year of Object.keys(yearSlips).map(Number)) {
    if (yearSlips[year].some(slip => hasDiscrepancy(slip))) return true
  }
  return false
})

// Computed: whether XML export is allowed
const canExportXml = computed(() => {
  return transmitterInfoValid.value === true && !hasNetPayErrors.value
})

// Computed: reason why XML export is disabled
const xmlExportDisabledReason = computed(() => {
  const reasons: string[] = []
  if (transmitterInfoValid.value !== true) {
    reasons.push('Transmitter info (BN15 and name) is not set')
  }
  if (hasNetPayErrors.value) {
    reasons.push('One or more slips have net pay errors')
  }
  return reasons.join('; ') || 'Cannot export XML'
})

// Watch for year expansion and load slips on demand
watch(expandedYearItems, async (newVal, oldVal) => {
  const oldSet = new Set(oldVal)
  const opened = newVal.filter(y => !oldSet.has(y))
  for (const yearVal of opened) {
    const yearNum = typeof yearVal === 'number' ? yearVal : parseInt(String(yearVal), 10)
    await loadYearSlips(yearNum)
  }
}, { deep: true })

// Check if transmitter BN15 and name are set in registry
const checkTransmitterInfo = async () => {
  try {
    const bn15 = await registryApi.getString('transmitter/bn15')
    const name = await registryApi.getString('transmitter/name')
    transmitterInfoValid.value = !!(bn15 && bn15.trim() && name && name.trim())
  } catch {
    transmitterInfoValid.value = false
  }
}

// Load T4 slips for a specific year
const loadYearSlips = async (year: number) => {
  if (!year || isNaN(year)) return
  if (yearSlips[year]) return // already loaded
  loadingYear.value = year
  try {
    await t4Store.loadT4sForYear(year)
    yearSlips[year] = t4Store.t4Slips
    // Cache summary data
    if (t4Store.summaryData) {
      yearSummaryData[year] = t4Store.summaryData
    }
    // Load tax rates for validation
    await loadTaxRates(year)
  } catch (error) {
    appStore.showNotification(`Failed to load T4s for ${year}: ${getErrorMessage(error)}`, 'error')
  } finally {
    loadingYear.value = null
  }
}

// Load all available years
const loadYears = async () => {
  try {
    await t4Store.fetchYears()
    payrollHistoryYears.value = await payrollApi.listPayrollYears()
    // Clear cached data for years no longer present
    for (const cachedYear of Object.keys(yearSlips).map(Number)) {
      if (!t4Store.availableYears.includes(cachedYear)) {
        delete yearSlips[cachedYear]
        delete yearSummaryData[cachedYear]
      }
    }
  } catch (error) {
    appStore.showNotification(`Failed to load years: ${getErrorMessage(error)}`, 'error')
  }
}

// Calculate T4s for a year
const calculateT4s = async () => {
  if (!selectedYear.value) return
  try {
    await t4Store.calculateForYear(selectedYear.value)
    appStore.showNotification(
      `Calculated T4s for ${t4Store.employeeCount} employee(s)`,
      'success'
    )
    showNewT4Dialog.value = false
    selectedYear.value = null
    await loadYears()
  } catch (error) {
    appStore.showNotification(`Failed to calculate T4s: ${getErrorMessage(error)}`, 'error')
  }
}

// Show totals dialog for a year
const openTotalsDialog = async (item: { year: number; slip_count: number; employment_income: number; total_deductions: number; remittances_paid: number }) => {
  const year = item.year
  // Load summary data if not cached
  if (!yearSummaryData[year]) {
    loadingYear.value = year
    try {
      const summary = await reportsApi.getT4Summary(year)
      yearSummaryData[year] = summary
    } catch (error) {
      appStore.showNotification(`Failed to load summary: ${getErrorMessage(error)}`, 'error')
    } finally {
      loadingYear.value = null
    }
  }
  selectedYearTotals.value = {
    year,
    slip_count: item.slip_count,
    employment_income: item.employment_income,
    total_deductions: item.total_deductions,
    remittances_paid: item.remittances_paid
  }
  // Set selectedYear so recalculate flow works
  t4Store.selectedYear = year
  showTotalsDialog.value = true
}

// Confirm recalculate
const confirmRecalculate = async () => {
  showRecalculateConfirm.value = false
  if (!t4Store.selectedYear) return
  try {
    await t4Store.calculateForYear(t4Store.selectedYear)
    appStore.showNotification(
      `Recalculated T4s for ${t4Store.employeeCount} employee(s)`,
      'success'
    )
  } catch (error) {
    appStore.showNotification(`Failed to recalculate T4s: ${getErrorMessage(error)}`, 'error')
  }
}

// Format helpers
const formatAmount = (amount: any) => {
  let numValue
  if (typeof amount === 'string') {
    numValue = parseFloat(amount)
  } else if (typeof amount === 'object' && amount !== null && typeof amount.toString === 'function') {
    numValue = parseFloat(amount.toString())
  } else if (typeof amount === 'number') {
    numValue = amount
  } else {
    return '0.00'
  }
  if (isNaN(numValue)) {
    return '0.00'
  }
  return numValue.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

const formatCurrency = (value: any): string => {
  const num = Number(value ?? 0)
  if (Number.isNaN(num)) return '$0.00'
  return num.toLocaleString('en-CA', { style: 'currency', currency: 'CAD' })
}

const hasDiscrepancy = (item: any): boolean => {
  return Math.abs((item.computed_net_pay ?? 0) - (item.net_pay ?? 0)) > 0.01
}

// ==================== CPP/EI Validation ====================

/** Tolerance for rounding differences (in dollars) */
const VALIDATION_TOLERANCE = 0.50

/** Cached tax rates by year */
const taxRatesCache = reactive<Record<number, {
  cpp_employee_rate: number;
  cpp_basic_exemption: number;
  cpp_ympe: number;
  cpp2_rate: number;
  ei_rate: number;
  ei_max_contribution: number;
} | null>>({})

/** Load tax rates for a year if not cached */
const loadTaxRates = async (year: number) => {
  if (taxRatesCache[year] !== undefined) return
  try {
    const rates = await employeeApi.getTaxRates(year)
    taxRatesCache[year] = rates
  } catch {
    taxRatesCache[year] = null
  }
}

/** Check if CPP contribution has a discrepancy from expected value */
const hasCppDiscrepancy = (item: any): boolean => {
  const rates = taxRatesCache[item.year]
  if (!rates) return false
  // cpp_pensionable_earnings (Box 26) already has the $3,500 exemption subtracted
  const pensionable = item.cpp_pensionable_earnings ?? 0
  const actual = item.cpp_contributions ?? 0
  const rate = rates.cpp_employee_rate
  const expected = pensionable * rate
  return Math.abs(actual - expected) > VALIDATION_TOLERANCE
}

/** Check if EI premium has a discrepancy from expected value */
const hasEiDiscrepancy = (item: any): boolean => {
  const rates = taxRatesCache[item.year]
  if (!rates) return false
  const insurable = item.ei_insurable_earnings ?? 0
  const actual = item.ei_premiums ?? 0
  const rate = rates.ei_rate
  const expected = insurable * rate
  return Math.abs(actual - expected) > VALIDATION_TOLERANCE
}

// PDF generation
const generateSinglePdf = async (slip: T4Slip) => {
  try {
    const path = await save({
      defaultPath: `T4_${slip.year}_${slip.employee.employee_number}_${slip.employee.last_name}.pdf`,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    })
    if (path) {
      await t4Store.generateT4Pdf(slip.employee.id!, slip.year, path)
    }
  } catch (error) {
    // Error handled in store
  }
}

const generateAllPdfs = async (year: number) => {
  try {
    const dir = await save({
      defaultPath: `T4_${year}`,
    })
    if (dir) {
      await t4Store.generateAllT4Pdfs(dir)
    }
  } catch (error) {
    // Error handled in store
  }
}

const generateSummaryPdf = async (year: number) => {
  try {
    const path = await save({
      defaultPath: `T4_Summary_${year}.pdf`,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    })
    if (path) {
      await t4Store.generateT4SummaryPdf(year, path)
    }
  } catch (error) {
    // Error handled in store
  }
}

// Export
const exportXml = async () => {
  await checkTransmitterInfo()
  if (!transmitterInfoValid.value) {
    appStore.showNotification(
      'Cannot export XML: Transmitter info (BN15 and name) is not set. Please configure transmitter settings first.',
      'error'
    )
    return
  }
  if (hasNetPayErrors.value) {
    appStore.showNotification(
      'Cannot export XML: One or more slips have net pay errors. Please fix the discrepancies before exporting.',
      'error'
    )
    return
  }
  try {
    const path = await save({
      defaultPath: `T4_${t4Store.selectedYear}.xml`,
      filters: [{ name: 'XML', extensions: ['xml'] }],
    })
    if (path) {
      exporting.value = true
      await t4Store.exportXml(t4Store.selectedYear!, path)
    }
  } catch (error) {
    // Error handled in store
  } finally {
    exporting.value = false
  }
}

const exportCsv = async () => {
  try {
    const path = await save({
      defaultPath: `T4_${t4Store.selectedYear}.csv`,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    })
    if (path) {
      exporting.value = true
      await t4Store.exportCsv(t4Store.selectedYear!, path)
    }
  } catch (error) {
    // Error handled in store
  } finally {
    exporting.value = false
  }
}

const onTransmitterSaved = async () => {
  await checkTransmitterInfo()
  appStore.showNotification('Transmitter settings saved successfully', 'success')
}

// Adjustment dialog
const openAdjustmentDialog = async (slip: T4Slip) => {
  selectedSlip.value = slip
  showAdjustmentDialog.value = true
}

const saveAdjustment = async (adjustments: Record<string, number>) => {
  if (!selectedSlip.value) return
  try {
    const slip = selectedSlip.value
    await t4Store.updateBoxValues({
      employee_id: slip.employee.id!,
      year: slip.year,
      box_14_adjustment: adjustments['box_14'] ?? 0,
      box_16_adjustment: adjustments['box_16'] ?? 0,
      box_16a_adjustment: adjustments['box_16a'] ?? 0,
      box_18_adjustment: adjustments['box_18'] ?? 0,
      box_20_adjustment: adjustments['box_20'] ?? 0,
      box_22_adjustment: adjustments['box_22'] ?? 0,
      box_24_adjustment: adjustments['box_24'] ?? 0,
      box_26_adjustment: adjustments['box_26'] ?? 0,
      box_45_adjustment: adjustments['box_45'] ?? 0,
      box_52_adjustment: adjustments['box_52'] ?? 0,
    })
    appStore.showNotification('Adjustment saved', 'success')
    showAdjustmentDialog.value = false
    // Refresh slips for this year
    if (t4Store.selectedYear) {
      delete yearSlips[t4Store.selectedYear]
      delete yearSummaryData[t4Store.selectedYear]
      await loadYearSlips(t4Store.selectedYear)
    }
  } catch (error) {
    appStore.showNotification(`Failed to save adjustment: ${getErrorMessage(error)}`, 'error')
  }
}

onMounted(async () => {
  await checkTransmitterInfo()
  await loadYears()

  // Eagerly load all years' slip data so totals show immediately
  for (const year of t4Store.availableYears) {
    await loadYearSlips(year)
  }
})
</script>
