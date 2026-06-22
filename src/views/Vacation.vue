<template>
  <div class="history-layout">
    <div class="history-main">
      <!-- Title Bar -->
      <v-row class="align-center mb-4">
        <v-col cols="auto">
          <h1 class="text-h5">Vacation Pay</h1>
        </v-col>
        <v-spacer />
        <v-col cols="auto" class="d-flex ga-2">
          <v-btn variant="outlined" icon="mdi-plus" size="small" :disabled="!selectedEmployeeId" @click="showAdjustmentDialog = true" />
          <v-btn variant="outlined" icon="mdi-beach" size="small" :disabled="!selectedEmployeeId" @click="openTimeOffCreateDialog" />
        </v-col>
      </v-row>

      <div class="history-body">
        <!-- Employee List (Left) -->
        <div class="history-employee-panel">
          <v-card>
            <v-text-field v-model="employeeSearch" density="compact" variant="outlined" placeholder="Search employees..." prepend-inner-icon="mdi-magnify" clearable hide-details class="mx-3 mt-3" />
            <v-list density="compact" nav class="py-0">
              <v-list-item v-for="emp in filteredEmployees" :key="emp.id" :active="selectedEmployeeId === emp.id" :title="`${emp.first_name} ${emp.last_name}`" :subtitle="emp.employee_number"
                @click="selectEmployee(emp)" />
            </v-list>
          </v-card>
        </div>

        <!-- Content (Right) -->
        <div class="history-content">
          <!-- No Employee Selected -->
          <v-card v-if="!selectedEmployeeId">
            <v-card-text>
              <v-alert type="info" variant="tonal" class="mb-0">
                Select an employee from the list to view their vacation pay.
              </v-alert>
            </v-card-text>
          </v-card>

          <!-- Employee Vacation Data -->
          <template v-else>
            <!-- Balance Summary -->
            <v-row class="mb-2">
              <v-col cols="12" md="3">
                <v-card variant="tonal" color="success">
                  <v-card-text class="text-center">
                    <div class="text-caption">{{ isNonHourlyEmployee ? 'Days Available' : 'Balance' }}</div>
                    <div class="text-h5 font-weight-bold" v-if="isNonHourlyEmployee">{{ Number(balanceData.balance_days ?? 0).toFixed(1) }} days</div>
                    <div class="text-h5 font-weight-bold" v-else>{{ formatCurrency(balanceData.balance) }}</div>
                    <div class="text-caption" v-if="isNonHourlyEmployee && balanceData.balance > 0">≈ {{ formatCurrency(balanceData.balance) }}</div>
                    <div class="text-caption" v-else-if="!isNonHourlyEmployee && hoursAvailable > 0">≈ {{ hoursAvailable.toFixed(1) }} hours</div>
                  </v-card-text>
                </v-card>
              </v-col>
              <v-col cols="12" md="3">
                <v-card variant="tonal" color="info">
                  <v-card-text class="text-center">
                    <div class="text-caption">Total Accrued</div>
                    <div class="text-h5 font-weight-bold">{{ formatCurrency(balanceData.total_accrued) }}</div>
                  </v-card-text>
                </v-card>
              </v-col>
              <v-col cols="12" md="3">
                <v-card variant="tonal" color="warning">
                  <v-card-text class="text-center">
                    <div class="text-caption">Total Used/Paid</div>
                    <div class="text-h5 font-weight-bold">{{ formatCurrency(balanceData.total_paid) }}</div>
                  </v-card-text>
                </v-card>
              </v-col>
              <v-col cols="12" md="3">
                <v-card variant="tonal" :color="balanceData.balance >= 0 ? 'success' : 'error'">
                  <v-card-text class="text-center">
                    <div class="text-caption">Vacation Rate</div>
                    <div class="text-h5 font-weight-bold">{{ ((selectedEmployee?.vacation_pay_rate ?? 0) * 100).toFixed(1) }}%</div>
                    <div class="text-caption" v-if="isNonHourlyEmployee">{{ Number(selectedEmployee?.vacation_pay_rate ?? 0) * 250 }} days/year</div>
                    <div class="text-caption" v-else-if="hourlyRate > 0">Rate: {{ formatCurrency(hourlyRate) }}/hr</div>
                  </v-card-text>
                </v-card>
              </v-col>
            </v-row>

            <!-- Transaction History (Year-Grouped) -->
            <v-card>
              <v-card-text class="pa-0">
                <v-data-table v-model:expanded="expandedTransactionYears" :items="transactionYearItems" :headers="transactionYearHeaders" show-expand density="compact" item-value="year"
                  :items-per-page="-1" hide-default-footer disable-sort>
                  <template #top>
                    <div class="text-subtitle-1 font-weight-bold pa-4 pb-0">
                      <v-icon size="small" class="mr-1">mdi-history</v-icon>
                      Transaction History
                    </div>
                  </template>

                  <template #item.year="{ item }">
                    <v-icon class="mr-2" size="small">mdi-calendar</v-icon>
                    <strong>{{ item.year }}</strong>
                  </template>

                  <template #item.count="{ item }">
                    <v-chip size="small" variant="tonal">{{ item.count }}</v-chip>
                  </template>

                  <template #item.totalAccrued="{ item }">
                    <span class="text-success">{{ formatCurrency(item.totalAccrued) }}</span>
                  </template>

                  <template #item.totalDaysAccrued="{ item }">
                    <span class="text-success">{{ Number(item.totalDaysAccrued).toFixed(1) }}d</span>
                  </template>

                  <template #item.totalPaid="{ item }">
                    <span class="text-warning">{{ formatCurrency(item.totalPaid) }}</span>
                  </template>

                  <template #item.totalDaysPaid="{ item }">
                    <span class="text-warning">{{ Number(item.totalDaysPaid).toFixed(1) }}d</span>
                  </template>

                  <template #item.netChange="{ item }">
                    <strong :class="item.netChange >= 0 ? 'text-success' : 'text-error'">
                      {{ formatCurrency(item.netChange) }}
                    </strong>
                  </template>

                  <!-- Expanded Row: Individual Transactions -->
                  <template #expanded-row="{ columns, item }">
                    <td :colspan="columns.length" class="pa-0">
                      <v-card variant="flat" class="ml-4 mr-4 mb-2">
                        <v-card-text>
                          <v-alert v-if="!yearTransactions[item.year] || yearTransactions[item.year].length === 0" type="info" variant="tonal" density="compact">
                            No transactions found for {{ item.year }}.
                          </v-alert>

                          <v-data-table v-else :items="yearTransactions[item.year]" :headers="transactionDetailHeaders" density="compact" :items-per-page="-1" hide-default-footer fixed-header
                            disable-sort>
                            <template #item.accrual_date="{ item }">
                              {{ formatDate(item.accrual_date) }}
                            </template>
                            <template #item.transaction_type="{ item }">
                              <v-chip :color="getTransactionColor(item.transaction_type)" size="small" variant="tonal">
                                {{ item.transaction_type }}
                              </v-chip>
                            </template>
                            <template #item.amount="{ item }">
                              <span :class="Number(item.amount) >= 0 ? 'text-success' : 'text-error'">
                                {{ formatCurrency(Number(item.amount)) }}
                              </span>
                            </template>
                            <template #item.amount_days="{ item }">
                              <span :class="Number(item.amount_days) >= 0 ? 'text-success' : 'text-error'">
                                {{ Number(item.amount_days).toFixed(1) }}d
                              </span>
                            </template>
                            <template #item.balance_after="{ item }">
                              {{ formatCurrency(Number(item.balance_after)) }}
                            </template>
                            <template #item.balance_after_days="{ item }">
                              {{ Number(item.balance_after_days).toFixed(1) }}d
                            </template>
                            <template #item.notes="{ item }">
                              {{ item.notes || '-' }}
                            </template>
                          </v-data-table>
                        </v-card-text>
                      </v-card>
                    </td>
                  </template>
                </v-data-table>
              </v-card-text>
            </v-card>

            <!-- Time Off History (Year-Grouped) -->
            <v-card v-if="timeOffHistory.length > 0">
              <v-card-text class="pa-0">
                <v-data-table v-model:expanded="expandedTimeOffYears" :items="timeOffYearItems" :headers="timeOffYearHeaders" show-expand density="compact" item-value="year" :items-per-page="-1"
                  hide-default-footer disable-sort>
                  <template #top>
                    <div class="text-subtitle-1 font-weight-bold pa-4 pb-0">
                      <v-icon size="small" class="mr-1">mdi-calendar-clock</v-icon>
                      Time Off
                    </div>
                  </template>

                  <template #item.year="{ item }">
                    <v-icon class="mr-2" size="small">mdi-calendar</v-icon>
                    <strong>{{ item.year }}</strong>
                  </template>

                  <template #item.count="{ item }">
                    <v-chip size="small" variant="tonal">{{ item.count }}</v-chip>
                  </template>

                  <template #item.totalPaid="{ item }">
                    <span class="text-success">{{ formatCurrency(item.totalPaid) }}</span>
                  </template>

                  <template #item.totalUnpaid="{ item }">
                    <span class="text-grey">{{ item.totalUnpaid }}</span>
                  </template>

                  <!-- Expanded Row: Individual Time Off Records -->
                  <template #expanded-row="{ columns, item }">
                    <td :colspan="columns.length" class="pa-0">
                      <v-card variant="flat" class="ml-4 mr-4 mb-2">
                        <v-card-text>
                          <v-alert v-if="!yearTimeOffs[item.year] || yearTimeOffs[item.year].length === 0" type="info" variant="tonal" density="compact">
                            No time off records found for {{ item.year }}.
                          </v-alert>

                          <v-data-table v-else :items="yearTimeOffs[item.year]" :headers="timeOffDetailHeaders" density="compact" :items-per-page="-1" hide-default-footer fixed-header disable-sort>
                            <template #item.start_date="{ item }">
                              {{ formatDate(item.start_date) }}
                            </template>
                            <template #item.end_date="{ item }">
                              {{ formatDate(item.end_date) }}
                            </template>
                            <template #item.pay_type="{ item }">
                              <v-chip :color="item.pay_type === 'paid' ? 'success' : 'grey'" size="small" variant="tonal">
                                {{ item.pay_type }}
                              </v-chip>
                            </template>
                            <template #item.estimated_payout="{ item }">
                              <span class="text-grey">{{ formatCurrency(Number(item.estimated_payout)) }}</span>
                            </template>
                            <template #item.payout_amount="{ item }">
                              <strong>{{ formatCurrency(Number(item.payout_amount)) }}</strong>
                            </template>
                            <template #item.days_taken="{ item }">
                              <strong>{{ Number(item.days_taken ?? 0) }} days</strong>
                            </template>
                            <template #item.actions="{ item }">
                              <v-btn icon="mdi-pencil" size="x-small" variant="text" @click="openTimeOffEditDialog(item)" />
                              <v-btn icon="mdi-delete" size="x-small" variant="text" color="error" @click="confirmDeleteTimeOff(item)" />
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
            <v-card v-if="transactions.length === 0 && timeOffHistory.length === 0 && !loadingHistory">
              <v-card-text class="d-flex align-center">
                <v-alert type="info" variant="tonal" class="flex-grow-1 mb-0">
                  No vacation records found for this employee.
                </v-alert>
              </v-card-text>
            </v-card>
          </template>
        </div>
      </div>
    </div>

    <!-- Adjustment Dialog -->
    <v-dialog v-model="showAdjustmentDialog" max-width="400">
      <v-card>
        <v-card-title>Record Vacation Adjustment – {{ selectedEmployee?.first_name }} {{ selectedEmployee?.last_name }}</v-card-title>
        <v-card-text>
          <v-text-field v-model.number="adjustmentAmount" label="Amount ($)" type="number" step="0.01" prefix="$" hint="Positive to add, negative to deduct" persistent-hint variant="outlined"
            class="mb-3" />
          <v-text-field v-if="isNonHourlyEmployee" v-model.number="adjustmentDays" label="Days" type="number" step="0.5" hint="Positive to add, negative to deduct (independent of dollar amount)"
            persistent-hint variant="outlined" class="mb-3" />
          <v-text-field v-model="adjustmentNotes" label="Notes" variant="outlined" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showAdjustmentDialog = false">Cancel</v-btn>
          <v-btn color="primary" @click="submitAdjustment" :loading="submitting">Save</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Time Off Create Dialog -->
    <v-dialog v-model="showTimeOffDialog" max-width="500">
      <v-card>
        <v-card-title>New Time Off – {{ selectedEmployee?.first_name }} {{ selectedEmployee?.last_name }}</v-card-title>
        <v-card-text>
          <v-row>
            <v-col cols="6">
              <v-text-field v-model="timeOffStart" label="Start Date" type="date" variant="outlined" />
            </v-col>
            <v-col cols="6">
              <v-text-field v-model="timeOffEnd" label="End Date" type="date" variant="outlined" />
            </v-col>
          </v-row>
          <v-row>
            <v-col cols="6">
              <v-text-field :model-value="timeOffPayTypeDisplay" label="Pay Type" variant="outlined" readonly density="compact" />
            </v-col>
            <v-col cols="6" v-if="isNonHourlyEmployee">
              <v-text-field :model-value="timeOffDays" label="Days" type="number" variant="outlined" readonly density="compact" hint="Auto-calculated from dates (weekdays)" persistent-hint />
            </v-col>
            <v-col cols="6" v-else>
              <v-text-field :model-value="0" label="Payout Amount" type="number" step="0.01" prefix="$" variant="outlined" readonly hint="Always $0 for hourly employees" persistent-hint />
            </v-col>
          </v-row>
          <v-text-field v-model="timeOffNotes" label="Notes" variant="outlined" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showTimeOffDialog = false">Cancel</v-btn>
          <v-btn color="primary" @click="submitTimeOff" :loading="submitting">Submit</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Time Off Edit Dialog -->
    <v-dialog v-model="showTimeOffEditDialog" max-width="500">
      <v-card v-if="editingTimeOff">
        <v-card-title>Edit Time Off – {{ selectedEmployee?.first_name }} {{ selectedEmployee?.last_name }}</v-card-title>
        <v-card-text>
          <v-row>
            <v-col cols="6">
              <v-text-field v-model="editTimeOffStart" label="Start Date" type="date" variant="outlined" />
            </v-col>
            <v-col cols="6">
              <v-text-field v-model="editTimeOffEnd" label="End Date" type="date" variant="outlined" />
            </v-col>
          </v-row>
          <v-row>
            <v-col cols="6">
              <v-text-field :model-value="editingTimeOff.pay_type" label="Pay Type" variant="outlined" readonly density="compact" />
            </v-col>
            <v-col cols="6" v-if="isNonHourlyEmployee">
              <v-text-field :model-value="editTimeOffDays" label="Days" type="number" variant="outlined" readonly density="compact" hint="Auto-calculated from dates (weekdays)" persistent-hint />
            </v-col>
            <v-col cols="6" v-else>
              <v-text-field v-model.number="editTimeOffPayoutAmount" label="Payout Amount" type="number" step="0.01" prefix="$" variant="outlined" />
            </v-col>
          </v-row>
          <v-text-field v-model="editTimeOffNotes" label="Notes" variant="outlined" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showTimeOffEditDialog = false">Cancel</v-btn>
          <v-btn color="primary" @click="submitTimeOffEdit" :loading="submitting">Save</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="showDeleteConfirm" max-width="400">
      <v-card>
        <v-card-title>Delete Time Off?</v-card-title>
        <v-card-text>
          This will permanently delete the time off record
          <strong v-if="deletingTimeOff?.pay_type === 'paid'">and reverse the vacation accrual transaction</strong>.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showDeleteConfirm = false">Cancel</v-btn>
          <v-btn color="error" @click="submitDeleteTimeOff" :loading="submitting">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { vacationApi } from '@/services/api'
