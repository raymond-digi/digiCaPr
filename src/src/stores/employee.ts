// Employee store - manages employee data and operations
import { defineStore } from 'pinia'
import { employeeApi } from '@/services/api'
import type { Employee, PayRateHistory, EmploymentHistory } from '@/types/employee'
import { getErrorMessage } from '@/utils/error'

export const useEmployeeStore = defineStore('employee', {
  state: () => ({
    employees: [] as Employee[],
    currentEmployee: null as Employee | null,
    payRateHistory: [] as PayRateHistory[],
    employmentHistory: [] as EmploymentHistory[],
    loading: false,
    error: null as string | null
  }),
  
  getters: {
    activeEmployees: (state) => 
      state.employees.filter(e => e.is_active),
    
    inactiveEmployees: (state) => 
      state.employees.filter(e => !e.is_active),
    
    getEmployeeById: (state) => (id: number) =>
      state.employees.find(e => e.id === id),
    
    getEmployeeByNumber: (state) => (employeeNumber: string) =>
      state.employees.find(e => e.employee_number === employeeNumber)
  },
  
  actions: {
    async fetchEmployees() {
      this.loading = true
      this.error = null
      try {
        this.employees = await employeeApi.listEmployees()
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchActiveEmployees() {
      this.loading = true
      this.error = null
      try {
        this.employees = await employeeApi.listActiveEmployees()
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchEmployee(id: number) {
      this.loading = true
      this.error = null
      try {
        this.currentEmployee = await employeeApi.getEmployee(id)
        return this.currentEmployee
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async createEmployee(employee: Employee) {
      this.loading = true
      this.error = null
      try {
        const id = await employeeApi.createEmployee(employee)
        await this.fetchEmployees()
        return id
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async updateEmployee(employee: Employee) {
      this.loading = true
      this.error = null
      try {
        await employeeApi.updateEmployee(employee)
        await this.fetchEmployees()
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async deleteEmployee(id: number) {
      this.loading = true
      this.error = null
      try {
        await employeeApi.deleteEmployee(id)
        await this.fetchEmployees()
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async searchEmployees(query: string) {
      this.loading = true
      this.error = null
      try {
        this.employees = await employeeApi.searchEmployees(query)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchPayRateHistory(employeeId: number) {
      this.loading = true
      this.error = null
      try {
        this.payRateHistory = await employeeApi.getPayRateHistory(employeeId)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async fetchEmploymentHistory(employeeId: number) {
      this.loading = true
      this.error = null
      try {
        this.employmentHistory = await employeeApi.getEmploymentHistory(employeeId)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async exportEmployeesCsv(filePath: string) {
      this.loading = true
      this.error = null
      try {
        const count = await employeeApi.exportEmployeesCsv(filePath)
        return count
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    async importEmployeesCsv(filePath: string) {
      this.loading = true
      this.error = null
      try {
        const result = await employeeApi.importEmployeesCsv(filePath)
        await this.fetchEmployees()
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },
    
    clearCurrentEmployee() {
      this.currentEmployee = null
      this.payRateHistory = []
      this.employmentHistory = []
    }
  }
})
