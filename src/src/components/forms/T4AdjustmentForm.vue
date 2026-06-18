<template>
  <v-dialog v-model="dialog" max-width="1100" max-height="90vh">
    <v-card v-if="slip" class="fill-height d-flex flex-column">
      <v-card-title class="flex-shrink-0">
        Update T4 Box Values - {{ slip.employee.first_name }} {{ slip.employee.last_name }}
      </v-card-title>
      <v-card-text class="flex-grow-1 overflow-y-auto">
        <!-- Net Pay Balance Check -->
        <v-card variant="tonal" :color="netPayHasDiscrepancy ? 'warning' : 'success'" class="mt-4 mb-4">
          <v-card-text>
            <div class="d-flex align-center">
              <v-icon :icon="netPayHasDiscrepancy ? 'mdi-alert' : 'mdi-check-circle'" class="mr-2" />
              <span class="text-subtitle-2">Net Pay Verification</span>
            </div>
            <v-row dense class="mt-2">
              <v-col cols="3">
                <div class="text-caption">Net Paid (Payroll)</div>
                <div class="text-body-1 font-weight-bold">{{ formatCurrency(originalNetPay) }}</div>
              </v-col>
              <v-col cols="3">
                <div class="text-caption">Original Computed</div>
                <div class="text-body-1">{{ formatCurrency(slip?.computed_net_pay ?? 0) }}</div>
              </v-col>
              <v-col cols="3">
                <div class="text-caption">New Computed</div>
                <div class="text-body-1">{{ formatCurrency(adjustedNetPay) }}</div>
              </v-col>
              <v-col cols="3">
                <div class="text-caption">Discrepancy</div>
                <div class="text-body-1" :class="netPayHasDiscrepancy ? 'text-error' : 'text-success'">
                  {{ formatCurrency(adjustedNetPay - originalNetPay) }}
                </div>
              </v-col>
            </v-row>
            <v-alert v-if="netPayHasDiscrepancy" type="warning" variant="tonal" density="compact" class="mt-2">
              Computed net pay (${{ formatAmount(adjustedNetPay) }}) differs from payroll net paid (${{ formatAmount(originalNetPay) }}) by ${{ formatAmount(adjustedNetPay - originalNetPay) }}. Adjust
              income or deductions to match.
            </v-alert>
          </v-card-text>
        </v-card>

        <!-- CPP/CPP2/EI Validation Cards -->
        <div v-if="cppValidation || cpp2Validation || eiValidation" class="mb-4">
          <div class="text-subtitle-2 mb-2">
            <v-icon size="small" class="mr-1">mdi-calculator</v-icon>
            Deduction Verification
          </div>

          <!-- Tax Rates Error -->
          <v-alert v-if="taxRatesError" type="warning" variant="tonal" density="compact" class="mb-2">
            Unable to load tax rates: {{ taxRatesError }}
          </v-alert>

          <!-- CPP Validation -->
          <v-card v-if="cppValidation" variant="tonal" :color="cppValidation.isOk ? 'success' : 'error'" class="mb-2">
            <v-card-text class="py-2">
              <div class="d-flex align-center">
                <v-icon :icon="cppValidation.isOk ? 'mdi-check-circle' : 'mdi-alert-circle'" size="small" class="mr-2" />
                <span class="text-subtitle-2">{{ cppValidation.label }}</span>
                <v-spacer />
                <v-chip v-if="cppValidation.isOk" size="x-small" color="success" variant="flat">OK</v-chip>
                <v-chip v-else size="x-small" color="error" variant="flat">
                  Diff: ${{ formatAmount(cppValidation.difference) }}
                </v-chip>
              </div>
              <div class="text-caption mt-1">{{ cppValidation.description }}</div>
              <v-row dense class="mt-1">
                <v-col cols="4">
                  <div class="text-caption">Actual (Box 16)</div>
                  <div class="text-body-2 font-weight-bold">${{ formatAmount(cppValidation.actual) }}</div>
                </v-col>
                <v-col cols="4">
                  <div class="text-caption">Expected</div>
                  <div class="text-body-2">${{ formatAmount(cppValidation.expected) }}</div>
                </v-col>
                <v-col cols="4">
                  <div class="text-caption">Rate</div>
                  <div class="text-body-2">{{ (taxRates!.cpp_employee_rate * 100).toFixed(2) }}%</div>
                </v-col>
              </v-row>
            </v-card-text>
          </v-card>

          <!-- CPP2 Validation -->
          <v-card v-if="cpp2Validation" variant="tonal" :color="cpp2Validation.isOk ? 'success' : 'error'" class="mb-2">
            <v-card-text class="py-2">
              <div class="d-flex align-center">
                <v-icon :icon="cpp2Validation.isOk ? 'mdi-check-circle' : 'mdi-alert-circle'" size="small" class="mr-2" />
                <span class="text-subtitle-2">{{ cpp2Validation.label }}</span>
                <v-spacer />
                <v-chip v-if="cpp2Validation.isOk" size="x-small" color="success" variant="flat">OK</v-chip>
                <v-chip v-else size="x-small" color="error" variant="flat">
                  Diff: ${{ formatAmount(cpp2Validation.difference) }}
                </v-chip>
              </div>
              <div class="text-caption mt-1">{{ cpp2Validation.description }}</div>
              <v-row dense class="mt-1">
                <v-col cols="4">
                  <div class="text-caption">Actual (Box 16a)</div>
                  <div class="text-body-2 font-weight-bold">${{ formatAmount(cpp2Validation.actual) }}</div>
                </v-col>
                <v-col cols="4">
                  <div class="text-caption">Expected</div>
                  <div class="text-body-2">${{ formatAmount(cpp2Validation.expected) }}</div>
                </v-col>
                <v-col cols="4">
                  <div class="text-caption">Rate</div>
                  <div class="text-body-2">{{ (taxRates!.cpp2_rate * 100).toFixed(2) }}%</div>
                </v-col>
              </v-row>
            </v-card-text>
          </v-card>

          <!-- EI Validation -->
          <v-card v-if="eiValidation" variant="tonal" :color="eiValidation.isOk ? 'success' : 'error'" class="mb-2">
            <v-card-text class="py-2">
              <div class="d-flex align-center">
                <v-icon :icon="eiValidation.isOk ? 'mdi-check-circle' : 'mdi-alert-circle'" size="small" class="mr-2" />
                <span class="text-subtitle-2">{{ eiValidation.label }}</span>
                <v-spacer />
                <v-chip v-if="eiValidation.isOk" size="x-small" color="success" variant="flat">OK</v-chip>
                <v-chip v-else size="x-small" color="error" variant="flat">
                  Diff: ${{ formatAmount(eiValidation.difference) }}
                </v-chip>
              </div>
              <div class="text-caption mt-1">{{ eiValidation.description }}</div>
              <v-row dense class="mt-1">
                <v-col cols="4">
                  <div class="text-caption">Actual (Box 18)</div>
                  <div class="text-body-2 font-weight-bold">${{ formatAmount(eiValidation.actual) }}</div>
                </v-col>
                <v-col cols="4">
                  <div class="text-caption">Expected</div>
                  <div class="text-body-2">${{ formatAmount(eiValidation.expected) }}</div>
                </v-col>
                <v-col cols="4">
                  <div class="text-caption">Rate</div>
                  <div class="text-body-2">{{ (taxRates!.ei_rate * 100).toFixed(2) }}%</div>
                </v-col>
              </v-row>
            </v-card-text>
          </v-card>
        </div>

        <v-row>
          <v-col v-for="box in t4BoxTypes" :key="box.box_code" cols="6" sm="4" md="3" xl="3">
            <div class="text-caption mb-1">Box {{ box.box_number }} - {{ box.display_name }}</div>
            <div class="text-body-2 text-medium-emphasis mb-1">
              Calculated: {{ box.box_code === 'box_45' ? formatDentalBenefit(getOriginalCalculatedValue(box.box_code)) : `$${formatAmount(getOriginalCalculatedValue(box.box_code))}` }}
            </div>
            <!-- <div v-if="getSlipBoxValue(box.box_code) !== getOriginalCalculatedValue(box.box_code)" class="text-body-2 text-medium-emphasis mb-1">
              Current: {{ box.box_code === 'box_45' ? formatDentalBenefit(getSlipBoxValue(box.box_code)) : `$${formatAmount(getSlipBoxValue(box.box_code))}` }}
            </div> -->
            <v-select v-if="box.box_code === 'box_45'" v-model="adjustmentForm[box.box_code]" :items="dentalBenefitOptions" label="New Value" variant="outlined" density="compact"
              hint="Employer-offered dental benefit code" persistent-hint />
            <v-text-field v-else v-model.number="adjustmentForm[box.box_code]" type="number" variant="outlined" density="compact"
              :hint="`Diff from calculated: $${formatAmount((adjustmentForm[box.box_code] ?? 0) - getOriginalCalculatedValue(box.box_code))}`" persistent-hint />
          </v-col>
        </v-row>
      </v-card-text>
      <v-card-actions class="flex-shrink-0">
        <v-spacer />
        <v-btn variant="text" @click="dialog = false"> Cancel </v-btn>
        <v-btn color="primary" :loading="saving" @click="handleSave"> Save </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { T4_BOX_TYPES, type T4SlipLegacy as T4Slip, type T4BoxValue } from '@/types/t4'
