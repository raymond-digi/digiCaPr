<template>
  <v-dialog v-model="dialog" max-width="600px" persistent>
    <v-card>
      <v-card-title>
        <span class="text-h5">
          <v-icon icon="mdi-cog" class="mr-2" />
          Transmitter Settings (T619)
        </span>
      </v-card-title>

      <v-card-text>
        <v-alert type="info" variant="tonal" class="mb-4">
          Transmitter information is used for CRA T619 Internet File Transfer XML submissions.
          If not set, company information will be used as fallback.
        </v-alert>

        <v-form v-model="valid">
          <v-container>
            <!-- CRA Transmitter Section -->
            <v-row>
              <v-col cols="12">
                <h4 class="text-subtitle-1 mb-2">CRA Transmitter Information</h4>
              </v-col>

              <!-- BN15 (Business Number with RP program identifier) -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.bn15" label="Transmitter BN15*" :rules="[rules.required, rules.bn15]" variant="outlined" density="comfortable" placeholder="123456789RP0001"
                  hint="Business Number with program identifier (e.g., 123456789RP0001)" persistent-hint />
              </v-col>

              <!-- Transmitter Name -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.name" label="Transmitter Name*" :rules="[rules.required]" variant="outlined" density="comfortable" hint="Legal name of the transmitting organization"
                  persistent-hint />
              </v-col>
            </v-row>

            <!-- Contact Information -->
            <v-row>
              <v-col cols="12">
                <v-divider class="my-2" />
                <h4 class="text-subtitle-1 mb-2">Contact Information</h4>
              </v-col>

              <!-- Contact Person -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.contact_name" label="Contact Person Name" variant="outlined" density="comfortable" />
              </v-col>

              <!-- Phone Area Code -->
              <v-col cols="12" md="3">
                <v-text-field v-model="formData.phone_area" label="Area Code" variant="outlined" density="comfortable" placeholder="416" :rules="[rules.areaCode]" />
              </v-col>

              <!-- Phone Number -->
              <v-col cols="12" md="3">
                <v-text-field v-model="formData.phone" label="Phone Number" variant="outlined" density="comfortable" placeholder="321-7654" :rules="[rules.phone]" />
              </v-col>
            </v-row>

            <!-- Additional Options -->
            <v-row>
              <v-col cols="12">
                <v-divider class="my-2" />
                <h4 class="text-subtitle-1 mb-2">Additional Options</h4>
              </v-col>

              <!-- Email -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.email" label="Contact Email" variant="outlined" density="comfortable" type="email" :rules="[rules.email]" />
              </v-col>

              <!-- Submission Reference -->
              <v-col cols="12" md="6">
                <v-text-field v-model="formData.submission_ref_id" label="Submission Reference ID" variant="outlined" density="comfortable" hint="Optional reference for this submission"
                  persistent-hint />
              </v-col>
            </v-row>
          </v-container>
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
import { registryApi } from '@/services/api'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save'): void
}>()

const dialog = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const valid = ref(false)
const loading = ref(false)

interface TransmitterData {
  bn15: string
  name: string
  contact_name: string
  phone_area: string
  phone: string
  email: string
  submission_ref_id: string
}

const defaultFormData = (): TransmitterData => ({
  bn15: '',
  name: '',
  contact_name: '',
  phone_area: '',
  phone: '',
  email: '',
  submission_ref_id: '',
})

const formData = ref<TransmitterData>(defaultFormData())

const rules = {
  required: (value: any) => !!value || 'Required field',
  bn15: (value: string) => {
    if (!value) return true
    // BN15 format: 9 digits + RP/RC/RT + 4 digits
    const cleaned = value.replace(/[\s-]/g, '')
    const bnRegex = /^(\d{9}|\d{9}(RP|RC|RT)\d{4})$/
    return bnRegex.test(cleaned) || 'Must be 9 digits or full BN format (e.g., 123456789RP0001)'
  },
  areaCode: (value: string) => {
    if (!value) return true
    return /^\d{3}$/.test(value) || 'Must be 3 digits'
  },
  phone: (value: string) => {
    if (!value) return true
    return /^\d{3}-?\d{4}$/.test(value.replace(/\s/g, '')) || 'Must be 7 digits (e.g., 321-7654)'
  },
  email: (value: string) => {
    if (!value) return true
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    return emailRegex.test(value) || 'Invalid email format'
  },
}

// Registry key paths for transmitter info
const REGISTRY_KEYS = {
  bn15: 'transmitter/bn15',
  name: 'transmitter/name',
  contact_name: 'transmitter/contact_name',
  phone_area: 'transmitter/phone_area',
  phone: 'transmitter/phone',
  email: 'transmitter/email',
  submission_ref_id: 'transmitter/submission_ref',
} as const

const loadTransmitterData = async () => {
  try {
    const data = defaultFormData()

    // Load each field from registry
    const bn15 = await registryApi.getString(REGISTRY_KEYS.bn15)
    if (bn15) data.bn15 = bn15

    const name = await registryApi.getString(REGISTRY_KEYS.name)
    if (name) data.name = name

    const contactName = await registryApi.getString(REGISTRY_KEYS.contact_name)
    if (contactName) data.contact_name = contactName

    const phoneArea = await registryApi.getString(REGISTRY_KEYS.phone_area)
    if (phoneArea) data.phone_area = phoneArea

    const phone = await registryApi.getString(REGISTRY_KEYS.phone)
    if (phone) data.phone = phone

    const email = await registryApi.getString(REGISTRY_KEYS.email)
    if (email) data.email = email

    const submissionRef = await registryApi.getString(REGISTRY_KEYS.submission_ref_id)
    if (submissionRef) data.submission_ref_id = submissionRef

    formData.value = data
  } catch (error) {
    console.error('Failed to load transmitter data:', error)
  }
}

const saveTransmitterData = async () => {
  loading.value = true
  try {
    // Save each field to registry
    await registryApi.setString(REGISTRY_KEYS.bn15, formData.value.bn15)
    await registryApi.setString(REGISTRY_KEYS.name, formData.value.name)
    await registryApi.setString(REGISTRY_KEYS.contact_name, formData.value.contact_name)
    await registryApi.setString(REGISTRY_KEYS.phone_area, formData.value.phone_area)
    await registryApi.setString(REGISTRY_KEYS.phone, formData.value.phone)
    await registryApi.setString(REGISTRY_KEYS.email, formData.value.email)
    await registryApi.setString(REGISTRY_KEYS.submission_ref_id, formData.value.submission_ref_id)

    emit('save')
    dialog.value = false
  } catch (error) {
    console.error('Failed to save transmitter data:', error)
  } finally {
    loading.value = false
  }
}

const handleSave = async () => {
  if (!valid.value) return
  await saveTransmitterData()
}

const handleCancel = () => {
  dialog.value = false
}

// Watch for dialog open to load data
watch(dialog, (newVal) => {
  if (newVal) {
    loadTransmitterData()
  }
})
</script>
