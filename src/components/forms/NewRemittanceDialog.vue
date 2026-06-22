<template>
  <v-dialog :model-value="modelValue" @update:model-value="$emit('update:modelValue', $event)" max-width="700" persistent>
    <v-card>
      <v-card-title class="d-flex align-center">
        <v-icon class="mr-2">mdi-plus-circle</v-icon>
        New Remittance
        <v-spacer />
        <v-btn icon="mdi-close" size="small" variant="text" @click="close" />
      </v-card-title>

      <v-card-text>
        <p class="text-body-2 mb-4">
          Create a remittance for all unfiled paid payrolls before a specific date.
        </p>

        <v-select v-model="selectedMonth" :items="monthOptions" item-title="label" item-value="value" label="Cutoff Month" hint="Include all paid payrolls before the end of the selected month"
          persistent-hint variant="outlined" density="compact" class="mb-4" />

        <v-expand-transition>
          <v-text-field v-if="selectedMonth === 'custom'" v-model="customDate" label="Cutoff Date" type="date" hint="Select the exact cutoff date" persistent-hint variant="outlined" density="compact"
            class="mb-4" />
        </v-expand-transition>

        <v-btn color="primary" block :loading="loading" :disabled="!cutoffDate" @click="fetchSummary">
          Calculate Remittance
        </v-btn>

        <!-- Remittance Summary -->
        <v-card v-if="summary" variant="tonal" class="mt-4">
          <v-card-title>Remittance Summary</v-card-title>
          <v-card-text>
            <v-alert v-if="summary.unfiled_payrolls_count === 0" type="info" variant="tonal">
              No unfiled payrolls found before {{ formatDate(cutoffDate) }}
            </v-alert>

            <template v-else>
              <div class="mb-4">
                <div class="text-caption">Period</div>
                <div class="text-h6">
                  {{ formatDate(summary.period_start) }} -
                  {{ formatDate(summary.period_end) }}
                </div>
              </div>

              <v-divider class="my-4" />

              <v-list density="compact">
                <v-list-item>
                  <template #prepend>
                    <v-icon>mdi-file-document-outline</v-icon>
                  </template>
                  <v-list-item-title>Payrolls Included</v-list-item-title>
                  <template #append>
                    <v-chip size="small">
                      {{ summary.unfiled_payrolls_count }}
                    </v-chip>
                  </template>
                </v-list-item>

                <v-list-item>
                  <template #prepend>
                    <v-icon>mdi-cash</v-icon>
                  </template>
                  <v-list-item-title>Earnings</v-list-item-title>
                  <template #append>
                    <strong>${{ formatAmount(summary.total_earnings) }}</strong>
                  </template>
                </v-list-item>

                <v-list-item>
                  <template #prepend>
                    <v-icon>mdi-account-cash</v-icon>
                  </template>
                  <v-list-item-title>CPP (CPP + CPP2)</v-list-item-title>
                  <template #append>
                    <strong>${{ formatAmount(Number(summary.total_cpp ?? 0) + Number(summary.total_cpp2 ?? 0)) }}</strong>
                  </template>
                </v-list-item>

                <v-list-item>
                  <template #prepend>
                    <v-icon>mdi-shield-account</v-icon>
                  </template>
                  <v-list-item-title>EI (Employee + Employer)</v-list-item-title>
                  <template #append>
                    <strong>${{ formatAmount(summary.total_ei) }}</strong>
                  </template>
                </v-list-item>

                <v-list-item>
                  <template #prepend>
                    <v-icon>mdi-cash-multiple</v-icon>
                  </template>
                  <v-list-item-title>Tax (Federal + Provincial)</v-list-item-title>
                  <template #append>
                    <strong>${{ formatAmount(Number(summary.total_federal_tax ?? 0) + Number(summary.total_provincial_tax ?? 0)) }}</strong>
                  </template>
                </v-list-item>

                <v-divider class="my-2" />

                <v-list-item class="bg-primary">
                  <template #prepend>
                    <v-icon color="white">mdi-cash-check</v-icon>
                  </template>
                  <v-list-item-title class="text-white font-weight-bold">
                    Grand Total
                  </v-list-item-title>
                  <template #append>
                    <strong class="text-white text-h6">
                      ${{ formatAmount(summary.grand_total) }}
                    </strong>
                  </template>
                </v-list-item>
              </v-list>

              <v-btn color="success" block class="mt-4" @click="showFinalizeConfirm = true">
                Finalize Remittance
              </v-btn>
            </template>
          </v-card-text>
        </v-card>
      </v-card-text>
    </v-card>

    <!-- Finalize Confirmation -->
    <v-dialog v-model="showFinalizeConfirm" max-width="500">
      <v-card>
        <v-card-title>Finalize Remittance</v-card-title>
        <v-card-text>
          <v-alert type="info" variant="tonal" class="mb-4">
            Enter the CRA confirmation number to finalize this remittance.
            This will create a permanent record.
          </v-alert>

          <v-text-field v-model="craConfirmation" label="CRA Confirmation Number (Optional)" hint="Enter the confirmation number from CRA after payment" persistent-hint variant="outlined"
            density="compact" />

          <div v-if="summary" class="mt-4">
            <div class="text-h6">Summary</div>
            <div class="text-body-2">
              Period: {{ formatDate(summary.period_start) }} -
              {{ formatDate(summary.period_end) }}
            </div>
            <div class="text-h5 mt-2">
              Total: ${{ formatAmount(summary.grand_total) }}
            </div>
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showFinalizeConfirm = false">Cancel</v-btn>
          <v-btn color="success" :loading="loading" @click="finalize">
            Create Remittance
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useCurrentPayrollStore } from '@/stores/currentPayroll'
import { useAppStore } from '@/stores/app'
import { formatDateLocal } from '@/utils/date'
import type { RemittanceSummary } from '@/types/payroll'
import { getErrorMessage } from '@/utils/error'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'created': [cutoffDate: string]
}>()

