<template>
  <v-dialog v-model="dialog" max-width="1100px" persistent>
    <v-card>
      <v-card-title>
        <span class="text-h5">
          {{ isEdit ? `Edit Employee - ${formData.first_name} ${formData.last_name}` : 'Add Employee' }}
        </span>
      </v-card-title>

      <v-card-text>
        <v-form ref="formRef" v-model="valid" style="height: 100%; display: flex; flex-direction: column;">
          <TabView v-model="activeTab" class="flex-1" :tab-status="tabStatus">
            <template #personal>
              <PersonalInfoTab :form-data="formData" :rules="rules" :provinces="provinces" />
            </template>
            <template #employment>
              <EmploymentTab :form-data="formData" :rules="rules" :provinces="provinces" />
            </template>
            <template #payroll>
              <PayrollTab ref="payrollTabRef" :form-data="formData" :rules="rules" :pay-types="payTypes"
                :provinces="provinces" />
            </template>
            <template #history>
              <HistoryTab :pay-rate-history="payRateHistory" :employment-history="employmentHistory"
                :loading-history="loadingHistory" :format-date="formatDate" />
            </template>
          </TabView>
        </v-form>
      </v-card-text>

      <!-- Error Alert -->
      <v-alert v-if="saveError" type="error" variant="tonal" class="mx-4 mt-2 mb-0" closable @click:close="saveError = ''">
        {{ saveError }}
      </v-alert>

      <v-card-actions>
        <small class="pa-2 text-caption text-grey">* indicates required field</small>
        <v-spacer />
        <v-btn color="grey" variant="text" @click="handleCancel">
          Cancel
        </v-btn>
        <v-btn color="primary" variant="elevated" @click="handleSave" :disabled="!isFormValid" :loading="loading">
          {{ isEdit ? 'Update' : 'Create' }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed, nextTick, watchEffect } from 'vue'
import TabView from './TabView.vue'
import PersonalInfoTab from './EmployeeFormTabs/PersonalInfoTab.vue'
import EmploymentTab from './EmployeeFormTabs/EmploymentTab.vue'
import PayrollTab from './EmployeeFormTabs/PayrollTab.vue'
import HistoryTab from './EmployeeFormTabs/HistoryTab.vue'
import { formatDateLocal, toDateString } from '@/utils/date'
import type { Employee, PayRateHistory, EmploymentHistory } from '@/types/employee'
import { Province, PayType } from '@/types/employee'
import { employeeApi } from '@/services/api'

const props = defineProps<{
  modelValue: boolean
  employee?: Employee | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', employee: Employee): void
}>()

const dialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const activeTab = ref<'personal' | 'employment' | 'payroll' | 'history'>('personal')

const formRef = ref<any>(null)
const payrollTabRef = ref<any>(null)
const valid = ref(false)
const loading = ref(false)
const loadingHistory = ref(false)
const payRateHistory = ref<PayRateHistory[]>([])
const employmentHistory = ref<EmploymentHistory[]>([])
const saveError = ref('')

// Tab validation status - tracks which tabs have been validated
const tabStatus = ref<Record<string, 'pending' | 'valid' | 'invalid'>>({
  personal: 'pending',
  employment: 'pending',
  payroll: 'pending',
  history: 'pending' // History tab is read-only, always valid
})

const isEdit = computed(() => !!props.employee?.id)

// Check if all required tabs (personal, employment, payroll) are valid
const allTabsValid = computed(() => {
  return tabStatus.value.personal === 'valid' &&
    tabStatus.value.employment === 'valid' &&
    tabStatus.value.payroll === 'valid'
})

// Computed property to determine if form is valid
const isFormValid = computed(() => {
  // In edit mode, we need to check if the form has been validated
  // For new employees, rely on the valid state
  if (isEdit.value) {
    // Allow save if form data is present and basic validation passes
    return formData.value.employee_number &&
      formData.value.first_name &&
      formData.value.last_name &&
      formData.value.sin &&
      formData.value.pay_rate > 0
  }
  // For new employees, require all tabs to be valid
  return valid.value && allTabsValid.value
})

