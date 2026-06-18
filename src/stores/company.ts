// Company store - manages company information
import { defineStore } from 'pinia'
import { companyApi } from '@/services/api'
import type { Company } from '@/types/company'
import { getErrorMessage } from '@/utils/error'

export const useCompanyStore = defineStore('company', {
  state: () => ({
    company: null as Company | null,
    loading: false,
    error: null as string | null
  }),
  
  getters: {
    hasCompany: (state) => state.company !== null,
    companyName: (state) => state.company?.name ?? 'No Company Set',
    businessNumber: (state) => state.company?.business_number ?? ''
  },
  
  actions: {
    async fetchCompany() {
      this.loading = true
      this.error = null
      try {
        this.company = await companyApi.getCompany()
      } catch (e) {
        this.error = getErrorMessage(e)
        throw e
      } finally {
        this.loading = false
      }
    },
    
    async saveCompany(companyInput: any) {
      this.loading = true
      this.error = null
      try {
        const flatCompany = {
          id: companyInput.id,
          name: companyInput.name,
          business_number: companyInput.business_number || null,
          address: (() => {
            const parts: string[] = [];
            if (companyInput.address?.street?.trim()) {
              parts.push(companyInput.address.street.trim());
            }
            if (companyInput.address?.city?.trim()) {
              parts.push(companyInput.address.city.trim());
            }
            if (companyInput.address?.postal_code?.trim()) {
              parts.push(companyInput.address.postal_code.trim());
            }
            return parts.join(', ');
          })(),
          province: companyInput.address.province,
          created_at: companyInput.created_at || new Date().toISOString(),
        };
        (companyApi.saveCompany as any)(flatCompany);
        await this.fetchCompany();
      } catch (e) {
        this.error = getErrorMessage(e)
        throw e
      } finally {
        this.loading = false
      }
    }
  }
})
