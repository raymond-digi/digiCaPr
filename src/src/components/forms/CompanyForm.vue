<template>
  <v-dialog v-model="dialog" max-width="600px" persistent>
    <v-card>
      <v-card-title>
        <span class="text-h5">Company Information</span>
      </v-card-title>

      <v-card-text>
        <v-form ref="formRef" v-model="valid">
          <v-container>
            <v-row>
              <!-- Company Name -->
              <v-col cols="12">
                <v-text-field v-model="formData.name" label="Company Name*" :rules="[rules.required]" variant="outlined" density="comfortable" />
              </v-col>

              <!-- Business Number -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.business_number" label="Business Number*" :rules="[rules.required, rules.businessNumber]" variant="outlined" density="comfortable"
                  placeholder="123456789RP0001" />
              </v-col>

              <!-- Payroll Account Number -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.payroll_account_number" label="Payroll Account Number" variant="outlined" density="comfortable" />
              </v-col>

              <!-- Address Section -->
              <v-col cols="12">
                <v-divider class="my-2" />
                <h4 class="text-subtitle-1 mb-2">Address</h4>
              </v-col>

              <!-- Street -->
              <v-col cols="12">
                <v-text-field v-model="formData.address.street" label="Street*" :rules="[rules.required]" variant="outlined" density="comfortable" />
              </v-col>

              <!-- City -->
              <v-col cols="12" md="4">
                <v-text-field v-model="formData.address.city" label="City*" :rules="[rules.required]" variant="outlined" density="comfortable" />
              </v-col>

              <!-- Province -->
              <v-col cols="12" md="4">
                <v-select v-model="formData.address.province" label="Province*" :items="provinces" :rules="[rules.required]" variant="outlined" density="comfortable" />
              </v-col>

              <!-- Postal Code -->
              <v-col cols="12" md="4">
                <v-text-field v-model="formData.address.postal_code" label="Postal Code*" :rules="[rules.required, rules.postalCode]" variant="outlined" density="comfortable" placeholder="A1A 1A1" />
              </v-col>

              <!-- Contact Information -->
              <v-col cols="12">
                <v-divider class="my-2" />
                <h4 class="text-subtitle-1 mb-2">Contact Information</h4>
              </v-col>

              <!-- Contact Person -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.contact_person" label="Contact Person" variant="outlined" density="comfortable" />
              </v-col>

              <!-- Phone -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.phone" label="Phone" :rules="[rules.phone]" variant="outlined" density="comfortable" placeholder="XXX-XXX-XXXX" />
              </v-col>

              <!-- Email -->
              <v-col cols="12">
                <v-text-field v-model="formData.email" label="Email" :rules="[rules.email]" variant="outlined" density="comfortable" type="email" />
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
        <v-btn color="primary" variant="elevated" @click="handleSave" :disabled="!valid" :loading="loading">
          Save
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

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

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import type { Company } from '@/types/company'

const props = defineProps<{
  modelValue: boolean
  company?: Company | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', company: Company): void
}>()

const dialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const formRef = ref<any>(null)
const valid = ref(false)
const loading = ref(false)

const provinces = [
  'AB', 'BC', 'MB', 'NB', 'NL', 'NS', 'NT', 'NU', 'ON', 'PE', 'QC', 'SK', 'YT'
]

const defaultFormData = (): Company => ({
  name: '',
  business_number: '',
  payroll_account_number: '',
  address: {
    street: '',
    city: '',
    province: 'ON',
    postal_code: ''
  },
  contact_person: '',
  phone: '',
  email: '',
  id: undefined,
  created_at: undefined,
  updated_at: undefined
})

const formData = ref<Company>(defaultFormData())

const rules = {
  required: (value: any) => !!value || 'Required field',
  businessNumber: (value: string) => {
    if (!value) return true
    // Canadian BN format: either 9 digits OR 9 digits + RP/RC/RT + 4 digits
    const cleaned = value.replace(/[\s-]/g, '')
    const bnRegex = /^(\d{9}|\d{9}(RP|RC|RT)\d{4})$/
    return bnRegex.test(cleaned) || 'Must be 9 digits (e.g., 123456789) or full format (e.g., 123456789RP0001)'
  },
  postalCode: (value: string) => {
    if (!value) return true
    // Canadian postal code (A1A 1A1)
    const postalRegex = /^[A-Z]\d[A-Z]\s?\d[A-Z]\d$/i
    return postalRegex.test(value) || 'Invalid postal code format'
  },
  phone: (value: string) => {
    if (!value) return true
    // Basic phone validation
    const phoneRegex = /^\d{3}-?\d{3}-?\d{4}$/
    return phoneRegex.test(value.replace(/\s/g, '')) || 'Invalid phone format'
  },
  email: (value: string) => {
    if (!value) return true
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    return emailRegex.test(value) || 'Invalid email format'
  }
}

watch(() => props.company, (newCompany) => {
  if (newCompany) {
    const backendCompany = newCompany as any;
    const parts = (backendCompany.address as string)?.split(', ').map(p => p.trim()) ?? ['', '', ''];
    formData.value = {
      name: backendCompany.name || '',
      business_number: backendCompany.business_number || '',
      payroll_account_number: backendCompany.payroll_account_number || '',
      address: {
        street: parts[0] || '',
        city: parts[1] || '',
        province: backendCompany.province || 'ON',
        postal_code: parts[2] || '',
      },
      contact_person: backendCompany.contact_person || '',
      phone: backendCompany.phone || '',
      email: backendCompany.email || '',
      id: backendCompany.id,
      created_at: backendCompany.created_at,
      updated_at: backendCompany.updated_at,
    };
  } else {
    formData.value = defaultFormData()
  }
}, { immediate: true })

watch(() => props.modelValue, (isOpen) => {
  if (isOpen && !props.company) {
    formData.value = defaultFormData()
    formRef.value?.resetValidation()
  } else if (!isOpen) {
    formData.value = props.company ? { ...props.company } : defaultFormData()
    formRef.value?.resetValidation()
  }
})

const handleCancel = () => {
  dialog.value = false
  formData.value = props.company ? { ...props.company } : defaultFormData()
  formRef.value?.resetValidation()
}

const handleSave = async () => {
  const { valid } = await formRef.value.validate()
  if (!valid) return

  loading.value = true
  try {
    emit('save', formData.value)
    dialog.value = false
  } finally {
    loading.value = false
  }
}
</script>