import { useEmployeeStore } from '@/stores/employee'
import { useAppStore } from '@/stores/app'
import type { Employee } from '@/types/employee'
import { formatDateLocal, parseLocalDate } from '@/utils/date'

const employeeStore = useEmployeeStore()
const appStore = useAppStore()

// --- State ---
const selectedEmployeeId = ref<number | null>(null)
const selectedEmployee = ref<Employee | null>(null)
const employeeSearch = ref('')
const balanceData = ref({ balance: 0, balance_days: 0, total_accrued: 0, total_paid: 0 })
const transactions = ref<any[]>([])
const timeOffHistory = ref<any[]>([])
const loadingHistory = ref(false)
const submitting = ref(false)

// Year expansion state - Transactions
const expandedTransactionYears = ref<string[]>([])
const yearTransactions = ref<Record<number, any[]>>({})

// Year expansion state - Time Off
const expandedTimeOffYears = ref<string[]>([])
const yearTimeOffs = ref<Record<number, any[]>>({})

// Adjustment dialog
const showAdjustmentDialog = ref(false)
const adjustmentAmount = ref(0)
const adjustmentDays = ref<number | null>(null)
const adjustmentNotes = ref('')

// Time off create dialog
const showTimeOffDialog = ref(false)
const timeOffStart = ref('')
const timeOffEnd = ref('')
const timeOffPayoutAmount = ref(0)
const timeOffNotes = ref('')