import { t4Api, employeeApi } from '@/services/api'

const props = defineProps<{
  modelValue: boolean
  slip: T4Slip | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', adjustments: Record<string, number>): void
}>()

const dialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const saving = ref(false)
const originalCalculatedValues = ref<T4BoxValue[]>([])
const t4BoxTypes = T4_BOX_TYPES

/** Tax rates for validation (loaded from config) */
interface TaxRates {
  cpp_employee_rate: number
  cpp_basic_exemption: number
  cpp_ympe: number
  cpp_max_contribution: number
  cpp2_rate: number
  cpp2_max_earnings: number
  cpp2_max_contribution: number
  ei_rate: number
  ei_max_insurable_earnings: number
  ei_max_contribution: number
}
const taxRates = ref<TaxRates | null>(null)
const taxRatesError = ref<string | null>(null)

const dentalBenefitOptions = [
  { title: 'No dental benefit', value: 1 },
  { title: 'Basic dental coverage', value: 2 },
  { title: 'Comprehensive dental coverage', value: 3 },
]

const adjustmentForm = ref<Record<string, number>>(
  Object.fromEntries(T4_BOX_TYPES.map(b => [b.box_code, 0]))
)

/** Map box_code to the current value from a specific slip */
const getSlipBoxValueFor = (slip: T4Slip, boxCode: string): number => {
  const s = slip as any
  switch (boxCode) {
    case 'box_14': return s.employment_income ?? 0
    case 'box_16': return s.cpp_contributions ?? 0
    case 'box_16a': return s.cpp2_contributions ?? 0
    case 'box_18': return s.ei_premiums ?? 0
    case 'box_20': return s.rpp_contributions ?? 0
    case 'box_22': return s.income_tax_deducted ?? 0
    case 'box_24': return s.ei_insurable_earnings ?? 0
    case 'box_26': return s.cpp_pensionable_earnings ?? 0
    case 'box_45': return s.dental_benefit ?? 1
    case 'box_52': return s.pension_adjustment ?? 0
    default: return 0
  }
}