const provinces: Province[] = Object.values(Province).sort() as Province[];

const payTypes = [PayType.Hourly, PayType.Weekly, PayType.Monthly, PayType.Annual]

const defaultFormData = (): Employee => ({
  employee_number: '',
  first_name: '',
  last_name: '',
  sin: '',
  address: {
    street: '',
    city: '',
    province: 'ON',
    postal_code: ''
  },
  hire_province: 'ON',
  pay_type: PayType.Monthly,
  pay_rate: 0.0,
  date_of_birth: '1990-01-01',
  vacation_pay_rate: 0.04,
  overtime_multiplier: 1.5,
  additional_tax_amount: 0,
  hire_date: toDateString(new Date()),
  termination_date: undefined,
  is_active: true,
  created_at: new Date().toISOString()
})

const formData = ref<Employee>(defaultFormData())

const rules = {
  required: (value: any) => !!value || 'Required field',
  positiveNumber: (value: number) => value > 0 || 'Must be greater than 0',
  sin: (value: string) => {
    // Basic SIN format validation (XXX-XXX-XXX)
    const sinRegex = /^\d{3}-?\d{3}-?\d{3}$/
    const cleanSin = value.replace(/\s/g, '')
    if (!sinRegex.test(cleanSin)) {
      return 'Invalid SIN format (use XXX-XXX-XXX)'
    }
    
    // Luhn algorithm (mod 10 check digit validation)
    const digits = cleanSin.replace(/-/g, '').split('').map(Number)
    let sum = 0
    
    for (let i = 0; i < digits.length; i++) {
      let digit = digits[i]
      
      // Double every second digit (odd positions: 1, 3, 5, 7)
      if (i % 2 === 1) {
        digit *= 2
        // If doubling results in two digits, add them together
        if (digit > 9) {
          digit = Math.floor(digit / 10) + (digit % 10)
        }
      }
      sum += digit
    }
    
    // Valid if sum is divisible by 10
    return sum % 10 === 0 || 'Invalid SIN check digit'
  },
  postalCode: (value: string) => {
    // Canadian postal code (A1A 1A1)
    const postalRegex = /^[A-Z]\d[A-Z]\s?\d[A-Z]\d$/i
    return postalRegex.test(value) || 'Invalid postal code format'
  },
  date: (value: string) => {
    if (!value) return true
    const date = new Date(value)
    return !isNaN(date.getTime()) || 'Invalid date'
  },
  dateOfBirth: (value: string) => {
    if (!value) return 'Date of birth is required'
    const dob = new Date(value)
    const today = new Date()
    const age = today.getFullYear() - dob.getFullYear()
    const monthDiff = today.getMonth() - dob.getMonth()
    const actualAge = monthDiff < 0 || (monthDiff === 0 && today.getDate() < dob.getDate()) ? age - 1 : age
    if (actualAge < 15) return 'Employee must be at least 15 years old'
    if (actualAge > 100) return 'Invalid date of birth'
    if (dob >= today) return 'Date of birth must be in the past'
    return true
  },
  vacationRate: (value: number) => {
    if (value < 0 || value > 20) return 'Must be between 0 and 20%'
    return true
  },
  overtimeMultiplier: (value: number) => {
    if (value < 1.0 || value > 3.0) return 'Must be between 1.0x and 3.0x'
    return true
  },
  personalAmountYear: (value: number) => {
    if (value < 2020 || value > 2030) return 'Must be between 2020 and 2030'
    return true
  }
}

// Validation helper functions for each tab
const validatePersonalTab = (): 'valid' | 'invalid' => {
  const required = [
    formData.value.employee_number,
    formData.value.first_name,
    formData.value.last_name,
    formData.value.address?.street,
    formData.value.address?.city,
    formData.value.address?.postal_code
  ]
  
  // Check if all required fields are filled
  const allFilled = required.every(field => field && String(field).trim() !== '')
  
  // Check postal code format if filled
  const postalCodeValid = !formData.value.address?.postal_code ||
    /^[A-Z]\d[A-Z]\s?\d[A-Z]\d$/i.test(formData.value.address.postal_code)
  
  return allFilled && postalCodeValid ? 'valid' : 'invalid'
}