// Time off edit dialog
const showTimeOffEditDialog = ref(false)
const editingTimeOff = ref<any>(null)
const editTimeOffStart = ref('')
const editTimeOffEnd = ref('')
const editTimeOffPayoutAmount = ref(0)
const editTimeOffNotes = ref('')

// Delete confirmation
const showDeleteConfirm = ref(false)
const deletingTimeOff = ref<any>(null)

// --- Computed ---
const filteredEmployees = computed(() => {
  const list = employeeStore.employees.filter(e => e.is_active)
  if (!employeeSearch.value) return list
  const search = employeeSearch.value.toLowerCase()
  return list.filter(e =>
    e.first_name.toLowerCase().includes(search) ||
    e.last_name.toLowerCase().includes(search) ||
    e.employee_number.toLowerCase().includes(search)
  )
})

const isHourlyEmployee = computed(() => {
  return selectedEmployee.value?.pay_type === 'Hourly'
})

const isNonHourlyEmployee = computed(() => {
  return selectedEmployee.value?.pay_type !== 'Hourly'
})

const timeOffPayTypeDisplay = computed(() => {
  return isHourlyEmployee.value ? 'Unpaid' : 'Paid'
})

const hourlyRate = computed(() => {
  if (!selectedEmployee.value) return 0
  const payType = selectedEmployee.value.pay_type
  const payRate = Number(selectedEmployee.value.pay_rate ?? 0)
  if (payRate <= 0) return 0
  if (payType === 'Hourly') return payRate
  if (payType === 'Weekly') return payRate / 40
  if (payType === 'Monthly') return (payRate * 12) / 2080
  if (payType === 'Annual') return payRate / 2080
  return payRate
})