const payrollStore = useCurrentPayrollStore()
const appStore = useAppStore()

const cutoffDate = ref('')
const selectedMonth = ref('')
const customDate = ref('')
const craConfirmation = ref('')
const loading = ref(false)
const summary = ref<RemittanceSummary | null>(null)
const showFinalizeConfirm = ref(false)

const formatDate = formatDateLocal

// Generate month options for the last 24 months plus a custom date option
const monthOptions = computed(() => {
  const options: { label: string; value: string }[] = []
  const today = new Date()
  for (let i = 0; i < 24; i++) {
    const d = new Date(today.getFullYear(), today.getMonth() - i, 1)
    const year = d.getFullYear()
    const month = d.getMonth()
    const lastDay = new Date(year, month + 1, 0)
    const label = d.toLocaleDateString('en-US', { year: 'numeric', month: 'long' })
    const value = lastDay.toISOString().split('T')[0]
    options.push({ label, value })
  }
  options.push({ label: 'Custom Date...', value: 'custom' })
  return options
})

// When a month is selected, update the cutoff date to the end of that month
watch(selectedMonth, (val) => {
  if (val && val !== 'custom') {
    cutoffDate.value = val
  } else if (val === 'custom') {
    cutoffDate.value = customDate.value
  }
})

// When custom date is picked, update the cutoff date
watch(customDate, (val) => {
  if (selectedMonth.value === 'custom' && val) {
    cutoffDate.value = val
  }
})

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

// Set default cutoff date to end of last month when dialog opens
watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    const today = new Date()
    // Day 0 of current month = last day of previous month
    const endOfLastMonth = new Date(today.getFullYear(), today.getMonth(), 0)
    const defaultValue = endOfLastMonth.toISOString().split('T')[0]
    cutoffDate.value = defaultValue
    selectedMonth.value = defaultValue
    customDate.value = ''
  }
})

const fetchSummary = async () => {
  loading.value = true
  try {
    summary.value = await payrollStore.fetchRemittanceSummary(cutoffDate.value)
    if (summary.value) {
      if (summary.value.unfiled_payrolls_count > 0) {
        appStore.showNotification(
          `Found ${summary.value.unfiled_payrolls_count} unfiled payroll(s)`,
          'success'
        )
      } else {
        appStore.showNotification('No unfiled payrolls found', 'info')
      }
    }
  } catch (error) {
    appStore.showNotification(`Failed to fetch summary: ${getErrorMessage(error)}`, 'error')
  } finally {
    loading.value = false
  }
}

const finalize = async () => {
  loading.value = true
  try {
    await payrollStore.createRemittance(
      cutoffDate.value,
      craConfirmation.value || undefined
    )
    appStore.showNotification('Remittance created successfully', 'success')
    const emittedCutoff = cutoffDate.value
    close()
    emit('created', emittedCutoff)
  } catch (error) {
    appStore.showNotification(`Failed to create remittance: ${getErrorMessage(error)}`, 'error')
  } finally {
    loading.value = false
  }
}

const close = () => {
  showFinalizeConfirm.value = false
  summary.value = null
  craConfirmation.value = ''
  emit('update:modelValue', false)
}
</script>
