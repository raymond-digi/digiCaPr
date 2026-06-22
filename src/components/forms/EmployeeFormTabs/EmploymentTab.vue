<template>
  <v-container>
    <v-row>
      <!-- Hire Province (for tax calculations) -->
      <v-col cols="12" sm="4" md="3">
        <v-select v-model="formData.hire_province" :items="provinces" label="Hire Province (for Tax Calculations)*" :rules="[rules.required]" variant="outlined" density="comfortable"
          hint="Province where hired - used for tax calculations (independent of address)" persistent-hint />
      </v-col>

      <!-- Hire Date -->
      <v-col cols="12" sm="4" md="3">
        <v-text-field v-model="formData.hire_date" label="Hire Date*" :rules="[rules.required, rules.date]" variant="outlined" density="comfortable" type="date" />
      </v-col>

      <!-- Termination Date -->
      <v-col cols="12" sm="4" md="3">
        <v-text-field v-model="formData.termination_date" label="Termination Date" :rules="[rules.date]" variant="outlined" density="comfortable" type="date" clearable />
      </v-col>

      <!-- Dental Benefit (T4 Box 45) -->
      <v-col cols="12" sm="4" md="3">
        <v-select v-model="formData.dental_benefit" :items="dentalBenefitOptions" label="Dental Benefit (T4 Box 45)*" variant="outlined" density="comfortable"
          hint="Employer-offered dental benefit code" persistent-hint />
      </v-col>


      <!-- Vacation Balance Card -->
      <v-col cols="12">
        <VacationBalanceCard v-if="formData.id" :employee-id="formData.id" :hourly-rate="formData.pay_type === 'Hourly' ? Number(formData.pay_rate) : hourlyEquivalent" />
      </v-col>

    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Employee } from '@/types/employee'
import VacationBalanceCard from '@/components/VacationBalanceCard.vue'

const props = defineProps<{
  formData: Employee
  rules: any
  provinces: string[]
}>()

// Calculate hourly equivalent for non-hourly employees
const hourlyEquivalent = computed(() => {
  const payType = props.formData.pay_type
  const payRate = Number(props.formData.pay_rate ?? 0)
  if (payRate <= 0) return 0

  if (payType === 'Weekly') return payRate / 40  // ~40 hrs/week
  if (payType === 'Monthly') return (payRate * 12) / 2080  // ~2080 hrs/year
  if (payType === 'Annual') return payRate / 2080
  return payRate  // Hourly already
})

// T4 Box 45 - Employer-offered dental benefit options
const dentalBenefitOptions = [
  { title: 'No dental benefit', value: 1 },
  { title: 'Basic dental coverage', value: 2 },
  { title: 'Comprehensive dental coverage', value: 3 },
]
</script>