const hoursAvailable = computed(() => {
  if (hourlyRate.value <= 0) return 0
  return balanceData.value.balance / hourlyRate.value
})

/** Auto-calculated weekdays for time off create */
const timeOffDays = computed(() => {
  return countWeekdays(timeOffStart.value, timeOffEnd.value)
})

/** Auto-calculated weekdays for time off edit */
const editTimeOffDays = computed(() => {
  return countWeekdays(editTimeOffStart.value, editTimeOffEnd.value)
})

/** Count weekdays between two dates (inclusive) */
const countWeekdays = (startStr: string, endStr: string): number => {
  if (!startStr || !endStr) return 0
  const start = parseLocalDate(startStr)
  const end = parseLocalDate(endStr)
  if (end < start) return 0
  let count = 0
  const current = new Date(start)
  while (current <= end) {
    const day = current.getDay()
    if (day >= 1 && day <= 5) count++
    current.setDate(current.getDate() + 1)
  }
  return count
}

/** Reset adjustment form when dialog opens */
watch(showAdjustmentDialog, (open) => {
  if (open) {
    adjustmentAmount.value = 0
    adjustmentDays.value = null
    adjustmentNotes.value = ''
  }
})


/** The auto-calculated payout estimate (readonly reference) */
const timeOffPayoutEstimate = ref(0)

