<template>
  <div>
    <v-row class="align-center mb-4">
      <v-col cols="auto">
        <h1 class="text-h5">Tax Remittance</h1>
      </v-col>
      <v-spacer />
      <v-col cols="auto" class="d-flex ga-2">
        <v-btn color="primary" prepend-icon="mdi-plus" size="small" @click="showNewRemittance = true">
          New
        </v-btn>
        <v-btn variant="outlined" icon="mdi-refresh" size="small" :loading="payrollStore.loading" @click="loadYears" />
      </v-col>
    </v-row>

    <!-- Remittance Years Table -->
    <v-card v-if="payrollStore.remittanceYears.length > 0">
      <v-card-text class="pa-0">
        <v-data-table v-model:expanded="expandedYearItems" :items="yearTableItems" :headers="yearHeaders" show-expand density="compact" item-key="year" :items-per-page="-1" hide-default-footer
          :sort-by="[{ key: 'year', order: 'desc' }]">
          <template #item.year="{ item }">
            <v-icon class="mr-2" size="small">mdi-calendar</v-icon>
            <strong>{{ item.year }}</strong>
          </template>

          <template #item.count="{ item }">
            <v-chip size="small" variant="tonal">{{ item.count }}</v-chip>
          </template>

          <template #item.earnings="{ item }">
            ${{ formatAmount(item.earnings) }}
          </template>

          <template #item.cpp="{ item }">
            ${{ formatAmount(item.cpp) }}
          </template>

          <template #item.ei="{ item }">
            ${{ formatAmount(item.ei) }}
          </template>

          <template #item.lastDate="{ item }">
            {{ item.lastDate ? formatDateShort(item.lastDate, item.year) : '—' }}
          </template>

          <template #item.tax="{ item }">
            ${{ formatAmount(item.tax) }}
          </template>

          <template #item.grand="{ item }">
            <strong class="text-primary">${{ formatAmount(item.grand) }}</strong>
          </template>

          <template #expanded-row="{ columns, item }">
            <td :colspan="columns.length" class="pa-0">
              <v-card variant="flat" class="ml-4 mr-4 mb-2">
                <v-card-text>
                  <v-progress-linear v-if="loadingYear === item.year" indeterminate color="primary" class="mb-2" />

                  <v-alert v-else-if="!yearRemittances[item.year] || yearRemittances[item.year].length === 0" type="info" variant="tonal" density="compact">
                    No remittances found for {{ item.year }}.
                  </v-alert>

                  <div v-else ">
                    <v-data-table :items="yearRemittances[item.year]" :headers="remittanceHeaders" density="compact" :items-per-page="-1" hide-default-footer fixed-header
                    :sort-by="[{ key: 'period_start', order: 'desc' }]">
                    <template #item.period="{ item }">
                      {{ formatDateShort(item.period_start, parseInt(String(item.period_start).split('-')[0])) }} – {{ formatDateShort(item.period_end, parseInt(String(item.period_start).split('-')[0])) }}
                    </template>

                    <template #item.total_earnings="{ item }">
                      ${{ formatAmount(item.total_earnings) }}
                    </template>

                    <template #item.cpp_total="{ item }">
                      ${{ formatAmount(Number(item.total_cpp ?? 0) + Number(item.total_cpp2 ?? 0)) }}
                    </template>

                    <template #item.ei_total="{ item }">
                      ${{ formatAmount(item.total_ei) }}
                    </template>

                    <template #item.tax_total="{ item }">
                      ${{ formatAmount(Number(item.total_federal_tax ?? 0) + Number(item.total_provincial_tax ?? 0)) }}
                    </template>

                    <template #item.total_deductions="{ item }">
                      ${{ formatAmount(Number(item.total_cpp ?? 0) + Number(item.total_cpp2 ?? 0) + Number(item.total_ei ?? 0) + Number(item.total_federal_tax ?? 0) + Number(item.total_provincial_tax ?? 0)) }}
                    </template>

                    <template #item.grand_total="{ item }">
                      <strong>${{ formatAmount(item.grand_total) }}</strong>
                    </template>

                    <template #item.cra_report_reference="{ item }">
                      <v-chip v-if="item.cra_report_reference" size="small" color="success">
                        {{ item.cra_report_reference }}
                      </v-chip>
                      <span v-else class="text-grey">—</span>
                    </template>

                    <template #item.actions="{ item }">
                      <v-btn icon="mdi-eye" size="small" variant="text" @click="viewRemittance(item)" />
                      <v-btn icon="mdi-file-pdf-box" size="small" variant="text" color="primary" @click="generateReport(item.id!)" />
                      <v-btn v-if="appStore.devMode" icon="mdi-delete" size="small" variant="text" color="error" @click="confirmDelete(item.id!)" />
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
      No remittance records found. Click <strong>New</strong> to create a remittance.
    </v-alert>
    <v-btn color="primary" prepend-icon="mdi-plus" class="ml-4" @click="showNewRemittance = true">
      New Remittance
    </v-btn>
  </v-card-text>