const validateEmploymentTab = (): 'valid' | 'invalid' => {
  const required = [
    formData.value.hire_province,
    formData.value.hire_date
  ]
  
  const allFilled = required.every(field => field && String(field).trim() !== '')
  
  // Check termination date is valid if present
  const terminationDateValid = !formData.value.termination_date ||
    !isNaN(new Date(formData.value.termination_date).getTime())
  
  return allFilled && terminationDateValid ? 'valid' : 'invalid'
}

const validatePayrollTab = (): 'valid' | 'invalid' => {
  // Check SIN format and mod 10 validation
  const sinValid = (() => {
    if (!formData.value.sin) return false
    const cleanSin = formData.value.sin.replace(/\s/g, '')
    
    // Check format
    if (!/^\d{3}-?\d{3}-?\d{3}$/.test(cleanSin)) return false
    
    // Luhn algorithm (mod 10 check digit validation)
    const digits = cleanSin.replace(/-/g, '').split('').map(Number)
    let sum = 0
    
    for (let i = 0; i < digits.length; i++) {
      let digit = digits[i]
      
      // Double every second digit (odd positions: 1, 3, 5, 7)
      if (i % 2 === 1) {
        digit *= 2
        // If doubling results in two digits, add them together
        if (digit > 9) {
          digit = Math.floor(digit / 10) + (digit % 10)
        }
      }
      sum += digit
    }
    
    // Valid if sum is divisible by 10
    return sum % 10 === 0
  })()
  
  // Check date of birth
  const dobValid = (() => {
    if (!formData.value.date_of_birth) return false
    const dob = new Date(formData.value.date_of_birth)
    const today = new Date()
    const age = today.getFullYear() - dob.getFullYear()
    const monthDiff = today.getMonth() - dob.getMonth()
    const actualAge = monthDiff < 0 || (monthDiff === 0 && today.getDate() < dob.getDate()) ? age - 1 : age
    return actualAge >= 15 && actualAge <= 100 && dob < today
  })()
  
  // Check numeric fields
  const payRateValid = formData.value.pay_rate > 0
  const vacationRateValid = formData.value.vacation_pay_rate >= 0 && formData.value.vacation_pay_rate <= 0.20
  const overtimeValid = formData.value.overtime_multiplier >= 1.0 && formData.value.overtime_multiplier <= 3.0
  
  return sinValid && dobValid && payRateValid && vacationRateValid && overtimeValid ? 'valid' : 'invalid'
}

// Watch formData changes to update tab status in real-time
watchEffect(() => {
  // Only update if dialog is open
  if (props.modelValue) {
    tabStatus.value.personal = validatePersonalTab()
    tabStatus.value.employment = validateEmploymentTab()
    tabStatus.value.payroll = validatePayrollTab()
  }
})

watch(() => props.employee, async (newEmployee) => {
  if (newEmployee) {
    formData.value = { ...newEmployee }
    // Load history if editing
    if (newEmployee.id) {
      await loadHistory(newEmployee.id)
    }
    // Force validation after data is loaded
    await nextTick()
    formRef.value?.validate()
  } else {
    formData.value = defaultFormData()
    payRateHistory.value = []
    employmentHistory.value = []
  }
}, { immediate: true })

watch(() => props.modelValue, async (isOpen) => {
  if (isOpen) {
    if (!props.employee) {
      formData.value = defaultFormData()
      formRef.value?.resetValidation()
      payRateHistory.value = []
      employmentHistory.value = []
    }
    // Tab status will be set by watchEffect automatically
    tabStatus.value.history = 'valid' // History tab is always valid (read-only)
  } else {
    // Reset form when dialog closes (success or cancel)
    formData.value = defaultFormData()
    formRef.value?.resetValidation()
    payRateHistory.value = []
    employmentHistory.value = []
    saveError.value = ''
  }
})


const loadHistory = async (employeeId: number) => {
  loadingHistory.value = true
  try {
    const [payRates, employment] = await Promise.all([
      employeeApi.getPayRateHistory(employeeId),
      employeeApi.getEmploymentHistory(employeeId)
    ])
    payRateHistory.value = payRates
    employmentHistory.value = employment
  } catch (error) {
    console.error('Failed to load history:', error)
    payRateHistory.value = []
    employmentHistory.value = []
  } finally {
    loadingHistory.value = false
  }
}

