<template>
  <v-card variant="outlined" class="pa-3 mt-3">
    <div class="text-subtitle-1 font-weight-bold mb-2">
      <v-icon size="small" class="mr-1">mdi-beach</v-icon>
      Vacation Balance
    </div>

    <v-row dense>
      <v-col cols="6" sm="3">
        <div class="text-caption text-medium-emphasis">Accrued</div>
        <div class="text-body-2 font-weight-medium">{{ formatCurrency(totalAccrued) }}</div>
      </v-col>
      <v-col cols="6" sm="3">
        <div class="text-caption text-medium-emphasis">Used/Paid</div>
        <div class="text-body-2 font-weight-medium">{{ formatCurrency(totalPaid) }}</div>
      </v-col>
      <v-col cols="6" sm="3">
        <div class="text-caption text-medium-emphasis">Balance</div>
        <div class="text-body-2 font-weight-bold" :class="balanceColor">{{ formatCurrency(balance) }}</div>
      </v-col>
      <v-col cols="6" sm="3" v-if="(hourlyRate ?? 0) > 0">
        <div class="text-caption text-medium-emphasis">Hours Available</div>
        <div class="text-body-2 font-weight-medium">≈ {{ hoursAvailable.toFixed(1) }} hrs</div>
      </v-col>
    </v-row>

    <div class="text-caption text-medium-emphasis mt-2" v-if="(hourlyRate ?? 0) > 0">
      Based on current rate: {{ formatCurrency(hourlyRate ?? 0) }}/hr
    </div>
  </v-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { vacationApi } from '@/services/api'

const props = defineProps<{
  employeeId: number
  hourlyRate?: number  // Current hourly pay rate for hours calculation
}>()

const balance = ref(0)
const totalAccrued = ref(0)
const totalPaid = ref(0)

const hoursAvailable = computed(() => {
  if (!props.hourlyRate || props.hourlyRate <= 0) return 0
  return balance.value / props.hourlyRate
})

const balanceColor = computed(() => {
  if (balance.value < 0) return 'text-error'
  if (balance.value > 0) return 'text-success'
  return 'text-medium-emphasis'
})

const formatCurrency = (value: number): string => {
  const num = Number(value ?? 0)
  if (Number.isNaN(num)) return '$0.00'
  return num.toLocaleString('en-CA', {
    style: 'currency',
    currency: 'CAD'
  })
}

const loadBalance = async () => {
  if (!props.employeeId) return
  try {
    const result = await vacationApi.getBalance(props.employeeId)
    balance.value = Number(result.balance ?? 0)
    totalAccrued.value = Number(result.total_accrued ?? 0)
    totalPaid.value = Number(result.total_paid ?? 0)
  } catch (error) {
    console.error('Failed to load vacation balance:', error)
  }
}

onMounted(loadBalance)

watch(() => props.employeeId, loadBalance)

// Expose refresh for parent components
defineExpose({ refresh: loadBalance })
</script>
