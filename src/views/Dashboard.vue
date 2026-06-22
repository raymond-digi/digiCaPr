<template>
  <div>
    <v-row>
      <v-col cols="12">
        <h1 class="text-h5 mb-4">Dashboard</h1>
      </v-col>
    </v-row>

    <!-- Summary Cards -->
    <v-row>
      <v-col cols="12" md="3">
        <v-card>
          <v-card-text>
            <div class="text-overline mb-1">Active Employees</div>
            <div class="text-h4">{{ employeeStore.activeEmployees.length }}</div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="3">
        <v-card>
          <v-card-text>
            <div class="text-overline mb-1">Total Employees</div>
            <div class="text-h4">{{ employeeStore.employees.length }}</div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="3">
        <v-card>
          <v-card-text>
            <div class="text-overline mb-1">Payroll Records</div>
            <div class="text-h4">{{ payrollStore.payrolls.length }}</div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="3">
        <v-card>
          <v-card-text>
            <div class="text-overline mb-1">Current Year</div>
            <div class="text-h4">{{ currentYear }}</div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Quick Actions -->
    <v-row class="mt-4">
      <v-col cols="12">
        <v-card>
          <v-card-title>Quick Actions</v-card-title>
          <v-card-text>
            <v-row>
              <v-col cols="12" md="3">
                <v-btn block color="primary" prepend-icon="mdi-account-plus" @click="$router.push('/employees')">
                  Employee
                </v-btn>
              </v-col>
              <v-col cols="12" md="3">
                <v-btn block color="success" prepend-icon="mdi-cash-plus" @click="$router.push('/payroll')">
                  Process Payroll
                </v-btn>
              </v-col>
              <v-col cols="12" md="3">
                <v-btn block color="warning" prepend-icon="mdi-bank-transfer" @click="$router.push('/remittance')">
                  Remittance
                </v-btn>
              </v-col>
              <v-col cols="12" md="3">
                <v-btn block color="secondary" prepend-icon="mdi-cog" @click="$router.push('/settings')">
                  Settings
                </v-btn>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Recent Activity -->
    <v-row class="mt-4">
      <v-col cols="12">
        <v-card>
          <v-card-title>Recent Payroll Activity</v-card-title>
          <v-card-text>
            <v-list v-if="recentPayrolls.length > 0">
              <v-list-item v-for="payroll in recentPayrolls" :key="payroll.id">
                <template #prepend>
                  <v-icon>mdi-cash</v-icon>
                </template>
                <v-list-item-title>
                  Employee ID: {{ payroll.employee_id }} - ${{ Number(payroll.net_pay).toFixed(2) }}
                </v-list-item-title>
                <v-list-item-subtitle>
                  {{ payroll.pay_period_start }} to {{ payroll.pay_period_end }}
                </v-list-item-subtitle>
                <template #append>
                </template>
              </v-list-item>
            </v-list>
            <div v-else class="text-center py-4 text-grey">
              No recent payroll activity
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useEmployeeStore } from '@/stores/employee'
import { usePayrollStore } from '@/stores/historyPayroll'

const employeeStore = useEmployeeStore()
const payrollStore = usePayrollStore()

const currentYear = new Date().getFullYear()

const recentPayrolls = computed(() => {
  return payrollStore.payrolls.slice(0, 10)
})


onMounted(async () => {
  try {
    await Promise.all([
      employeeStore.fetchEmployees(),
      payrollStore.fetchPayrolls()
    ])
  } catch (error) {
    console.error('Failed to load dashboard data:', error)
  }
})
</script>
