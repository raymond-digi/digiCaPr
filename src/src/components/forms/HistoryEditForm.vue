<template>
  <v-dialog v-model="dialog" max-width="90%" persistent>
    <v-card>
      <v-card-title>
        <span class="text-h5">{{ isEdit ? 'Edit' : 'Add' }} History Payroll (Developer Mode)</span>
      </v-card-title>

      <v-card-text>
        <v-alert type="warning" variant="tonal" class="mb-4">
          <strong>Developer Mode:</strong> No tax calculations will be performed. All values are entered directly.
        </v-alert>

        <v-form ref="formRef" v-model="valid">
          <v-container>
            <!-- Employee Selection -->
            <v-row>
              <v-col cols="12">
                <v-autocomplete v-model="formData.employee_id" label="Employee*" :items="employeeOptions" item-title="text" item-value="value" :rules="[rules.required]" variant="outlined"
                  density="comfortable" :disabled="isEdit" />
              </v-col>
            </v-row>

            <!-- Dates Section -->
            <v-row>
              <v-col cols="12">
                <v-divider class="mb-2" />
                <h4 class="text-subtitle-1 mb-2">Pay Period</h4>
              </v-col>
              <v-col cols="12" sm="6" md="4" lg="3">
                <v-text-field v-model="formData.pay_period_start" label="Period Start*" :rules="[rules.required, rules.date]" variant="outlined" density="comfortable" type="date" />
              </v-col>
              <v-col cols="12" sm="6" md="4" lg="3">
                <v-text-field v-model="formData.pay_period_end" label="Period End*" :rules="[rules.required, rules.date]" variant="outlined" density="comfortable" type="date" />
              </v-col>
              <v-col cols="4" class="d-none d-md-flex d-lg-none">
              </v-col>
              <v-col cols="12" sm="6" md="4" lg="3">
                <v-text-field v-model="formData.pay_date" label="Pay Date*" :rules="[rules.required, rules.date]" variant="outlined" density="comfortable" type="date" />
              </v-col>
              <v-col cols="12" sm="6" md="4" lg="3">
                <v-row>
                  <v-col cols="6">
                    <v-text-field v-model.number="formData.pay_period_number" label="Period Number" variant="outlined" density="comfortable" type="number" min="1" />
                  </v-col>
                  <v-col cols="6">
                    <v-text-field v-model.number="formData.total_pay_periods" label="Total Periods" variant="outlined" density="comfortable" type="number" min="1" />
                  </v-col>
                </v-row>
              </v-col>
            </v-row>

            <!-- Hours Section -->
            <v-row>
              <v-col cols="12" lg="6">
                <v-row>
                  <v-col cols="12">
                    <v-divider class="mb-2" />
                    <h4 class="text-subtitle-1 mb-2">Hours</h4>
                  </v-col>
                  <v-col cols="12" sm="6" md="4" lg="6">
                    <v-text-field v-model.number="formData.regular_hours" label="Regular Hours" variant="outlined" density="comfortable" type="number" step="0.01" min="0" />
                  </v-col>
                  <v-col cols="12" sm="6" md="4" lg="6">
                    <v-text-field v-model.number="formData.overtime_hours" label="Overtime Hours" variant="outlined" density="comfortable" type="number" step="0.01" min="0" />
                  </v-col>
                </v-row>
              </v-col>

              <!-- Earnings Section -->
              <v-col cols="12" lg="6">
                <v-row>
                  <v-col cols="12">
                    <v-divider class="mb-2" />
                    <h4 class="text-subtitle-1 mb-2">Earnings</h4>
                  </v-col>
                  <v-col cols="12" sm="6" md="4" lg="6">
                    <v-text-field v-model.number="formData.gross_pay" label="Gross Pay*" :rules="[rules.required, rules.number]" variant="outlined" density="comfortable" type="number" step="0.01"
                      prefix="$" />
                  </v-col>
                  <v-col cols="12" sm="6" md="4" lg="6">
                    <v-text-field v-model.number="formData.insured_earning" label="Insured Earning" variant="outlined" density="comfortable" type="number" step="0.01" prefix="$" />
                  </v-col>
                </v-row>
              </v-col>
            </v-row>

            <!-- Additional Earnings -->
            <v-row>
              <v-col cols="12">
                <v-divider class="mb-2" />
                <div class="d-flex align-center mb-2">
                  <h4 class="text-subtitle-1">Additional Earnings</h4>
                  <v-spacer />
                  <v-btn size="small" color="primary" variant="outlined" @click="addEarning">
                    <v-icon left>mdi-plus</v-icon>
                    Add
                  </v-btn>
                </div>
              </v-col>
              <v-col cols="12" sm="6" lg="4" v-for="(earning, index) in formData.additional_earnings" :key="index">
                <v-row dense>
                  <v-col cols="6">
                    <v-select v-model="earning.earning_type" label="Type*" :items="earningTypeOptions" item-title="display_name" item-value="name" :rules="[rules.required, rules.uniqueEarningType(index)]" variant="outlined" density="compact" />
                  </v-col>
                  <v-col cols="5">
                    <v-text-field v-model.number="earning.amount" label="Amount*" :rules="[rules.required, rules.number]" variant="outlined" density="compact" type="number" step="0.01" prefix="$" />
                  </v-col>
                  <v-col cols="1">
                    <v-btn icon="mdi-delete" size="small" color="error" variant="text" @click="removeEarning(index)" />
                  </v-col>
                </v-row>
              </v-col>
            </v-row>

            <!-- Deductions Section -->
            <v-row>
              <v-col cols="12">
                <v-divider class="mb-2" />
                <h4 class="text-subtitle-1 mb-2">Deductions</h4>
              </v-col>
              <v-col cols="12" md="4" lg="3">
                <v-text-field v-model.number="formData.deductions.cpp" label="CPP" variant="outlined" density="comfortable" type="number" step="0.01" prefix="$" />
              </v-col>
              <v-col cols="12" md="4" lg="3">
                <v-text-field v-model.number="formData.deductions.ei" label="EI" variant="outlined" density="comfortable" type="number" step="0.01" prefix="$" />
              </v-col>
              <v-col cols="12" md="4" lg="3">
                <v-text-field v-model.number="formData.deductions.federal_tax" label="Federal Tax" variant="outlined" density="comfortable" type="number" step="0.01" prefix="$" />
              </v-col>
              <v-col cols="12" md="4" lg="3">
                <v-text-field v-model.number="formData.deductions.provincial_tax" label="Provincial Tax" variant="outlined" density="comfortable" type="number" step="0.01" prefix="$" />
              </v-col>
            </v-row>

            <!-- Additional Deductions -->
            <v-row>
              <v-col cols="12">
                <v-divider class="mb-2" />
                <div class="d-flex align-center mb-2">
                  <h4 class="text-subtitle-1">Additional Deductions</h4>
                  <v-spacer />
                  <v-btn size="small" color="primary" variant="outlined" @click="addDeduction">
                    <v-icon left>mdi-plus</v-icon>
                    Add
                  </v-btn>
                </div>
              </v-col>
              <v-col cols="12" sm="6" lg="4" v-for="(deduction, index) in formData.deductions.additional" :key="index">
                <v-row dense>
                  <v-col cols="6">
                    <v-select v-model="deduction.name" label="Name*" :items="deductionTypeOptions" item-title="display_name" item-value="name" :rules="[rules.required, rules.uniqueDeductionType(index)]" variant="outlined" density="compact" />
                  </v-col>
                  <v-col cols="5">
                    <v-text-field v-model.number="deduction.amount" label="Amount*" :rules="[rules.required, rules.number]" variant="outlined" density="compact" type="number" step="0.01" prefix="$" />
                  </v-col>
                  <v-col cols="1">
                    <v-btn icon="mdi-delete" size="small" color="error" variant="text" @click="removeDeduction(index)" />
                  </v-col>
                </v-row>
              </v-col>
            </v-row>

            <!-- Calculated Summary -->
            <v-divider class="mb-4" />
            <v-card variant="tonal" color="info" class="pa-3">
              <v-row dense class="text-end">
                <v-col cols="4" class="px-4">
                  <div class="text-caption text-grey">Total Earnings</div>
                  <div class="text-body-1 font-weight-bold">${{ totalEarnings.toFixed(2) }}</div>
                </v-col>
                <v-col cols="4" class="px-4">
                  <div class="text-caption text-grey">Total Deductions</div>
                  <div class="text-body-1 font-weight-bold">${{ totalDeductions.toFixed(2) }}</div>
                </v-col>
                <v-col cols="4" class="px-4">
                  <div class="text-caption text-grey">Net Pay</div>
                  <div class="text-body-1 font-weight-bold">${{ calculatedNetPay.toFixed(2) }}</div>
                </v-col>
              </v-row>
            </v-card>

          </v-container>
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="handleCancel">
          Cancel
        </v-btn>
        <v-btn color="primary" variant="elevated" @click="handleSave" :disabled="!valid" :loading="loading">
          {{ isEdit ? 'Update' : 'Save' }} Payroll
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { toDateString } from '@/utils/date'
import type { Employee } from '@/types/employee'
import type { Payroll } from '@/types/payroll'
import { EARNING_TYPES, DEDUCTION_TYPES } from '@/types/payroll'

