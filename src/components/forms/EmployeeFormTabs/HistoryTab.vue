<template>
  <v-container>
    <v-row>
      <!-- Pay Rate History Section -->
      <v-col cols="12">
        <v-card variant="outlined" class="mb-4">
          <v-card-title class="text-subtitle-1 bg-blue-grey-lighten-5">
            Pay Rate History
          </v-card-title>
          <v-card-text style="max-height: 300px; overflow-y: auto;">
            <v-progress-circular v-if="loadingHistory" indeterminate size="24" class="ma-4" />
            <div v-else-if="payRateHistory.length === 0" class="text-caption text-grey pa-2">
              No pay rate history available
            </div>
            <v-timeline v-else side="end" density="compact" align="start" size="small">
              <v-timeline-item v-for="(rate, index) in payRateHistory" :key="rate.id" :dot-color="index === 0 ? 'primary' : 'grey'" size="small">
                <template #opposite>
                  <div class="text-caption">{{ formatDate(rate.effective_date) }}</div>
                </template>
                <v-card variant="outlined" density="compact">
                  <v-card-text class="py-2">
                    <div class="text-body-2 font-weight-medium">
                      ${{ Number(rate.pay_rate).toFixed(2) }}
                      <v-chip size="x-small" class="ml-1">{{ rate.pay_type }}</v-chip>
                    </div>
                    <div v-if="rate.reason" class="text-caption text-grey">{{ rate.reason }}</div>
                    <div v-if="rate.end_date" class="text-caption text-grey">
                      Ended: {{ formatDate(rate.end_date) }}
                    </div>
                    <div v-else class="text-caption text-success">Current</div>
                  </v-card-text>
                </v-card>
              </v-timeline-item>
            </v-timeline>
          </v-card-text>
        </v-card>
      </v-col>

      <!-- Employment History Section -->
      <v-col cols="12">
        <v-card variant="outlined">
          <v-card-title class="text-subtitle-1 bg-blue-grey-lighten-5">
            Employment History
          </v-card-title>
          <v-card-text style="max-height: 300px; overflow-y: auto;">
            <v-progress-circular v-if="loadingHistory" indeterminate size="24" class="ma-4" />
            <div v-else-if="employmentHistory.length === 0" class="text-caption text-grey pa-2">
              No employment history available
            </div>
            <v-timeline v-else side="end" density="compact" align="start" size="small">
              <v-timeline-item v-for="(emp, index) in employmentHistory" :key="emp.id" :dot-color="index === 0 ? 'success' : 'grey'" size="small">
                <template #opposite>
                  <div class="text-caption">{{ formatDate(emp.hire_date) }}</div>
                </template>
                <v-card variant="outlined" density="compact">
                  <v-card-text class="py-2">
                    <div class="text-body-2 font-weight-medium">
                      Hired
                    </div>
                    <div v-if="emp.termination_date" class="text-caption text-grey">
                      Terminated: {{ formatDate(emp.termination_date) }}
                    </div>
                    <div v-else class="text-caption text-success">Currently Employed</div>
                    <div v-if="emp.termination_reason" class="text-caption text-grey">
                      Reason: {{ emp.termination_reason }}
                    </div>
                    <div v-if="emp.notes" class="text-caption text-grey">
                      {{ emp.notes }}
                    </div>
                  </v-card-text>
                </v-card>
              </v-timeline-item>
            </v-timeline>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import type { PayRateHistory, EmploymentHistory } from '@/types/employee'

defineProps<{
  payRateHistory: PayRateHistory[]
  employmentHistory: EmploymentHistory[]
  loadingHistory: boolean
  formatDate: (date: string) => string
}>()
</script>