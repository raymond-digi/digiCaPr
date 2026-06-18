// CurrentPayroll store - manages current payroll operations
import { defineStore } from 'pinia'
import { payrollApi, remittanceApi, reportsApi } from '@/services/api'
import { useAppStore } from '@/stores/app'
import { getErrorMessage } from '@/utils/error'
import type {
  Payroll,
  CurrentPayrollInput,
  CurrentPayrollResult,
  Remittance,
  RemittanceSummary,
  CurrentPayrollDates
} from '@/types/payroll'
import type { Employee } from '@/types/employee'

export const useCurrentPayrollStore = defineStore('currentPayroll', {
  state: () => ({
    // Current payroll state
    payrolls: [] as Payroll[],
    errors: [] as { employee_id: number; employee_name: string; error: string }[],
    availableEmployees: [] as Employee[],
    selectedEmployeeIds: [] as number[],
    currentPayrollDates: null as CurrentPayrollDates | null,
    
    // Remittance state
    remittances: [] as Remittance[],
    remittanceYears: [] as number[],
    currentRemittanceSummary: null as RemittanceSummary | null,
    
    loading: false,
    error: null as string | null
  }),
  
  getters: {
    eors: (state) => state.errors.length > 0,
       
    remittanceTotal: (state) =>
      state.remittances.reduce((sum, r) => sum + (r.grand_total ?? 0), 0),
    
    payrollTotal: (state) => {
      const grossPay = state.payrolls.reduce((sum, p) => sum + Number(p.gross_pay ?? 0), 0)
      const cppTotal = state.payrolls.reduce((sum, p) => sum + Number(p.deductions?.cpp ?? 0), 0)
      const cpp2Total = state.payrolls.reduce((sum, p) => sum + Number(p.deductions?.cpp2 ?? 0), 0)
      const eiTotal = state.payrolls.reduce((sum, p) => sum + Number(p.deductions?.ei ?? 0), 0)
      const federalTaxTotal = state.payrolls.reduce((sum, p) => sum + Number(p.deductions?.federal_tax ?? 0), 0)
      const provincialTaxTotal = state.payrolls.reduce((sum, p) => sum + Number(p.deductions?.provincial_tax ?? 0), 0)
      const additionalDeductionsTotal = state.payrolls.reduce((sum, p) => sum + Number(p.additional_deductions ?? 0), 0)
      const totalDeductions = cppTotal + cpp2Total + eiTotal + federalTaxTotal + provincialTaxTotal + additionalDeductionsTotal
      const netPay = state.payrolls.reduce((sum, p) => sum + Number(p.net_pay ?? 0), 0)
      
      return {
        count: state.payrolls.length,
        grossPay,
        cppTotal,
        cpp2Total,
        eiTotal,
        federalTaxTotal,
        provincialTaxTotal,
        additionalDeductionsTotal,
        totalDeductions,
        netPay
      }
    }
  },
  
  actions: {
    // Current Payroll Actions
    async createPayroll(input: CurrentPayrollInput) {
      this.loading = true
      this.error = null
      this.errors = []
      try {
        const result: CurrentPayrollResult = await payrollApi.createPayroll(input)
        this.payrolls = result.payrolls
        this.errors = result.errors
        // Set current payroll dates from input
        this.currentPayrollDates = {
          pay_period_start: input.pay_period_start,
          pay_period_end: input.pay_period_end,
          pay_date: input.pay_date,
          pay_period_number: input.pay_period_number,
          total_pay_periods: input.total_pay_periods
        }
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchAvailableEmployees(payPeriodStart: string, payPeriodEnd: string) {
      this.loading = true
      this.error = null
      try {
        this.availableEmployees = await payrollApi.getAvailableEmployeesForCurrentPayroll(
          payPeriodStart,
          payPeriodEnd
        )
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async addEmployeesToPayroll(input: CurrentPayrollInput) {
      this.loading = true
      this.error = null
      try {
        const result: CurrentPayrollResult = await payrollApi.createPayroll(input)
        // Append new payrolls
        this.payrolls = [...this.payrolls, ...result.payrolls]
        this.errors = [...this.errors, ...result.errors]
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async postCurrentToHistory(payrollIds: number[]): Promise<number[]> {
      this.loading = true
      this.error = null
      try {
        const newIds: number[] = await payrollApi.postCurrentToHistory(payrollIds)
        // Remove posted payrolls
        this.payrolls = this.payrolls.filter(
          p => !payrollIds.includes(p.id!)
        )
        return newIds
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async removeFromPayroll(payrollId: number) {
      this.loading = true
      this.error = null
      try {
        await payrollApi.deletePayroll(payrollId)
        this.payrolls = this.payrolls.filter(p => p.id !== payrollId)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    clearPayroll() {
      this.payrolls = []
      this.errors = []
      this.selectedEmployeeIds = []
      this.currentPayrollDates = null
    },

    async resetCurrentPayroll() {
      this.loading = true
      this.error = null
      try {
        await payrollApi.clearCurrentPayroll()
        this.clearPayroll()
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async loadCurrentPayroll() {
      this.loading = true
      this.error = null
      try {
        const dates = await payrollApi.getCurrentPayrollDates()
        if (dates) {
          this.currentPayrollDates = dates
          this.payrolls = await payrollApi.listCurrentPayroll()
          this.errors = []
        } else {
          this.currentPayrollDates = null
          this.clearPayroll()
        }
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    // Remittance Actions
    async fetchRemittanceSummary(cutoffDate: string) {
      this.loading = true
      this.error = null
      try {
        this.currentRemittanceSummary = await remittanceApi.getRemittanceSummary(cutoffDate)
        return this.currentRemittanceSummary
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async createRemittance(cutoffDate: string, craConfirmation?: string) {
      this.loading = true
      this.error = null
      try {
        const id = await remittanceApi.createRemittance(cutoffDate, craConfirmation)
        await this.fetchRemittances()
        this.currentRemittanceSummary = null
        return id
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchRemittanceYears() {
      this.loading = true
      this.error = null
      try {
        this.remittanceYears = await remittanceApi.getRemittanceYears()
        return this.remittanceYears
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchRemittances(year?: number) {
      this.loading = true
      this.error = null
      try {
        this.remittances = await remittanceApi.listRemittances(year)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async deleteRemittance(id: number) {
      this.loading = true
      this.error = null
      try {
        await remittanceApi.deleteRemittance(id)
        await this.fetchRemittances()
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async generateRemittanceReport(remittanceId: number, outputDir: string = 'reports') {
      this.loading = true
      this.error = null
      const appStore = useAppStore()
      try {
        const result = await reportsApi.generateRemittanceReport(remittanceId, outputDir)
        appStore.showNotification(`Remittance report generated: ${result}`, 'success')
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        appStore.showNotification(`Failed to generate report: ${errorMsg}`, 'error')
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async exportCurrentPayrollCsv(outputPath: string) {
      this.loading = true
      this.error = null
      try {
        const result = await payrollApi.exportCurrentPayrollCsv(outputPath)
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async importCurrentPayrollCsv(
      filePath: string,
      payPeriodStart?: string,
      payPeriodEnd?: string,
      payDate?: string
    ) {
      this.loading = true
      this.error = null
      this.errors = []
      try {
        const result = await payrollApi.importCurrentPayrollCsv(
          filePath,
          payPeriodStart,
          payPeriodEnd,
          payDate
        )
        // Merge imported payrolls: replace existing entries (by employee_id) and add new ones
        const updatedPayrolls = [...this.payrolls]
        for (const payroll of result.payrolls) {
          const existingIndex = updatedPayrolls.findIndex(p => p.employee_id === payroll.employee_id)
          if (existingIndex >= 0) {
            // Replace existing payroll for this employee (update case)
            updatedPayrolls[existingIndex] = payroll
          } else {
            // Add new payroll (create case)
            updatedPayrolls.push(payroll)
          }
        }
        this.payrolls = updatedPayrolls
        this.errors = [...this.errors, ...result.errors]
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    clearRemittanceSummary() {
      this.currentRemittanceSummary = null
    }
  }
})
