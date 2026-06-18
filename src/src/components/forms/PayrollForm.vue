<template>

  <v-dialog v-model="dialog" max-width="80vw" max-height="90vh" persistent>
    <v-card class="fill-height d-flex flex-column" v-if="editingPayroll">
      <v-card-title class="flex-shrink-0 pa-4 pb-2 d-flex align-center justify-space-between">
        <span>{{ isAddMode ? 'Add Employee to Payroll' : 'Edit Payroll' }}</span>
        <v-chip color="grey-darken-3" variant="tonal" v-if="payPeriodDates">
          Pay Period {{ payPeriodDates.pay_period_number }}
          of {{ payPeriodDates.total_pay_periods }}
          ({{ formatDate(payPeriodDates.pay_period_start) }}
          - {{ formatDate(payPeriodDates.pay_period_end) }})
        </v-chip>
      </v-card-title>

      <!-- Net Pay Summary - Sticky at Top -->
      <div class="flex-shrink-0">
        <v-card variant="tonal" color="success" class="pa-2" style="border-radius: 0;">
          <v-row align="center">
            <v-col cols="12" sm="5">
              <div class="flex-shrink-0 text-h6 text-medium-emphasis pa-2 px-4">
                {{ employeeName }}
              </div>
            </v-col>
            <v-col cols="2" class="hidden-sm-and-up"> </v-col>
            <v-col cols="3" sm="2">
              <div class="text-caption">Gross Pay</div>
              <div class="text-h7 text-lg-h6">{{ formatCurrency(totalGrossPay) }}</div>
            </v-col>
            <v-col cols="3" sm="2">
              <div class="text-caption">Deductions</div>
              <div class="text-h7 text-lg-h6">{{ formatCurrency(totalDeductions) }}</div>
            </v-col>
            <v-col cols="3" sm="2">
              <div class="text-caption">Net Pay</div>
              <div class="text-h7 text-lg-h6 font-weight-bold">{{ formatCurrency(netPay) }}</div>
            </v-col>
          </v-row>
        </v-card>
      </div>

      <v-divider class="flex-shrink-0" />

      <div class="flex-grow-1 overflow-y-auto pa-6">
        <v-alert v-if="error" type="error" variant="tonal" class="mb-4" closable @click:close="error = ''">
          {{ error }}
        </v-alert>

        <v-form ref="formRef" v-model="formValid">
          <!-- Two Column Layout -->
          <v-row>
            <!-- LEFT COLUMN: EARNINGS -->
            <v-col cols="12" md="6">
              <v-card variant="outlined" class="pa-4">
                <div class="text-h5 mb-3 text-success">
                  <v-icon left>mdi-cash-plus</v-icon>
                  Earnings
                </div>

                <!-- Warning when no earnings at all -->
                <v-alert v-if="grossPay === 0 && !hasAdditionalEarnings" type="warning" variant="tonal" density="compact" class="mb-3" icon="mdi-alert">
                  Base and additional earnings are both zero.
                </v-alert>

                <!-- Base Pay Input -->
                <div class="text-subtitle-1 font-weight-bold mb-2">Base Pay</div>
                <v-row v-if="employee?.pay_type === 'Hourly'">
                  <v-col cols="6" sm="3" md="4" xl="3">
                    <v-text-field v-model.number="regularHours" label="Regular Hours" type="number" min="0" step="0.01" suffix="hrs" :rules="[rules.defined]" variant="outlined" density="compact"
                      :hint="`@ ${formatCurrency(employee?.pay_rate ?? 0)} / hr`" persistent-hint />
                  </v-col>
                  <v-col cols="6" sm="3" md="4" xl="3">
                    <v-text-field v-model.number="overtimeHours" label="Overtime Hours" type="number" min="0" step="0.01" suffix="hrs" variant="outlined" density="compact"
                      :hint="`@ ${formatCurrency((employee?.pay_rate ?? 0) * (employee?.overtime_multiplier ?? 1.5))} / hr`" persistent-hint />
                  </v-col>
                  <v-col cols="6" sm="3" md="4" xl="3">
                    <v-text-field v-model.number="grossPay" label="Base Pay" type="number" prefix="$" :rules="[rules.defined]" variant="outlined" density="compact" readonly
                      bg-color="grey-lighten-4" />
                  </v-col>
                </v-row>

                <!-- Base Pay for Salaried -->
                <v-row class="mt-2" v-else>
                  <v-col cols="5" sm="4" md="5" xl="3">
                    <v-text-field v-model.number="grossPay" label="Base Pay" type="number" min="0" step="0.01" prefix="$" :rules="[rules.defined]" variant="outlined"
                      density="compact" :hint="`period: ${formatCurrency(defaultGrossPay)}`" persistent-hint />
                  </v-col>
                  <v-col cols="7" sm="6" md="7" xl="8">
                    <v-card variant="tonal" color="info" style="height: 42px; display: flex; align-items: center;">
                      <div class="text-body-2 my-auto">
                        {{ employee?.pay_type === 'Monthly' ? 'Monthly Rate' : employee?.pay_type === 'Weekly' ? 'Weekly Rate' : 'Annual Rate' }}:
                        {{ formatCurrency(employee?.pay_rate ?? 0) }}
                      </div>
                    </v-card>
                  </v-col>
                </v-row>

                <v-divider class="my-3" />

                <!-- Additional Earnings -->
                <div class="text-subtitle-1 font-weight-bold mb-3">Additional Earnings</div>
                <v-container fluid class="pa-0" v-if="editAdditionalEarnings.length >= 6">
                  <v-row dense class="mb-2">
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalEarnings[0].amount" label="Bonus" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined" hide-details />
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalEarnings[1].amount" label="Commission" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined"
                        hide-details />
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalEarnings[2].amount" label="Benefit" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined" hide-details />
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalEarnings[3].amount" label="Allowance" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined"
                        hide-details />
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalEarnings[4].amount" label="Vacation" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined" hide-details>
                        <template #append-inner>
                          <v-btn icon="mdi-calculator" size="x-small" variant="text" color="primary" @click="calculateVacationPay" :disabled="!employee || grossPay <= 0" density="compact" />
                        </template>
                      </v-text-field>
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalEarnings[5].amount" label="Other" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined" hide-details />
                    </v-col>
                  </v-row>
                </v-container>
              </v-card>
            </v-col>

            <!-- RIGHT COLUMN: DEDUCTIONS -->
            <v-col cols="12" md="6">
              <v-card variant="outlined" class="pa-4">
                <div class="text-h5 mb-3 text-error">
                  <v-icon left>mdi-cash-minus</v-icon>
                  Deductions
                </div>

                <!-- Statutory Deductions (Read-only) -->
                <div class="d-flex justify-space-between align-center mb-3 text-subtitle-1 font-weight-bold">
                  <span>Statutory Deductions</span>
                  <v-btn color="warning" :loading="loadingRecalc" :disabled="!formValid" @click="recalculate" size="small">
                    Recalculate
                  </v-btn>
                </div>
                <v-card variant="tonal" color="grey-lighten" class="pa-3 mb-4">
                  <v-row dense>
                    <v-col cols="6" sm="3" md="6" lg="4" xl="3" class="text-body-2 order-sm-1 order-md-1 order-lg-1 order-xl-1">
                      CPP: {{ formatCurrency(editingPayroll.deductions?.cpp ?? 0) }}
                    </v-col>
                    <v-col cols="6" sm="3" md="6" lg="4" xl="3" class="text-body-2 order-sm-5 order-md-3 order-lg-4 order-xl-5">
                      CPP2: {{ formatCurrency(editingPayroll.deductions?.cpp2 ?? 0) }}
                    </v-col>
                    <v-col cols="6" sm="3" md="6" lg="4" xl="3" class="text-body-2 order-sm-2 order-md-2 order-lg-7 order-xl-2">
                      EI: {{ formatCurrency(editingPayroll.deductions?.ei ?? 0) }}
                    </v-col>
                    <v-col cols="6" sm="3" md="6" lg="4" xl="3" class="text-body-2 order-sm-6 order-md-4 order-lg-9 order-xl-6" offset-lg="4">
                      Prov: {{ editingPayroll.province ?? 'N/A' }}
                    </v-col>
                    <v-col cols="6" sm="3" md="6" lg="4" xl="3" class="text-body-2 order-sm-3 order-md-5 order-lg-2 order-xl-3">
                      Fed: {{ formatCurrency(editingPayroll.deductions?.federal_tax ?? 0) }}
                    </v-col>
                    <v-col cols="6" sm="3" md="6" lg="4" xl="3" class="text-body-2 order-sm-7 order-md-7 order-lg-5 order-xl-7">
                      Fed PA: {{ formatCurrency(editingPayroll.federal_personal_amount ?? 0) }}
                    </v-col>
                    <v-col cols="6" sm="3" md="6" lg="4" xl="3" class="text-body-2 order-sm-4 order-md-6 order-lg-3 order-xl-4">
                      Prov: {{ formatCurrency(editingPayroll.deductions?.provincial_tax ?? 0) }}
                    </v-col>
                    <v-col cols="6" sm="3" md="6" lg="4" xl="3" class="text-body-2 order-sm-8 order-md-8 order-lg-6 order-xl-8">
                      Prov PA: {{ formatCurrency(editingPayroll.provincial_personal_amount ?? 0) }}
                    </v-col>
                  </v-row>
                </v-card>

                <v-divider class="my-3" />

                <!-- Additional Deductions -->
                <div class="text-subtitle-1 font-weight-bold mb-3">Additional Deductions</div>
                <v-container fluid class="pa-0" v-if="editAdditionalDeductions.length >= 4">
                  <v-row dense class="mb-2">
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalDeductions[1].amount" label="Pension/RRSP" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined"
                        hide-details />
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalDeductions[2].amount" label="Union Dues" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined"
                        hide-details />
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalDeductions[4].amount" label="Addon Tax" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined"
                        hide-details />
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalDeductions[0].amount" label="Insurance" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined"
                        hide-details />
                    </v-col>
                    <v-col cols="6" sm="3" md="4" xl="3">
                      <v-text-field v-model.number="editAdditionalDeductions[3].amount" label="Pay Adjust" prefix="$" type="number" step="0.01" min="0" density="compact" variant="outlined"
                        hide-details />
                    </v-col>
                  </v-row>
                </v-container>
              </v-card>
            </v-col>
          </v-row>
        </v-form>
      </div>

      <v-card-actions class="xflex-shrink-0 xpa-4 xjustify-space-between">
        <v-spacer />
        <v-btn variant="outlined" @click="handleCancel">Cancel</v-btn>
        <v-btn color="primary" :disabled="loadingRecalc || saving || !formValid || (grossPay === 0 && !hasAdditionalEarnings)" :loading="loadingRecalc || saving" @click="handleSave">
          {{ isAddMode ? 'Add to Payroll' : 'Save Changes' }}
        </v-btn>
      </v-card-actions>
    </v-card>

    <v-card v-else>
      <v-card-text>editingPayroll is not defined</v-card-text>
      <v-card-actions>
        <v-btn variant="outlined" @click="handleCancel">Cancel</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { payrollApi, employeeApi } from '@/services/api'
