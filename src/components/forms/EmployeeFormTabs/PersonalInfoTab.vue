<template>
  <v-container>
    <v-row>
      <!-- Employee Number -->
      <v-col cols="12" md="6">
        <v-text-field v-model="formData.employee_number" label="Employee Number*" :rules="[rules.required]"
          variant="outlined" density="comfortable" />
      </v-col>

      <!-- Active Status -->
      <v-col cols="12" md="4">
        <v-checkbox v-model="formData.is_active" label="Active Employee" density="comfortable" />
      </v-col>

      <!-- First Name -->
      <v-col cols="12" md="6">
        <v-text-field v-model="formData.first_name" label="First Name*" :rules="[rules.required]" variant="outlined"
          density="comfortable" />
      </v-col>

      <!-- Last Name -->
      <v-col cols="12" md="6">
        <v-text-field v-model="formData.last_name" label="Last Name*" :rules="[rules.required]" variant="outlined"
          density="comfortable" />
      </v-col>

      <!-- Address Section -->
      <v-col cols="12">
        <v-divider class="my-2" />
        <h4 class="text-subtitle-1 mb-2">Address</h4>
      </v-col>

      <!-- Street -->
      <v-col cols="12">
        <v-text-field v-model="formData.address.street" label="Street*" :rules="[rules.required]" variant="outlined"
          density="comfortable" />
      </v-col>

      <!-- City -->
      <v-col cols="12" md="4">
        <v-text-field v-model="formData.address.city" label="City*" :rules="[rules.required]" variant="outlined"
          density="comfortable" />
      </v-col>

      <!-- Province -->
      <v-col cols="12" md="4">
        <v-select v-model="formData.address.province" label="Province*" :items="provinces" :rules="[rules.required]"
          variant="outlined" density="comfortable" />
      </v-col>

      <!-- Postal Code -->
      <v-col cols="12" md="4">
        <v-text-field v-model="formData.address.postal_code" label="Postal Code*"
          :rules="[rules.required, rules.postalCode]" variant="outlined" density="comfortable" placeholder="A1A 1A1" />
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { watch } from 'vue'
import type { Employee } from '@/types/employee'

const props = defineProps<{
  formData: Employee
  rules: any
  provinces: string[]
}>()

// Watch postal code and convert to uppercase
watch(() => props.formData.address.postal_code, (newValue) => {
  if (newValue && newValue !== newValue.toUpperCase()) {
    props.formData.address.postal_code = newValue.toUpperCase()
  }
})
</script>