const props = defineProps<{
  modelValue: boolean
  payroll?: Payroll | null
  employees: Employee[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', payroll: Payroll): void
}>()

const dialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const isEdit = computed(() => !!props.payroll?.id)

const formRef = ref<any>(null)
const valid = ref(false)
const loading = ref(false)

const defaultFormData = (): Payroll => {
  const today = toDateString(new Date())
  return {
    employee_id: 0,
    pay_period_start: today,
    pay_period_end: today,
    pay_date: today,
    regular_hours: null,
    overtime_hours: null,
    additional_earnings: [],
    insured_earning: 0,
    gross_pay: 0,
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
    net_pay: 0,
    pay_period_number: undefined,
    total_pay_periods: 0,
    total_deductions: 0,
    additional_deductions: 0,
    federal_personal_amount: 0,
    provincial_personal_amount: 0,
    province: '',
    created_at: new Date().toISOString()
  }
}

const formData = ref<Payroll>(defaultFormData())

const employeeOptions = computed(() =>
  props.employees
    .filter(e => e.is_active)
    .map(e => ({
      text: `${e.employee_number} - ${e.first_name} ${e.last_name}`,
      value: e.id!
    }))
)

const totalEarnings = computed(() => {
  const additional = formData.value.additional_earnings?.reduce((sum, e) => sum + Number(e.amount || 0), 0) || 0
  return Number(formData.value.gross_pay || 0) + additional
})

const totalDeductions = computed(() => {
  const d = formData.value.deductions
  const standard = Number(d?.cpp || 0) + Number(d?.ei || 0) + Number(d?.federal_tax || 0) + Number(d?.provincial_tax || 0)
  const additional = d?.additional?.reduce((sum, ad) => sum + Number(ad.amount || 0), 0) || 0
  return standard + additional
})

const calculatedNetPay = computed(() => {
  return totalEarnings.value - totalDeductions.value
})

const earningTypeOptions = EARNING_TYPES.map(t => ({
  name: t.name,
  display_name: t.display_name,
  is_periodic: t.is_periodic
}))

const deductionTypeOptions = DEDUCTION_TYPES.map(t => ({
  name: t.name,
  display_name: t.display_name,
  t4127_variable: t.t4127_variable
}))

const isEarningTypePeriodic = (type: string): boolean => {
  const found = EARNING_TYPES.find(t => t.name === type)
  return found ? found.is_periodic : true
}

const rules = {
  required: (value: any) => !!value || value === 0 || 'Required field',
  number: (value: any) => value === null || value === undefined || value === '' || !isNaN(Number(value)) || 'Must be a number',
  date: (value: string) => {
    if (!value) return true
    const date = new Date(value)
    return !isNaN(date.getTime()) || 'Invalid date'
  },
  uniqueEarningType: (currentIndex: number) => (value: string) => {
    if (!value) return true
    const duplicates = formData.value.additional_earnings?.filter((e, i) =>
      i !== currentIndex && e.earning_type?.toLowerCase() === value?.toLowerCase()
    )
    return !duplicates || duplicates.length === 0 || 'Duplicate earning type'
  },
  uniqueDeductionType: (currentIndex: number) => (value: string) => {
    if (!value) return true
    const duplicates = formData.value.deductions.additional?.filter((d, i) =>
      i !== currentIndex && d.name?.toLowerCase() === value?.toLowerCase()
    )
    return !duplicates || duplicates.length === 0 || 'Duplicate deduction type'
  }
}

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    if (props.payroll) {
      formData.value = JSON.parse(JSON.stringify(props.payroll))
    } else {
      formData.value = defaultFormData()
    }
    formRef.value?.resetValidation()
  }
})