import { useAppStore } from '@/stores/app'
import { getErrorMessage } from '@/utils/error'
import { formatDateLocal } from '@/utils/date'
import { PayrollCalculationInput, DEDUCTION_TYPES, EARNING_TYPES } from '@/types/payroll'
import type { Payroll, AdditionalEarning, AdditionalDeduction } from '@/types/payroll'
import type { Employee } from '@/types/employee'

interface PayPeriodDates {
  pay_period_start: string
  pay_period_end: string
  pay_date: string
  pay_period_number?: number
  total_pay_periods?: number
}

const props = defineProps<{
  modelValue: boolean
  payroll?: Payroll | null
  employee?: Employee | null
  isAddMode?: boolean
  payPeriodDates?: PayPeriodDates | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', payroll: Payroll): void
}>()

const appStore = useAppStore()

const dialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const formRef = ref<any>(null)
const formValid = ref(false)
const editingPayroll = ref<Payroll | null>(null)
const regularHours = ref<number | null>(null)
const overtimeHours = ref<number | null>(null)
const grossPay = ref(0)
const loadingRecalc = ref(false)
const saving = ref(false)
const error = ref('')
const editAdditionalEarnings = ref<AdditionalEarning[]>([])
const editAdditionalDeductions = ref<AdditionalDeduction[]>([])