const formatDate = (dateStr: string): string => {
  if (!dateStr) return ''
  return formatDateLocal(dateStr)
}

const handleCancel = () => {
  dialog.value = false
  formData.value = defaultFormData()
  formRef.value?.resetValidation()
  payRateHistory.value = []
  employmentHistory.value = []
}

const handleSave = async () => {
  // Validate all tabs before saving
  if (formRef.value) {
    const result = await formRef.value.validate()
    const isValid = result.valid
    
    // Update status for all tabs based on validation
    if (!isEdit.value) {
      // For new employees, mark all tabs based on validation
      // Since the form validates all fields, we need to determine which tab has errors
      const errors = result.errors || []
      const errorIds = errors.map((e: any) => e.id || '')
      
      // Check each tab's fields for errors
      const personalFields = ['employee_number', 'first_name', 'last_name', 'address.street', 'address.city', 'address.province', 'address.postal_code']
      const employmentFields = ['hire_province', 'hire_date']
      const payrollFields = ['sin', 'date_of_birth', 'pay_type', 'pay_rate', 'vacation_pay_rate', 'overtime_multiplier']
      
      const hasPersonalError = personalFields.some(f => errorIds.some((id: string) => id.includes(f.replace('.', '-'))) || errors.some((e: any) => e.field?.includes(f)))
      const hasEmploymentError = employmentFields.some(f => errorIds.some((id: string) => id.includes(f.replace('.', '-'))) || errors.some((e: any) => e.field?.includes(f)))
      const hasPayrollError = payrollFields.some(f => errorIds.some((id: string) => id.includes(f.replace('.', '-'))) || errors.some((e: any) => e.field?.includes(f)))
      
      tabStatus.value.personal = hasPersonalError ? 'invalid' : 'valid'
      tabStatus.value.employment = hasEmploymentError ? 'invalid' : 'valid'
      tabStatus.value.payroll = hasPayrollError ? 'invalid' : 'valid'
      
      if (!isValid) {
        // Switch to the first tab with errors
        if (hasPersonalError) {
          activeTab.value = 'personal'
        } else if (hasEmploymentError) {
          activeTab.value = 'employment'
        } else if (hasPayrollError) {
          activeTab.value = 'payroll'
        }
        return
      }
    }
  }

  // Additional check for required fields
  if (!isFormValid.value) {
    return
  }

  loading.value = true
  saveError.value = ''
  try {
    const employeeData = { ...formData.value }
    let employeeId: number

    if (isEdit.value) {
      await employeeApi.updateEmployee(employeeData)
      employeeId = employeeData.id!
    } else {
      employeeId = await employeeApi.createEmployee(employeeData)
      employeeData.id = employeeId
      formData.value.id = employeeId
    }

    emit('save', employeeData)

    // Save personal amount (best effort)
    if (payrollTabRef.value) {
      payrollTabRef.value.savePersonalAmount().catch((error: any) => {
        console.error('Personal amount save failed:', error)
      })
    }

    // Save autofills (best effort)
    if (payrollTabRef.value?.saveAutofills) {
      payrollTabRef.value.saveAutofills().catch((error: any) => {
        console.error('Autofills save failed:', error)
      })
    }

    dialog.value = false
  } catch (error: any) {
    console.error('Save failed:', error)
    saveError.value = error?.message || error?.toString() || 'Failed to save employee. Please try again.'
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
  height: 90vh !important;
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

.v-card-text .v-form {
  flex: 1 1 auto !important;
  display: flex !important;
  flex-direction: column !important;
  min-height: 0 !important;
  overflow: hidden !important;
}

.v-card-text .flex-1 {
  flex: 1 1 auto !important;
  min-height: 0 !important;
}

.v-card-actions {
  flex-shrink: 0 !important;
  background: white !important;
  border-top: 1px solid rgba(0, 0, 0, 0.12) !important;
}
</style>