const addEarning = () => {
  if (!formData.value.additional_earnings) {
    formData.value.additional_earnings = []
  }
  formData.value.additional_earnings.push({
    payroll_id: formData.value.id || 0,
    earning_type: '',
    amount: 0,
    hours: null,
    is_periodic: true
  })
}

// Watch earning_type changes and auto-set is_periodic
watch(() => formData.value.additional_earnings, (earnings) => {
  if (!earnings) return
  earnings.forEach((earning) => {
    if (earning.earning_type) {
      earning.is_periodic = isEarningTypePeriodic(earning.earning_type)
    }
  })
}, { deep: true })

const removeEarning = (index: number) => {
  formData.value.additional_earnings.splice(index, 1)
}

const addDeduction = () => {
  if (!formData.value.deductions.additional) {
    formData.value.deductions.additional = []
  }
  formData.value.deductions.additional.push({
    name: '',
    amount: 0
  })
}

const removeDeduction = (index: number) => {
  formData.value.deductions.additional.splice(index, 1)
}

const handleCancel = () => {
  dialog.value = false
  formData.value = defaultFormData()
  formRef.value?.resetValidation()
}

const handleSave = async () => {
  const { valid: isValid } = await formRef.value.validate()
  if (!isValid) return

  loading.value = true
  try {
    // Calculate totals
    const additionalEarningsTotal = formData.value.additional_earnings?.reduce((sum, e) => sum + Number(e.amount || 0), 0) || 0
    const additionalDeductionsTotal = formData.value.deductions.additional?.reduce((sum, d) => sum + Number(d.amount || 0), 0) || 0
    const totalDeductionsVal = Number(formData.value.deductions.cpp || 0) +
      Number(formData.value.deductions.ei || 0) +
      Number(formData.value.deductions.federal_tax || 0) +
      Number(formData.value.deductions.provincial_tax || 0) +
      additionalDeductionsTotal

    const payrollToSave: Payroll = {
      ...formData.value,
      additional_earnings_total: additionalEarningsTotal,
      additional_deductions: additionalDeductionsTotal,
      total_deductions: totalDeductionsVal,
      created_at: formData.value.created_at || new Date().toISOString()
    }

    emit('save', payrollToSave)
    dialog.value = false
  } finally {
    loading.value = false
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
  overflow-y: auto !important;
}

.v-card-actions {
  flex-shrink: 0 !important;
  background: white !important;
  border-top: 1px solid rgba(0, 0, 0, 0.12) !important;
}
</style>