</v-card>

<!-- New Remittance Dialog -->
<NewRemittanceDialog v-model="showNewRemittance" @created="onRemittanceCreated" />

<!-- Delete Confirmation Dialog -->
<v-dialog v-model="showDeleteDialog" max-width="500">
  <v-card>
    <v-card-title>Delete Remittance</v-card-title>
    <v-card-text>
      <v-alert type="warning" variant="tonal">
        Are you sure you want to delete this remittance record?
        This action cannot be undone.
      </v-alert>
    </v-card-text>
    <v-card-actions>
      <v-spacer />
      <v-btn variant="text" @click="showDeleteDialog = false">Cancel</v-btn>
      <v-btn color="error" :loading="payrollStore.loading" @click="deleteRemittance">
        Delete
      </v-btn>
    </v-card-actions>
  </v-card>
</v-dialog>

<!-- View Details Dialog -->
<v-dialog v-model="showDetailsDialog" max-width="700">
  <v-card v-if="selectedRemittance">
    <v-card-title>Remittance Details</v-card-title>
    <v-card-text>
      <v-row>
        <v-col cols="6">
          <div class="text-caption">Period Start</div>
          <div class="text-body-1">{{ formatDate(selectedRemittance.period_start) }}</div>
        </v-col>
        <v-col cols="6">
          <div class="text-caption">Period End</div>
          <div class="text-body-1">{{ formatDate(selectedRemittance.period_end) }}</div>
        </v-col>
        <v-col cols="6">
          <div class="text-caption">Generated</div>
          <div class="text-body-1">{{ formatDateTime(selectedRemittance.generated_at) }}</div>
        </v-col>
        <v-col cols="6">
          <div class="text-caption">CRA Reference</div>
          <div class="text-body-1">
            <v-chip v-if="selectedRemittance.cra_report_reference" size="small" color="success">
              {{ selectedRemittance.cra_report_reference }}
            </v-chip>
            <span v-else class="text-grey">Not provided</span>
          </div>
        </v-col>
      </v-row>

      <v-divider class="my-4" />

      <v-row dense class="mt-2">
        <v-col cols="6" sm="3">
          <v-card variant="tonal" color="info" class="px-2 py-1">
            <div class="text-caption">Earnings</div>
            <div class="text-subtitle-1 font-weight-bold">${{ formatAmount(selectedRemittance.total_earnings) }}</div>
          </v-card>
        </v-col>
        <v-col cols="6" sm="3">
          <v-card variant="tonal" color="purple" class="px-2 py-1">
            <div class="text-caption">CPP</div>
            <div class="text-subtitle-1 font-weight-bold">${{ formatAmount(Number(selectedRemittance.total_cpp ?? 0) + Number(selectedRemittance.total_cpp2 ?? 0)) }}</div>
          </v-card>
        </v-col>
        <v-col cols="6" sm="3">
          <v-card variant="tonal" color="orange" class="px-2 py-1">
            <div class="text-caption">EI</div>
            <div class="text-subtitle-1 font-weight-bold">${{ formatAmount(selectedRemittance.total_ei) }}</div>
          </v-card>
        </v-col>
        <v-col cols="6" sm="3">
          <v-card variant="tonal" color="red" class="px-2 py-1">
            <div class="text-caption">Tax</div>
            <div class="text-subtitle-1 font-weight-bold">${{ formatAmount(Number(selectedRemittance.total_federal_tax ?? 0) + Number(selectedRemittance.total_provincial_tax ?? 0)) }}</div>
          </v-card>
        </v-col>
      </v-row>
      <v-card variant="tonal" color="success" class="mt-2 px-3 py-2">
        <div class="d-flex justify-space-between align-center">
          <span class="text-body-1 font-weight-bold">Grand Total</span>
          <span class="text-h6 font-weight-bold">${{ formatAmount(selectedRemittance.grand_total) }}</span>
        </div>
      </v-card>
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
import { useDisplay } from 'vuetify'
import { useCurrentPayrollStore } from '@/stores/currentPayroll'
import { useAppStore } from '@/stores/app'
import { formatDateLocal } from '@/utils/date'
import type { Remittance } from '@/types/payroll'
import { getErrorMessage } from '@/utils/error'
import NewRemittanceDialog from '@/components/forms/NewRemittanceDialog.vue'

