<template>
  <v-dialog v-model="dialog" max-width="700px" persistent>
    <v-card>
      <v-card-title>
        <span class="text-h5">Process Payroll</span>
      </v-card-title>

      <v-card-text>
        <v-form ref="formRef" v-model="valid">
          <v-container>
            <v-row>
              <!-- Employee Selection -->
              <v-col cols="12">
                <v-autocomplete
                  v-model="formData.employee_id"
                  label="Employee*"
                  :items="employeeOptions"
                  item-title="text"
                  item-value="value"
                  :rules="[rules.required]"
                  variant="outlined"
                  density="comfortable"
                  @update:model-value="handleEmployeeChange"
                />
              </v-col>

              <!-- Pay Period Section -->
              <v-col cols="12">
                <v-divider class="my-2" />
                <h4 class="text-subtitle-1 mb-2">Pay Period</h4>
              </v-col>

              <!-- Pay Period Start -->
              <v-col cols="12" md="6">
                <v-text-field
                  v-model="formData.pay_period_start"
                  label="Period Start*"
                  :rules="[rules.required, rules.date]"
                  variant="outlined"
                  density="comfortable"
                  type="date"
                />
              </v-col>

              <!-- Pay Period End -->
              <v-col cols="12" md="6">
                <v-text-field
                  v-model="formData.pay_period_end"
                  label="Period End*"
                  :rules="[rules.required, rules.date, rules.endAfterStart]"
                  variant="outlined"
                  density="comfortable"
                  type="date"
                />
              </v-col>

              <!-- Pay Date -->
              <v-col cols="12" md="6">
                <v-text-field
                  v-model="formData.pay_date"
                  label="Pay Date*"
                  :rules="[rules.required, rules.date]"
                  variant="outlined"
                  density="comfortable"
                  type="date"
                />
              </v-col>


              <!-- Hours/Amount Input -->
              <v-col cols="12" v-if="selectedEmployee">
                <v-text-field
                  v-model.number="formData.gross_pay"
                  :label="selectedEmployee.pay_type === 'Hourly' ? 'Hours Worked*' : 'Pay Amount*'"
                  :rules="[rules.required, rules.positiveNumber]"
                  variant="outlined"
                  density="comfortable"
                  type="number"
                  step="0.01"
                  :suffix="selectedEmployee.pay_type === 'Hourly' ? 'hours' : ''"
                  :prefix="selectedEmployee.pay_type === 'Weekly' || selectedEmployee.pay_type === 'Monthly' || selectedEmployee.pay_type === 'Annual' ? '$' : ''"
                />
              </v-col>

              <!-- Preview Section -->
              <v-col cols="12" v-if="preview">
                <v-divider class="my-2" />
                <h4 class="text-subtitle-1 mb-2">Payroll Preview</h4>
                
                <v-card variant="outlined" class="mb-2">
                  <v-card-text>
                    <v-row dense>
                      <v-col cols="6">
                        <div class="text-caption text-grey">Gross Pay</div>
                        <div class="text-h6">${{ preview.gross_pay.toFixed(2) }}</div>
                      </v-col>
                      <v-col cols="6">
                        <div class="text-caption text-grey">Net Pay</div>
                        <div class="text-h6 text-success">${{ preview.net_pay.toFixed(2) }}</div>
                      </v-col>
                    </v-row>
                  </v-card-text>
                </v-card>

                <v-card variant="outlined">
                  <v-list density="compact">
                    <v-list-subheader>Deductions</v-list-subheader>
                    <v-list-item>
                      <v-list-item-title>CPP</v-list-item-title>
                      <template v-slot:append>
                        <span>${{ preview.deductions.cpp.toFixed(2) }}</span>
                      </template>
                    </v-list-item>
                    <v-list-item>
                      <v-list-item-title>EI</v-list-item-title>
                      <template v-slot:append>
                        <span>${{ preview.deductions.ei.toFixed(2) }}</span>
                      </template>
                    </v-list-item>
                    <v-list-item>
                      <v-list-item-title>Federal Tax</v-list-item-title>
                      <template v-slot:append>
                        <span>${{ preview.deductions.federal_tax.toFixed(2) }}</span>
                      </template>
                    </v-list-item>
                    <v-list-item>
                      <v-list-item-title>Provincial Tax</v-list-item-title>
                      <template v-slot:append>
                        <span>${{ preview.deductions.provincial_tax.toFixed(2) }}</span>
                      </template>
                    </v-list-item>
                    <v-divider />
                    <v-list-item>
                      <v-list-item-title class="font-weight-bold">Total Deductions</v-list-item-title>
                      <template v-slot:append>
                        <span class="font-weight-bold">${{ totalDeductions.toFixed(2) }}</span>
                      </template>
                    </v-list-item>
                  </v-list>
                </v-card>
              </v-col>

              <!-- Calculate Button -->
              <v-col cols="12" v-if="!preview && canCalculate">
                <v-btn color="primary" variant="outlined" block @click="handleCalculate" :loading="calculating">
                  Calculate Payroll
                </v-btn>
              </v-col>
            </v-row>
          </v-container>

          <small>*indicates required field</small>
        </v-form>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="handleCancel">
          Cancel
        </v-btn>
        <v-btn 
          color="primary" 
          variant="elevated" 
          @click="handleSave" 
          :disabled="!valid || !preview" 
          :loading="loading"
        >
          Save Payroll
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { toDateString } from '@/utils/date'
import type { Employee } from '@/types/employee'
import type { Payroll, PayrollCalculationInput } from '@/types/payroll'
import { payrollApi } from '@/services/api'