/** Auto-calculate payout estimate when dates change */
const recalcPayoutEstimate = () => {
  if (isHourlyEmployee.value) {
    timeOffPayoutEstimate.value = 0
    timeOffPayoutAmount.value = 0
    return
  }
  // Non-hourly: only track days, no dollar values shown
  timeOffPayoutEstimate.value = 0
  timeOffPayoutAmount.value = 0
}

// Watch for date changes to auto-calculate
watch([timeOffStart, timeOffEnd], () => {
  if (!isHourlyEmployee.value) {
    recalcPayoutEstimate()
  }
})

/** Extract year from a date string (YYYY-MM-DD) */
const extractYear = (dateStr: string): number => {
  if (!dateStr) return 0
  return parseInt(dateStr.split('-')[0], 10)
}

/** Get unique years from transactions, sorted descending */
const transactionYears = computed(() => {
  const years = new Set<number>()
  for (const t of transactions.value) {
    const year = extractYear(t.accrual_date)
    if (year) years.add(year)
  }
  return Array.from(years).sort((a, b) => b - a)
})

/** Year table items for transactions */
const transactionYearItems = computed(() => {
  return transactionYears.value.map(year => {
    const txns = yearTransactions.value[year] ?? []
    let totalAccrued = 0
    let totalPaid = 0
    let totalDaysAccrued = 0
    let totalDaysPaid = 0
    for (const t of txns) {
      const amt = Number(t.amount ?? 0)
      const days = Number(t.amount_days ?? 0)
      if (t.transaction_type === 'accrue') {
        totalAccrued += amt
        totalDaysAccrued += days
      } else if (t.transaction_type === 'payout' || t.transaction_type === 'timeoff') {
        totalPaid += Math.abs(amt)
        totalDaysPaid += Math.abs(days)
      } else if (t.transaction_type === 'adjust') {
        // Positive adjustments increase balance (like accrual), negative decrease (like payout)
        if (amt >= 0) {
          totalAccrued += amt
          totalDaysAccrued += days
        } else {
          totalPaid += Math.abs(amt)
          totalDaysPaid += Math.abs(days)
        }
      }
    }
    const netChange = totalAccrued - totalPaid
    return {
      year,
      count: txns.length,
      totalAccrued,
      totalPaid,
      totalDaysAccrued,
      totalDaysPaid,
      netChange
    }
  })
})