/** Map box_code to the current value from the selected slip */
const getSlipBoxValue = (boxCode: string): number => {
  if (!props.slip) return 0
  return getSlipBoxValueFor(props.slip, boxCode)
}

/** Get the original calculated value (before adjustments) from the database */
const getOriginalCalculatedValue = (boxCode: string): number => {
  const bv = originalCalculatedValues.value.find(v => v.box_type === boxCode)
  if (bv) return Number(bv.calculated_value)
  return getSlipBoxValue(boxCode)
}

/** Calculate net pay from box values */
const calculateNetPay = (values: Record<string, number>): number => {
  return (values['box_14'] ?? 0)
    - (values['box_16'] ?? 0)
    - (values['box_16a'] ?? 0)
    - (values['box_18'] ?? 0)
    - (values['box_22'] ?? 0)
    - (values['box_20'] ?? 0)
    - (values['box_52'] ?? 0)
}

/** Original net pay (before adjustment) - from the slip's current net_pay */
const originalNetPay = computed(() => {
  if (!props.slip) return 0
  return Number((props.slip as any).net_pay ?? 0)
})

/** Adjusted net pay - calculated from the adjustment form values */
const adjustedNetPay = computed(() => {
  return calculateNetPay(adjustmentForm.value)
})

/** Whether there's a meaningful discrepancy */
const netPayHasDiscrepancy = computed(() => {
  return Math.abs(adjustedNetPay.value - originalNetPay.value) > 0.01
})