const payrollStore = useCurrentPayrollStore()
const appStore = useAppStore()

const showNewRemittance = ref(false)
const showDeleteDialog = ref(false)
const showDetailsDialog = ref(false)
const selectedRemittance = ref<Remittance | null>(null)
const remittanceToDelete = ref<number | null>(null)

// Expandable years state
const expandedYearItems = ref<string[]>([])
const yearRemittances = reactive<Record<number, Remittance[]>>({})
const yearTotals = reactive<Record<number, { earnings: number; cpp: number; ei: number; tax: number; grand: number }>>({})
const loadingYear = ref<number | null>(null)

const yearHeaders = [
  { title: 'Year', key: 'year' },
  { title: 'Last Date', key: 'lastDate', sortable: false },
  { title: 'Remittances', key: 'count', sortable: false },
  { title: 'Earnings', key: 'earnings' },
  { title: 'CPP', key: 'cpp' },
  { title: 'EI', key: 'ei' },
  { title: 'Tax', key: 'tax' },
  { title: 'Grand Total', key: 'grand' }
]

const yearTableItems = computed(() => {
  return payrollStore.remittanceYears.map(year => {
    const remittances = yearRemittances[year] ?? []
    // Find the latest period_end date across all remittances for this year
    let lastDate = ''
    for (const r of remittances) {
      if (r.period_end && r.period_end > lastDate) {
        lastDate = r.period_end
      }
    }
    return {
      year,
      lastDate,
      count: remittances.length,
      earnings: yearTotals[year]?.earnings ?? 0,
      cpp: yearTotals[year]?.cpp ?? 0,
      ei: yearTotals[year]?.ei ?? 0,
      tax: yearTotals[year]?.tax ?? 0,
      grand: yearTotals[year]?.grand ?? 0
    }
  })
})

const { lgAndUp, xlAndUp } = useDisplay()

const remittanceHeaders = computed(() => {
  const headers = [
    { title: 'Period', key: 'period' },
    { title: 'Emp.', key: 'total_employees', sortable: false },
    { title: 'Earnings', key: 'total_earnings' },
  ]
  if (xlAndUp.value) {
    headers.push(
      { title: 'CPP', key: 'cpp_total' },
      { title: 'EI', key: 'ei_total' },
      { title: 'Tax', key: 'tax_total' },
    )
  }
  if (lgAndUp.value) {
    headers.push(
      { title: 'Deductions', key: 'total_deductions' },
    )
  }
  headers.push(
    { title: 'Grand Total', key: 'grand_total' },
    { title: 'CRA Ref', key: 'cra_report_reference' },
    { title: 'Actions', key: 'actions', sortable: false }
  )
  return headers
})