/** Get unique years from time off, sorted descending */
const timeOffYears = computed(() => {
  const years = new Set<number>()
  for (const t of timeOffHistory.value) {
    const year = extractYear(t.start_date)
    if (year) years.add(year)
  }
  return Array.from(years).sort((a, b) => b - a)
})

/** Year table items for time off */
const timeOffYearItems = computed(() => {
  return timeOffYears.value.map(year => {
    const items = yearTimeOffs.value[year] ?? []
    let totalPaid = 0
    let unpaidCount = 0
    for (const t of items) {
      if (t.pay_type === 'paid') {
        totalPaid += Number(t.payout_amount ?? 0)
      } else {
        unpaidCount++
      }
    }
    return {
      year,
      count: items.length,
      totalPaid,
      totalUnpaid: unpaidCount > 0 ? `${unpaidCount} unpaid` : '-'
    }
  })
})

// --- Table Headers ---
const transactionYearHeaders = computed(() => {
  const headers = [
    { title: 'Year', key: 'year' },
    { title: 'Transactions', key: 'count', sortable: false },
    { title: 'Accrued', key: 'totalAccrued' },
    { title: 'Deducted', key: 'totalPaid' },
    { title: 'Net Change', key: 'netChange' },
  ]
  if (isNonHourlyEmployee.value) {
    headers.splice(2, 0, { title: 'Days Accrued', key: 'totalDaysAccrued' })
    headers.splice(4, 0, { title: 'Days Used', key: 'totalDaysPaid' })
  }
  return headers
})