const rules = {
  required: (v: any) => !!v || 'Required',
  defined: (v: any) => (v !== null && v !== undefined && v !== '') || 'Required',
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

const formatDate = formatDateLocal

const employeeName = computed(() => {
  return props.employee ? `${props.employee.first_name} ${props.employee.last_name}` : ''
})

const defaultGrossPay = computed(() => {
  if (!props.employee || !props.payPeriodDates) {
    return 0;
  }

  if (props.employee.pay_type === 'Annual') {
    const totalPeriods = props.payPeriodDates.total_pay_periods || 26
    return Number(props.employee.pay_rate ?? 0) / totalPeriods
  } else if (props.employee.pay_type === 'Weekly') {
    const totalPeriods = props.payPeriodDates.total_pay_periods || 26
    return (Number(props.employee.pay_rate ?? 0) * totalPeriods) / 52
  } else if (props.employee.pay_type === 'Monthly') {
    const totalPeriods = props.payPeriodDates.total_pay_periods || 26
    return (Number(props.employee.pay_rate ?? 0) * 12) / totalPeriods
  }
  return 0
})

const hasAdditionalEarnings = computed(() => {
  return editAdditionalEarnings.value?.some((e: any) => Number(e.amount ?? 0) > 0) ?? false
})

const totalGrossPay = computed(() => {
  return Number(grossPay.value ?? 0) +
    editAdditionalEarnings.value.reduce((sum: number, e: any) => sum + Number(e.amount ?? 0), 0)
})

const totalDeductions = computed(() => {
  return Number(editingPayroll.value?.deductions?.cpp ?? 0) +
    Number(editingPayroll.value?.deductions?.cpp2 ?? 0) +
    Number(editingPayroll.value?.deductions?.ei ?? 0) +
    Number(editingPayroll.value?.deductions?.federal_tax ?? 0) +
    Number(editingPayroll.value?.deductions?.provincial_tax ?? 0) +
    editAdditionalDeductions.value.reduce((sum: number, d: any) => sum + Number(d.amount ?? 0), 0)
})

const netPay = computed(() => {
  return totalGrossPay.value - totalDeductions.value
})

// Helper function to initialize earnings with fixed types
// Optionally accepts autofill values to pre-populate amounts
const initializeEarnings = (existingEarnings: AdditionalEarning[] = [], autofillValues: any[] = []): AdditionalEarning[] => {
  return EARNING_TYPES.map(et => {
    const type = et.name
    const existing = existingEarnings.find(e => e.earning_type === type)
    if (existing) {
      return existing
    }

    // Check for autofill value
    const autofill = autofillValues.find(a => a.type_name === type && a.autofill_type === 'earning')

    return {
      payroll_id: editingPayroll.value?.id || 0,
      earning_type: type,
      amount: autofill?.amount || 0,
      hours: null,
      is_periodic: et.is_periodic
    }
  })
}

// Helper function to initialize deductions with fixed types
// Optionally accepts autofill values to pre-populate amounts
const initializeDeductions = (existingDeductions: AdditionalDeduction[] = [], autofillValues: any[] = []): AdditionalDeduction[] => {
  return DEDUCTION_TYPES.map(dt => {
    const type = dt.name
    const existing = existingDeductions.find(d => d.name === type)
    if (existing) {
      return existing
    }

    // Check for autofill value
    const autofill = autofillValues.find(a => a.type_name === type && a.autofill_type === 'deduction')

    return {
      name: type,
      amount: autofill?.amount || 0
    }
  })
}

// Load autofill values for an employee
const loadAutofillValues = async (employeeId: number) => {
  try {
    const autofills = await employeeApi.getActiveEmployeeAutofill(employeeId)
    return autofills
  } catch (error) {
    console.error('Failed to load autofill values:', error)
    return []
  }
}

// Auto-calculate gross pay for hourly employees
watch([regularHours, overtimeHours, () => props.employee], () => {
  if (props.employee?.pay_type === 'Hourly') {
    const regularHrs = regularHours.value ?? 0
    const overtimeHrs = overtimeHours.value ?? 0
    const regularRate = Number(props.employee.pay_rate ?? 0)
    const overtimeMultiplier = Number(props.employee.overtime_multiplier ?? 1.5)
    const overtimeRate = regularRate * overtimeMultiplier

    grossPay.value = Math.round((regularHrs * regularRate + overtimeHrs * overtimeRate) * 100) / 100
  }
})

// Initialize form when payroll prop changes
watch(() => props.payroll, (newPayroll) => {
  if (newPayroll) {
    editingPayroll.value = JSON.parse(JSON.stringify(newPayroll))
    regularHours.value = newPayroll.regular_hours ?? null
    overtimeHours.value = newPayroll.overtime_hours ?? null
    grossPay.value = Number(newPayroll.gross_pay ?? 0)
    editAdditionalEarnings.value = initializeEarnings(newPayroll.additional_earnings)
    editAdditionalDeductions.value = initializeDeductions(newPayroll.deductions?.additional)
  }
}, { immediate: true })

// Initialize for add mode when employee changes
watch(() => props.employee, async (newEmployee) => {
  if (props.isAddMode && newEmployee && props.payPeriodDates) {
    // Initialize form values based on employee type
    if (newEmployee.pay_type === 'Hourly') {
      regularHours.value = 0
      overtimeHours.value = 0
      grossPay.value = 0
    } else {
      const defaultGross = defaultGrossPay.value ?? 0
      grossPay.value = Math.round(defaultGross * 100) / 100
    }

    // Initialize a new payroll object for adding
    editingPayroll.value = {
      id: undefined,
      employee_id: newEmployee.id!,
      pay_period_start: props.payPeriodDates.pay_period_start,
      pay_period_end: props.payPeriodDates.pay_period_end,
      pay_date: props.payPeriodDates.pay_date,
      regular_hours: null,
      overtime_hours: null,
      gross_pay: 0,
      insured_earning: 0,
      additional_earnings: [],
      additional_earnings_total: 0,
      additional_tax_amount: 0,
      deductions: {
        cpp: 0,
        cpp2: 0,
        ei: 0,
        federal_tax: 0,
        provincial_tax: 0,
        additional: []
      },
      additional_deductions: 0,
      net_pay: 0,
      pay_period_number: props.payPeriodDates.pay_period_number ?? 0,
      total_pay_periods: props.payPeriodDates.total_pay_periods ?? 0,
      federal_personal_amount: 0,
      provincial_personal_amount: 0,
      province: newEmployee.address?.province ?? 'ON'
    }

    // Load autofill values and apply them to earnings/deductions
    const autofills = await loadAutofillValues(newEmployee.id!)
    editAdditionalEarnings.value = initializeEarnings([], autofills)
    editAdditionalDeductions.value = initializeDeductions([], autofills)
  }
}, { immediate: true })

// Clean up when dialog closes, initialize when opens in add mode
watch(() => props.modelValue, async (isOpen) => {
  if (!isOpen) {
    await new Promise(resolve => setTimeout(resolve, 350))
    error.value = ''
    editingPayroll.value = null
    regularHours.value = null
    overtimeHours.value = null
    grossPay.value = 0
    editAdditionalEarnings.value = []
    editAdditionalDeductions.value = []
    formRef.value?.resetValidation()
  } else if (isOpen && props.isAddMode && props.employee) {
    // Initialize arrays when dialog opens in add mode
    // This ensures arrays are populated even if employee prop doesn't change
    // Load autofill values and apply them
    const autofills = await loadAutofillValues(props.employee.id!)
    editAdditionalEarnings.value = initializeEarnings([], autofills)
    editAdditionalDeductions.value = initializeDeductions([], autofills)
  }
})

const recalculate = async () => {
  if (!editingPayroll.value || !props.payPeriodDates) return

  loadingRecalc.value = true
  try {
    // Filter non-zero additional earnings to send to backend
    const nonZeroEarnings = editAdditionalEarnings.value
      .filter(e => Number(e.amount ?? 0) > 0)
      .map(e => ({
        payroll_id: editingPayroll.value!.id ?? 0,
        earning_type: e.earning_type,
        amount: Number(e.amount ?? 0),
        hours: e.hours,
        is_periodic: e.is_periodic
      }))

    // Filter non-zero additional deductions to send to backend
    const nonZeroDeductions = editAdditionalDeductions.value
      .filter(d => Number(d.amount ?? 0) > 0)
      .map(d => ({
        name: d.name,
        amount: Number(d.amount ?? 0)
      }))

    const calcInput: PayrollCalculationInput = {
      employee_id: editingPayroll.value.employee_id,
      pay_period_start: props.payPeriodDates.pay_period_start,
      pay_period_end: props.payPeriodDates.pay_period_end,
      pay_date: props.payPeriodDates.pay_date,
      regular_hours: props.employee?.pay_type === 'Hourly' ? Number(regularHours.value ?? 0) : null,
      overtime_hours: props.employee?.pay_type === 'Hourly' ? Number(overtimeHours.value ?? 0) : null,
      gross_pay: grossPay.value > 0 ? grossPay.value : null,
      additional_earnings: nonZeroEarnings.length > 0 ? nonZeroEarnings : null,
      additional_deductions: nonZeroDeductions.length > 0 ? nonZeroDeductions : null
    }

    const calculated = await payrollApi.calculatePayroll(calcInput)
    const originalId = editingPayroll.value.id

    // Update editingPayroll with recalculated values
    editingPayroll.value = calculated
    if (originalId !== undefined) {
      editingPayroll.value.id = originalId
    }

    // Update form fields
    regularHours.value = calculated.regular_hours ?? null
    overtimeHours.value = calculated.overtime_hours ?? null
    grossPay.value = Number(calculated.gross_pay)

    // Restore additional earnings to the edit form
    editAdditionalEarnings.value = initializeEarnings(calculated.additional_earnings)

    appStore.showNotification('Taxes recalculated successfully', 'success')
  } catch (e) {
    appStore.showNotification(`Recalculate failed: ${getErrorMessage(e)}`, 'error')
  } finally {
    loadingRecalc.value = false
  }
}

const calculateVacationPay = () => {
  if (!props.employee || grossPay.value <= 0) {
    return
  }

  const vacationRate = Number(props.employee.vacation_pay_rate ?? 0)

  if (vacationRate <= 0) {
    appStore.showNotification('Employee has no vacation pay rate configured', 'warning')
    return
  }

  const vacationPay = Math.round(grossPay.value * vacationRate * 100) / 100
  editAdditionalEarnings.value[4].amount = vacationPay

  appStore.showNotification(
    `Vacation pay calculated: ${formatCurrency(vacationPay)} (${(vacationRate * 100).toFixed(1)}% of ${formatCurrency(grossPay.value)})`,
    'success'
  )
}

const handleCancel = () => {
  dialog.value = false
}

const handleSave = async () => {
  if (!editingPayroll.value || !formRef.value) return

  const { valid } = await formRef.value.validate()
  if (!valid) {
    appStore.showNotification('Please fix validation errors before saving.', 'warning')
    return
  }

  error.value = ''
  saving.value = true

  try {
    // Check if base pay (hours/gross) has changed from original payroll
    const originalPayroll = props.payroll
    const basePayChanged = originalPayroll && (
      regularHours.value !== (originalPayroll.regular_hours ?? null) ||
      overtimeHours.value !== (originalPayroll.overtime_hours ?? null) ||
      grossPay.value !== Number(originalPayroll.gross_pay ?? 0)
    )

    // Only recalculate taxes if base pay changed or if it's add mode
    if (basePayChanged || props.isAddMode) {
      loadingRecalc.value = true

      const nonZeroEarnings = editAdditionalEarnings.value
        .filter(e => Number(e.amount ?? 0) > 0)
        .map(e => ({
          payroll_id: editingPayroll.value!.id ?? 0,
          earning_type: e.earning_type,
          amount: Number(e.amount ?? 0),
          hours: e.hours,
          is_periodic: e.is_periodic
        }))

      // Filter non-zero additional deductions to send to backend
      const nonZeroDeductions = editAdditionalDeductions.value
        .filter(d => Number(d.amount ?? 0) > 0)
        .map(d => ({
          name: d.name,
          amount: Number(d.amount ?? 0)
        }))

      const calcInput: PayrollCalculationInput = {
        employee_id: Number(editingPayroll.value.employee_id),
        pay_period_start: props.payPeriodDates!.pay_period_start,
        pay_period_end: props.payPeriodDates!.pay_period_end,
        pay_date: props.payPeriodDates!.pay_date,
        regular_hours: props.employee?.pay_type === 'Hourly' ? Number(regularHours.value ?? 0) : null,
        overtime_hours: props.employee?.pay_type === 'Hourly' ? Number(overtimeHours.value ?? 0) : null,
        gross_pay: grossPay.value > 0 ? Number(grossPay.value) : null,
        additional_earnings: nonZeroEarnings.length > 0 ? nonZeroEarnings : null,
        additional_deductions: nonZeroDeductions.length > 0 ? nonZeroDeductions : null
      }

      const calculated = await payrollApi.calculatePayroll(calcInput)
      const originalId = editingPayroll.value.id

      // Update with recalculated values
      editingPayroll.value = calculated
      if (originalId !== undefined) {
        editingPayroll.value.id = originalId
      }

      loadingRecalc.value = false
    } else {
      // Only additional earnings changed - update them without recalculating taxes
      const nonZeroEarnings = editAdditionalEarnings.value
        .filter(e => Number(e.amount ?? 0) > 0)
        .map(e => ({
          payroll_id: editingPayroll.value!.id ?? 0,
          earning_type: e.earning_type,
          amount: Number(e.amount ?? 0),
          hours: e.hours,
          is_periodic: e.is_periodic
        }))

      editingPayroll.value.additional_earnings = nonZeroEarnings
      editingPayroll.value.additional_earnings_total = nonZeroEarnings.reduce((sum: number, e: AdditionalEarning) => sum + Number(e.amount ?? 0), 0)
      // Note: gross_pay should NOT include additional earnings - they are tracked separately
      // The backend will calculate the total including additional earnings in insured_earning
    }

    // Filter out entries with zero amounts before saving
    const nonZeroDeductions = editAdditionalDeductions.value.filter(d => Number(d.amount ?? 0) > 0)

    editingPayroll.value.deductions!.additional = nonZeroDeductions
    editingPayroll.value.additional_deductions = nonZeroDeductions.reduce((sum: number, d: AdditionalDeduction) => sum + Number(d.amount ?? 0), 0)

    // Emit save event
    emit('save', editingPayroll.value)

    // Close dialog
    dialog.value = false
  } catch (e) {
    const errorMsg = getErrorMessage(e)
    error.value = errorMsg
    console.error('Error saving payroll:', e)
    appStore.showNotification(`Failed to ${props.isAddMode ? 'add' : 'update'} payroll: ${errorMsg}`, 'error')
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.v-dialog {
  max-height: 90vh !important;
}

.v-dialog__content {
  align-items: center !important;
  display: flex !important;
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
  overflow: hidden !important;
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
