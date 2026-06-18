// T4 store - manages T4 year-end slip generation and adjustments
import { defineStore } from 'pinia'
import { reportsApi, t4Api } from '@/services/api'
import { useAppStore } from '@/stores/app'
import { getErrorMessage } from '@/utils/error'
import type { T4SlipLegacy as T4Slip, T4BoxValue, T4SummaryData, T4BoxValueUpdate } from '@/types/t4'

export const useT4Store = defineStore('t4', {
  state: () => ({
    availableYears: [] as number[],
    t4Slips: [] as T4Slip[],
    selectedYear: null as number | null,
    summaryData: null as T4SummaryData | null,
    loading: false,
    error: null as string | null,
  }),

  getters: {
    totalEmploymentIncome: (state) =>
      state.t4Slips.reduce((sum, s) => sum + Number(s.employment_income ?? 0), 0),
    totalCpp: (state) =>
      state.t4Slips.reduce((sum, s) => sum + Number(s.cpp_contributions ?? 0), 0),
    totalEi: (state) =>
      state.t4Slips.reduce((sum, s) => sum + Number(s.ei_premiums ?? 0), 0),
    totalTax: (state) =>
      state.t4Slips.reduce((sum, s) => sum + Number(s.income_tax_deducted ?? 0), 0),
    totalEiInsurable: (state) =>
      state.t4Slips.reduce((sum, s) => sum + Number(s.ei_insurable_earnings ?? 0), 0),
    totalCppPensionable: (state) =>
      state.t4Slips.reduce((sum, s) => sum + Number(s.cpp_pensionable_earnings ?? 0), 0),
    employeeCount: (state) => state.t4Slips.length,
  },

  actions: {
    async fetchYears() {
      this.loading = true
      this.error = null
      try {
        this.availableYears = await t4Api.getT4Years()
        return this.availableYears
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async calculateForYear(year: number) {
      this.loading = true
      this.error = null
      try {
        this.t4Slips = await reportsApi.calculateT4ForYear(year)
        this.selectedYear = year
        // Also load summary data after calculation
        try {
          this.summaryData = await reportsApi.getT4Summary(year)
        } catch {
          this.summaryData = null
        }
        return this.t4Slips
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async loadT4sForYear(year: number) {
      this.loading = true
      this.error = null
      try {
        this.t4Slips = await t4Api.getT4SlipsForYear(year)
        this.selectedYear = year
        // Also load summary data
        try {
          this.summaryData = await reportsApi.getT4Summary(year)
        } catch {
          // Summary may not be available yet
          this.summaryData = null
        }
        return this.t4Slips
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async loadT4Summary(year: number) {
      this.loading = true
      this.error = null
      try {
        this.summaryData = await reportsApi.getT4Summary(year)
        return this.summaryData
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async generateT4Pdf(employeeId: number, year: number, outputPath: string) {
      this.loading = true
      this.error = null
      const appStore = useAppStore()
      try {
        const result = await reportsApi.generateT4(employeeId, year, outputPath)
        appStore.showNotification(`T4 PDF generated: ${result}`, 'success')
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        appStore.showNotification(`Failed to generate T4 PDF: ${errorMsg}`, 'error')
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async generateAllT4Pdfs(outputDir: string) {
      this.loading = true
      this.error = null
      const appStore = useAppStore()
      try {
        if (!this.selectedYear) throw new Error('No year selected')
        const result = await reportsApi.generatePayrollT4(this.selectedYear!, outputDir)
        appStore.showNotification(`Generated ${result.length} T4 PDF(s)`, 'success')
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        appStore.showNotification(`Failed to generate T4 PDFs: ${errorMsg}`, 'error')
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async generateT4SummaryPdf(year: number, outputPath: string) {
      this.loading = true
      this.error = null
      const appStore = useAppStore()
      try {
        const result = await reportsApi.generateT4SummaryPdf(year, outputPath)
        appStore.showNotification(`T4 Summary generated: ${result}`, 'success')
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        appStore.showNotification(`Failed to generate T4 Summary: ${errorMsg}`, 'error')
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async exportXml(year: number, outputPath: string) {
      this.loading = true
      this.error = null
      const appStore = useAppStore()
      try {
        const result = await reportsApi.exportT4Xml(year, outputPath)
        appStore.showNotification(`T4 XML exported: ${result}`, 'success')
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        appStore.showNotification(`Failed to export T4 XML: ${errorMsg}`, 'error')
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async exportCsv(year: number, outputPath: string) {
      this.loading = true
      this.error = null
      const appStore = useAppStore()
      try {
        const result = await reportsApi.exportT4Csv(year, outputPath)
        appStore.showNotification(`T4 CSV exported: ${result}`, 'success')
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        appStore.showNotification(`Failed to export T4 CSV: ${errorMsg}`, 'error')
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async listBoxValues(slipId: number) {
      this.loading = true
      this.error = null
      try {
        const boxValues = await t4Api.getT4BoxValues(slipId)
        return boxValues
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async saveBoxValue(boxValue: T4BoxValue) {
      this.loading = true
      this.error = null
      try {
        const id = await t4Api.saveT4BoxValue(boxValue)
        return id
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async createSlipVersion(employeeId: number, year: number) {
      this.loading = true
      this.error = null
      try {
        const slip = await t4Api.createT4SlipVersion(employeeId, year)
        return slip
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async fileSlip(slipId: number, filedBy: string) {
      this.loading = true
      this.error = null
      try {
        await t4Api.fileT4Slip(slipId, filedBy)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async lockSlip(slipId: number) {
      this.loading = true
      this.error = null
      try {
        await t4Api.lockT4Slip(slipId)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async unlockSlip(slipId: number) {
      this.loading = true
      this.error = null
      try {
        await t4Api.unlockT4Slip(slipId)
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    async updateBoxValues(update: T4BoxValueUpdate) {
      this.loading = true
      this.error = null
      try {
        const id = await t4Api.updateBoxValues(update)
        return id
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        throw new Error(errorMsg)
      } finally {
        this.loading = false
      }
    },

    clearSlips() {
      this.t4Slips = []
      this.selectedYear = null
      this.summaryData = null
    },
  },
})