const transactionDetailHeaders = computed(() => {
  const headers = [
    { title: 'Date', key: 'accrual_date' },
    { title: 'Type', key: 'transaction_type' },
    { title: 'Amount', key: 'amount' },
    { title: 'Balance', key: 'balance_after' },
    { title: 'Notes', key: 'notes' },
  ]
  if (isNonHourlyEmployee.value) {
    headers.splice(2, 0, { title: 'Days', key: 'amount_days' })
    headers.splice(4, 0, { title: 'Days Bal', key: 'balance_after_days' })
  }
  return headers
})

const timeOffYearHeaders = [
  { title: 'Year', key: 'year' },
  { title: 'Requests', key: 'count', sortable: false },
  { title: 'Deducted', key: 'totalPaid' },
  { title: 'Unpaid', key: 'totalUnpaid', sortable: false },
]

const timeOffDetailHeaders = computed(() => {
  if (isNonHourlyEmployee.value) {
    return [
      { title: 'Start', key: 'start_date' },
      { title: 'End', key: 'end_date' },
      { title: 'Type', key: 'pay_type' },
      { title: 'Days', key: 'days_taken' },
      { title: '', key: 'actions', sortable: false },
    ]
  }
  return [
    { title: 'Start', key: 'start_date' },
    { title: 'End', key: 'end_date' },
    { title: 'Type', key: 'pay_type' },
    { title: 'Estimate', key: 'estimated_payout' },
    { title: 'Amount', key: 'payout_amount' },
    { title: '', key: 'actions', sortable: false },
  ]
})

// --- Formatting Helpers ---
const formatCurrency = (value: any): string => {
  const num = Number(value ?? 0)
  if (Number.isNaN(num)) return '$0.00'
  return num.toLocaleString('en-CA', { style: 'currency', currency: 'CAD' })
}

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  return formatDateLocal(dateStr)
}

const getTransactionColor = (type: string) => {
  switch (type) {
    case 'accrue': return 'success'
    case 'payout': return 'warning'
    case 'adjust': return 'info'
    case 'timeoff': return 'error'
    default: return 'grey'
  }
}

// --- Data Loading ---
const loadEmployeeData = async (employeeId: number) => {
  loadingHistory.value = true
  try {
    const [balance, history, timeOff] = await Promise.all([
      vacationApi.getBalance(employeeId),
      vacationApi.getHistory(employeeId),
      vacationApi.getTimeOffHistory(employeeId),
    ])
    balanceData.value = balance
    transactions.value = history
    timeOffHistory.value = timeOff

    // Group transactions by year
    groupTransactionsByYear()
    // Group time off by year
    groupTimeOffByYear()

    // Expand all years by default
    expandedTransactionYears.value = transactionYears.value.map(String)
    expandedTimeOffYears.value = timeOffYears.value.map(String)
  } catch (error) {
    console.error('Failed to load vacation data:', error)
  } finally {
    loadingHistory.value = false
  }
}

/** Group transactions into year buckets */
const groupTransactionsByYear = () => {
  const grouped: Record<number, any[]> = {}
  for (const t of transactions.value) {
    const year = extractYear(t.accrual_date)
    if (year) {
      if (!grouped[year]) grouped[year] = []
      grouped[year].push(t)
    }
  }
  yearTransactions.value = grouped
}

/** Group time off requests into year buckets */
const groupTimeOffByYear = () => {
  const grouped: Record<number, any[]> = {}
  for (const t of timeOffHistory.value) {
    const year = extractYear(t.start_date)
    if (year) {
      if (!grouped[year]) grouped[year] = []
      grouped[year].push(t)
    }
  }
  yearTimeOffs.value = grouped
}

const selectEmployee = async (emp: Employee) => {
  selectedEmployeeId.value = emp.id ?? null
  selectedEmployee.value = emp
  if (emp.id) {
    await loadEmployeeData(emp.id)
  }
}

// --- Actions ---