const props = defineProps<{
  modelValue: boolean
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

const formRef = ref<any>(null)
const valid = ref(false)
const loading = ref(false)
const calculating = ref(false)
const preview = ref<Payroll | null>(null)


const defaultFormData = (): PayrollCalculationInput => {
  const today = toDateString(new Date())
  return {
    employee_id: 0,
    pay_period_start: today,
    pay_period_end: today,
    pay_date: today,
    gross_pay: 0
  }
}

const formData = ref<PayrollCalculationInput>(defaultFormData())

const employeeOptions = computed(() => 
  props.employees
    .filter(e => e.is_active)
    .map(e => ({
      text: `${e.employee_number} - ${e.first_name} ${e.last_name}`,
      value: e.id!
    }))
)

const selectedEmployee = computed(() => 
  props.employees.find(e => e.id === formData.value.employee_id)
)

const canCalculate = computed(() => {
  return formData.value.employee_id > 0 &&
    formData.value.pay_period_start &&
    formData.value.pay_period_end &&
    formData.value.pay_date &&
    formData.value.gross_pay != null && formData.value.gross_pay > 0
})

const totalDeductions = computed(() => {
  if (!preview.value) return 0
  return preview.value.deductions.cpp +
    preview.value.deductions.ei +
    preview.value.deductions.federal_tax +
    preview.value.deductions.provincial_tax
})

const rules = {
  required: (value: any) => !!value || 'Required field',
  positiveNumber: (value: number | null | undefined) => (value != null && value > 0) || 'Must be greater than 0',
  date: (value: string) => {
    if (!value) return true
    const date = new Date(value)
    return !isNaN(date.getTime()) || 'Invalid date'
  },
  endAfterStart: (value: string) => {
    if (!value || !formData.value.pay_period_start) return true
    return new Date(value) >= new Date(formData.value.pay_period_start) || 
      'End date must be after start date'
  }
}

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    formData.value = defaultFormData()
    preview.value = null
    formRef.value?.resetValidation()
  } else {
    formData.value = defaultFormData()
    preview.value = null
    formRef.value?.resetValidation()
  }
})

const handleEmployeeChange = () => {
  preview.value = null
  if (selectedEmployee.value) {
    // Pre-fill with employee's pay rate if weekly or monthly or annual
    const payPeriodsPerYear = 26; // Assuming bi-weekly pay periods
    if (selectedEmployee.value.pay_type === 'Annual') {
      const annualSalary = selectedEmployee.value.pay_rate
      formData.value.gross_pay = annualSalary / payPeriodsPerYear
    } else if (selectedEmployee.value.pay_type === 'Monthly') {
      const monthlySalary = selectedEmployee.value.pay_rate
      formData.value.gross_pay = (monthlySalary * 12) / payPeriodsPerYear
    } else if (selectedEmployee.value.pay_type === 'Weekly') {
      const weeklySalary = selectedEmployee.value.pay_rate
      formData.value.gross_pay = (weeklySalary * 52) / payPeriodsPerYear
    }
  }
}

const handleCalculate = async () => {
  const { valid } = await formRef.value.validate()
  if (!valid) return

  calculating.value = true
  try {
    preview.value = await payrollApi.calculatePayroll(formData.value)
  } catch (error) {
    console.error('Error calculating payroll:', error)
  } finally {
    calculating.value = false
  }
}

const handleCancel = () => {
  dialog.value = false
  formData.value = defaultFormData()
  preview.value = null
  formRef.value?.resetValidation()
}

const handleSave = async () => {
  if (!preview.value) return

  loading.value = true
  try {
    emit('save', preview.value)
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
  border-top: 1px solid rgba(0,0,0,0.12) !important;
}
</style>