// ==================== CPP/CPP2/EI Validation ====================

/** Tolerance for rounding differences (in dollars) */
const VALIDATION_TOLERANCE = 0.50

/** CPP Pensionable earnings from the form (Box 26) */
const cppPensionableEarnings = computed(() => adjustmentForm.value['box_26'] ?? 0)

/** CPP contributions from the form (Box 16) */
const cppContributions = computed(() => adjustmentForm.value['box_16'] ?? 0)

/** CPP2 contributions from the form (Box 16a) */
const cpp2Contributions = computed(() => adjustmentForm.value['box_16a'] ?? 0)

/** EI Insurable earnings from the form (Box 24) */
const eiInsurableEarnings = computed(() => adjustmentForm.value['box_24'] ?? 0)

/** EI premiums from the form (Box 18) */
const eiPremiums = computed(() => adjustmentForm.value['box_18'] ?? 0)

/** Expected CPP contribution based on pensionable earnings and rate */
const expectedCpp = computed(() => {
  if (!taxRates.value) return null
  // cpp_pensionable_earnings (Box 26) already has the $3,500 exemption subtracted
  const pensionable = cppPensionableEarnings.value
  const rate = taxRates.value.cpp_employee_rate
  const base = pensionable * rate
  return Math.min(base, taxRates.value.cpp_max_contribution)
})

/** Expected CPP2 contribution based on earnings exceeding YMPE */
const expectedCpp2 = computed(() => {
  if (!taxRates.value) return null
  const pensionable = cppPensionableEarnings.value
  const rate = taxRates.value.cpp2_rate
  // CPP2 only applies if earnings exceed YMPE
  const earningsAboveYmpe = Math.max(0, pensionable - taxRates.value.cpp_ympe)
  const base = earningsAboveYmpe * rate
  return Math.min(base, taxRates.value.cpp2_max_contribution)
})

/** Expected EI premium based on insurable earnings and rate */
const expectedEi = computed(() => {
  if (!taxRates.value) return null
  const insurable = eiInsurableEarnings.value
  const rate = taxRates.value.ei_rate
  const base = insurable * rate
  return Math.min(base, taxRates.value.ei_max_contribution)
})

/** CPP validation result */
interface ValidationCheck {
  label: string
  actual: number
  expected: number
  difference: number
  isOk: boolean
  description: string
}

const cppValidation = computed<ValidationCheck | null>(() => {
  if (!taxRates.value || expectedCpp.value === null) return null
  const actual = cppContributions.value
  const expected = expectedCpp.value
  const diff = Math.abs(actual - expected)
  return {
    label: 'CPP Contributions (Box 16)',
    actual,
    expected,
    difference: actual - expected,
    isOk: diff <= VALIDATION_TOLERANCE,
    description: `Expected: (${formatAmount(cppPensionableEarnings.value)} - ${formatAmount(taxRates.value.cpp_basic_exemption)}) × ${(taxRates.value.cpp_employee_rate * 100).toFixed(2)}% = ${formatAmount(expected)}`
  }
})