// Adjustment
const submitAdjustment = async () => {
  if (!selectedEmployee.value?.id) return
  // Require at least one non-zero value
  const hasAmount = adjustmentAmount.value !== 0
  const hasDays = isNonHourlyEmployee.value && adjustmentDays.value !== null && adjustmentDays.value !== 0
  if (!hasAmount && !hasDays) return
  submitting.value = true
  try {
    // Both values are independent — pass as-is
    const daysValue = isNonHourlyEmployee.value && adjustmentDays.value != null && adjustmentDays.value !== 0
      ? adjustmentDays.value : null
    await vacationApi.recordAdjustment(selectedEmployee.value.id, adjustmentAmount.value, daysValue, adjustmentNotes.value || null)
    showAdjustmentDialog.value = false
    await loadEmployeeData(selectedEmployee.value.id)
    appStore.showNotification('Vacation adjustment recorded', 'success')
  } catch (error) {
    appStore.showNotification('Failed to record adjustment', 'error')
  } finally {
    submitting.value = false
  }
}

// Time Off Create
const openTimeOffCreateDialog = () => {
  timeOffStart.value = ''
  timeOffEnd.value = ''
  timeOffPayoutEstimate.value = 0
  timeOffPayoutAmount.value = 0
  timeOffNotes.value = ''
  showTimeOffDialog.value = true
}

const submitTimeOff = async () => {
  if (!selectedEmployee.value?.id || !timeOffStart.value || !timeOffEnd.value) return
  submitting.value = true
  try {
    // Non-hourly employees: no dollar payout, only days tracked
    const payoutForBackend = isNonHourlyEmployee.value ? 0 : timeOffPayoutAmount.value
    const estimateForBackend = isNonHourlyEmployee.value ? 0 : timeOffPayoutEstimate.value
    await vacationApi.createTimeOff(
      selectedEmployee.value.id,
      timeOffStart.value,
      timeOffEnd.value,
      estimateForBackend,
      payoutForBackend,
      timeOffNotes.value || null
    )
    showTimeOffDialog.value = false
    await loadEmployeeData(selectedEmployee.value.id)
    appStore.showNotification('Time off recorded', 'success')
  } catch (error) {
    appStore.showNotification('Failed to record time off', 'error')
  } finally {
    submitting.value = false
  }
}

// Time Off Edit
const openTimeOffEditDialog = (item: any) => {
  editingTimeOff.value = item
  editTimeOffStart.value = item.start_date
  editTimeOffEnd.value = item.end_date
  editTimeOffPayoutAmount.value = Number(item.payout_amount ?? 0)
  editTimeOffNotes.value = item.notes || ''
  showTimeOffEditDialog.value = true
}

const submitTimeOffEdit = async () => {
  if (!editingTimeOff.value?.id) return
  submitting.value = true
  try {
    // Non-hourly employees: no dollar payout, only days tracked
    const payoutForBackend = isNonHourlyEmployee.value ? 0 : editTimeOffPayoutAmount.value
    await vacationApi.updateTimeOff(
      editingTimeOff.value.id,
      editTimeOffStart.value,
      editTimeOffEnd.value,
      payoutForBackend,
      editTimeOffNotes.value || null
    )
    showTimeOffEditDialog.value = false
    editingTimeOff.value = null
    if (selectedEmployee.value?.id) {
      await loadEmployeeData(selectedEmployee.value.id)
    }
    appStore.showNotification('Time off updated', 'success')
  } catch (error) {
    appStore.showNotification('Failed to update time off', 'error')
  } finally {
    submitting.value = false
  }
}

// Time Off Delete
const confirmDeleteTimeOff = (item: any) => {
  deletingTimeOff.value = item
  showDeleteConfirm.value = true
}

const submitDeleteTimeOff = async () => {
  if (!deletingTimeOff.value?.id) return
  submitting.value = true
  try {
    await vacationApi.deleteTimeOff(deletingTimeOff.value.id)
    showDeleteConfirm.value = false
    deletingTimeOff.value = null
    if (selectedEmployee.value?.id) {
      await loadEmployeeData(selectedEmployee.value.id)
    }
    appStore.showNotification('Time off deleted', 'success')
  } catch (error) {
    appStore.showNotification('Failed to delete time off', 'error')
  } finally {
    submitting.value = false
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
  overflow-x: hidden;
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
