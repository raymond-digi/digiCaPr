// Payroll store - manages payroll data and operations
import { defineStore } from 'pinia'
import { payrollApi } from '@/services/api'
import type { Payroll, PayrollCalculationInput, YtdTotals, PayrollHistoryFilters, PayrollPeriod } from '@/types/payroll'
import { getErrorMessage } from '@/utils/error'

export const usePayrollStore = defineStore('payroll', {
  state: () => ({
    payrolls: [] as Payroll[],
    currentPayroll: null as Payroll | null,
    employeePayrolls: [] as Payroll[],
    ytdTotals: null as YtdTotals | null,
    loading: false,
    error: null as string | null,
    totalCount: 0 as number,
    currentFilters: null as PayrollHistoryFilters | null,
    // Hierarchy state for payroll history navigation
    years: [] as number[],
    periods: [] as PayrollPeriod[],
    selectedYear: null as number | null,
    selectedPeriod: null as PayrollPeriod | null,
    selectedEmployeeId: null as number | null
  }),
  
  getters: {
    getPayrollById: (state) => (id: number) =>
      state.payrolls.find(p => p.id === id),
    
    getPayrollsByEmployee: (state) => (employeeId: number) =>
      state.payrolls.filter(p => p.employee_id === employeeId),
    
    totalGrossForPeriod: (state) => 
      state.payrolls.reduce((sum, p) => sum + p.gross_pay, 0),
    
    totalNetForPeriod: (state) => 
      state.payrolls.reduce((sum, p) => sum + p.net_pay, 0)
  },
  
  actions: {
    async calculatePayroll(input: PayrollCalculationInput) {
      this.loading = true
      this.error = null
      try {
        this.currentPayroll = await payrollApi.calculatePayroll(input)
        return this.currentPayroll
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async savePayroll(payroll: Payroll) {
      this.loading = true
      this.error = null
      try {
        const id = await payrollApi.savePayroll(payroll)
        await this.fetchPayrolls()
        return id
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchPayrolls(filters?: PayrollHistoryFilters) {
      this.loading = true
      this.error = null
      try {
        this.currentFilters = filters || null
        const result = await payrollApi.listPayrollHistory(filters)
        this.payrolls = result.payrolls
        this.totalCount = result.total_count
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async fetchYears(employeeId?: number) {
      this.loading = true
      this.error = null
      try {
        const result = await payrollApi.listPayrollYears(employeeId)
        this.years = result.sort((a, b) => b - a) // Sort DESC
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async fetchPeriods(year: number, employeeId?: number) {
      this.loading = true
      this.error = null
      try {
        this.periods = await payrollApi.listPayrollPeriods(year, employeeId)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async fetchPayrollsWithHierarchy(filters?: PayrollHistoryFilters) {
      // Build filters with current hierarchy scope
      const enhancedFilters: PayrollHistoryFilters = { ...filters }

      // Inject scope from selectedPeriod if any
      if (this.selectedPeriod) {
        enhancedFilters.pay_date_from = this.selectedPeriod.pay_date
        enhancedFilters.pay_date_to = this.selectedPeriod.pay_date
      } else if (this.selectedYear && !filters?.pay_date_from && !filters?.pay_date_to) {
        // If year selected but no period, use full year range
        enhancedFilters.pay_date_from = `${this.selectedYear}-01-01`
        enhancedFilters.pay_date_to = `${this.selectedYear}-12-31`
      }

      // Inject employee scope if selected
      if (this.selectedEmployeeId) {
        enhancedFilters.employee_id = this.selectedEmployeeId
      }

      await this.fetchPayrolls(enhancedFilters)
    },

    clearHierarchy() {
      this.years = []
      this.periods = []
      this.selectedYear = null
      this.selectedPeriod = null
      this.selectedEmployeeId = null
    },

    async fetchEmployeePayroll(employeeId: number) {
      this.loading = true
      this.error = null
      try {
        this.employeePayrolls = await payrollApi.listEmployeePayroll(employeeId)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchPayroll(id: number) {
      this.loading = true
      this.error = null
      try {
        this.currentPayroll = await payrollApi.getPayroll(id)
        return this.currentPayroll
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchYtdTotals(employeeId: number, year: number) {
      this.loading = true
      this.error = null
      try {
        this.ytdTotals = await payrollApi.getYtdTotals(employeeId, year)
        return this.ytdTotals
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async deletePayroll(id: number) {
      this.loading = true
      this.error = null
      try {
        await payrollApi.deletePayroll(id)
        await this.fetchPayrolls()
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async importHistoryPayrollCsv(filePath: string) {
      this.loading = true
      this.error = null
      try {
        const result = await payrollApi.importHistoryPayrollCsv(filePath)
        await this.fetchPayrolls()
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async exportHistoryPayrollCsv(
      outputPath: string,
      employeeId?: number | null,
      payDateFrom?: string | null,
      payDateTo?: string | null,
      searchTerm?: string | null
    ) {
      this.loading = true
      this.error = null
      try {
        const result = await payrollApi.exportHistoryPayrollCsv(
          outputPath,
          employeeId,
          payDateFrom,
          payDateTo,
          searchTerm
        )
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    clearCurrentPayroll() {
      this.currentPayroll = null
    },
    
    async deleteHistoryPayroll(id: number) {
      this.loading = true
      this.error = null
      try {
        await payrollApi.deleteHistoryPayroll(id)
        await this.fetchPayrolls(this.currentFilters || undefined)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async updateHistoryPayroll(payroll: Payroll) {
      this.loading = true
      this.error = null
      try {
        await payrollApi.updateHistoryPayroll(payroll)
        await this.fetchPayrolls(this.currentFilters || undefined)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async saveHistoryPayroll(payroll: Payroll) {
      this.loading = true
      this.error = null
      try {
        const id = await payrollApi.saveHistoryPayroll(payroll)
        await this.fetchPayrolls(this.currentFilters || undefined)
        return id
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async saveRawPayroll(payroll: Payroll) {
      this.loading = true
      this.error = null
      try {
        const id = await payrollApi.saveRawPayroll(payroll)
        await this.fetchPayrolls(this.currentFilters || undefined)
        return id
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    }
  }
})