// Watch for year expansion and load remittances on demand
watch(expandedYearItems, async (newVal, oldVal) => {
  const oldSet = new Set(oldVal)
  const opened = newVal.filter(y => !oldSet.has(y))
  for (const yearStr of opened) {
    await loadYearRemittances(Number(yearStr))
  }
}, { deep: true })

const loadYearRemittances = async (year: number) => {
  if (yearRemittances[year]) return // already loaded
  loadingYear.value = year
  try {
    await payrollStore.fetchRemittances(year)
    yearRemittances[year] = payrollStore.remittances
    // Compute year totals
    const totals = { earnings: 0, cpp: 0, ei: 0, tax: 0, grand: 0 }
    for (const r of yearRemittances[year]) {
      totals.earnings += Number(r.total_earnings ?? 0)
      totals.cpp += Number(r.total_cpp ?? 0) + Number(r.total_cpp2 ?? 0)
      totals.ei += Number(r.total_ei ?? 0)
      totals.tax += Number(r.total_federal_tax ?? 0) + Number(r.total_provincial_tax ?? 0)
      totals.grand += Number(r.grand_total ?? 0)
    }
    yearTotals[year] = totals
  } catch (error) {
    appStore.showNotification(`Failed to load remittances for ${year}: ${getErrorMessage(error)}`, 'error')
  } finally {
    loadingYear.value = null
  }
}

const loadYears = async () => {
  try {
    await payrollStore.fetchRemittanceYears()
    // Clear cached data for years no longer present
    for (const cachedYear of Object.keys(yearRemittances).map(Number)) {
      if (!payrollStore.remittanceYears.includes(cachedYear)) {
        delete yearRemittances[cachedYear]
        delete yearTotals[cachedYear]
      }
    }
  } catch (error) {
    appStore.showNotification(`Failed to load years: ${getErrorMessage(error)}`, 'error')
  }
}

const formatDate = formatDateLocal

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

const formatDateTime = (dateStr: string) => {
  if (!dateStr) return ''
  return new Date(dateStr).toLocaleString()
}

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

const onRemittanceCreated = async (cutoffDate: string) => {
  // Determine the year from the cutoff date used to create the remittance
  const createdYear = new Date(cutoffDate).getFullYear()

  // Refresh the years list
  await loadYears()

  // Clear cache for the affected year so it reloads with fresh data
  delete yearRemittances[createdYear]
  delete yearTotals[createdYear]

  // Refresh remittances for this year
  await loadYearRemittances(createdYear)

  // Auto-expand the year if not already expanded
  const isExpanded = expandedYearItems.value.some(y => Number(y) === createdYear)
  if (!isExpanded) {
    expandedYearItems.value.push(String(createdYear))
  }
}

const viewRemittance = (remittance: Remittance) => {
  selectedRemittance.value = remittance
  showDetailsDialog.value = true
}

const confirmDelete = (id: number) => {
  remittanceToDelete.value = id
  showDeleteDialog.value = true
}

const generateReport = async (id: number) => {
  try {
    await payrollStore.generateRemittanceReport(id)
  } catch (error) {
    // Error handled in store
  }
}

const deleteRemittance = async () => {
  if (!remittanceToDelete.value) return
  try {
    await payrollStore.deleteRemittance(remittanceToDelete.value)
    appStore.showNotification('Remittance deleted', 'success')
    showDeleteDialog.value = false
    remittanceToDelete.value = null
    // Clear cache and reload all expanded years
    for (const yearStr of expandedYearItems.value) {
      const year = Number(yearStr)
      delete yearRemittances[year]
      delete yearTotals[year]
    }
    const yearsToReload = [...expandedYearItems.value]
    expandedYearItems.value = []
    await loadYears()
    for (const yearStr of yearsToReload) {
      expandedYearItems.value.push(yearStr)
    }
  } catch (error) {
    appStore.showNotification(`Failed to delete remittance: ${getErrorMessage(error)}`, 'error')
  }
}

onMounted(async () => {
  await loadYears()

  // Eagerly load all years' remittance data so YTD totals show immediately
  for (const year of payrollStore.remittanceYears) {
    await loadYearRemittances(year)
  }
})
</script>