/** CPP2 validation result */
const cpp2Validation = computed<ValidationCheck | null>(() => {
  if (!taxRates.value || expectedCpp2.value === null) return null
  const actual = cpp2Contributions.value
  const expected = expectedCpp2.value
  const diff = Math.abs(actual - expected)
  const earningsAboveYmpe = Math.max(0, cppPensionableEarnings.value - taxRates.value.cpp_ympe)
  return {
    label: 'CPP2 Contributions (Box 16a)',
    actual,
    expected,
    difference: actual - expected,
    isOk: diff <= VALIDATION_TOLERANCE,
    description: earningsAboveYmpe > 0
      ? `Expected: (${formatAmount(cppPensionableEarnings.value)} - ${formatAmount(taxRates.value.cpp_ympe)}) × ${(taxRates.value.cpp2_rate * 100).toFixed(2)}% = ${formatAmount(expected)}`
      : `No CPP2 expected (earnings ${formatAmount(cppPensionableEarnings.value)} ≤ YMPE ${formatAmount(taxRates.value.cpp_ympe)})`
  }
})

/** EI validation result */
const eiValidation = computed<ValidationCheck | null>(() => {
  if (!taxRates.value || expectedEi.value === null) return null
  const actual = eiPremiums.value
  const expected = expectedEi.value
  const diff = Math.abs(actual - expected)
  return {
    label: 'EI Premiums (Box 18)',
    actual,
    expected,
    difference: actual - expected,
    isOk: diff <= VALIDATION_TOLERANCE,
    description: `Expected: ${formatAmount(eiInsurableEarnings.value)} × ${(taxRates.value.ei_rate * 100).toFixed(2)}% = ${formatAmount(expected)}`
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
  return numValue.toLocaleString('en-CA', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

const formatCurrency = (value: any): string => {
  const num = Number(value ?? 0)
  if (Number.isNaN(num)) return '$0.00'
  return num.toLocaleString('en-CA', { style: 'currency', currency: 'CAD' })
}

const formatDentalBenefit = (code: number) => {
  switch (code) {
    case 1: return 'No dental'
    case 2: return 'Basic'
    case 3: return 'Comprehensive'
    default: return 'No dental'
  }
}

/** Load the slip and pre-fill form values when dialog opens */
watch(() => props.modelValue, async (isOpen) => {
  if (isOpen && props.slip) {
    originalCalculatedValues.value = []
    taxRates.value = null
    taxRatesError.value = null
    // Pre-fill form with current values
    const initialValues: Record<string, number> = {}
    for (const box of T4_BOX_TYPES) {
      initialValues[box.box_code] = getSlipBoxValueFor(props.slip, box.box_code)
    }
    adjustmentForm.value = initialValues
    // Fetch original calculated values from the database
    try {
      const slipRecord = await t4Api.getOrCreateT4Slip(props.slip.employee.id!, props.slip.year)
      if (slipRecord.id) {
        originalCalculatedValues.value = await t4Api.getT4BoxValues(slipRecord.id)
      }
    } catch {
      // Fall back to current values if box values can't be fetched
    }
    // Load tax rates for validation
    try {
      taxRates.value = await employeeApi.getTaxRates(props.slip.year)
    } catch (e: any) {
      taxRatesError.value = e?.toString() || 'Failed to load tax rates'
    }
  }
})

const handleSave = async () => {
  if (!props.slip) return
  saving.value = true
  try {
    const adjustments: Record<string, number> = {}
    for (const box of T4_BOX_TYPES) {
      adjustments[box.box_code] = (adjustmentForm.value[box.box_code] ?? 0) - getSlipBoxValue(box.box_code)
    }
    emit('save', adjustments)
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.v-dialog {
  max-height: 90vh !important;
}

.v-card {
  display: flex !important;
  flex-direction: column !important;
  max-height: 90vh !important;
}

.v-card-title {
  flex-shrink: 0 !important;
}

.v-card-text {
  flex: 1 1 auto !important;
  overflow-y: auto !important;
  overflow-x: hidden !important;
  display: flex !important;
  flex-direction: column !important;
  min-height: 0 !important;
}

.v-card-actions {
  flex-shrink: 0 !important;
  background: white !important;
  border-top: 1px solid rgba(0, 0, 0, 0.12) !important;
}
</style>
