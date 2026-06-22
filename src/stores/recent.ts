// Recent databases store - manages the list of recently accessed databases
import { defineStore } from 'pinia'
import { recentApi } from '@/services/api'
import type { RecentDatabase } from '@/types/recent'
import { getErrorMessage } from '@/utils/error'

export const useRecentStore = defineStore('recent', {
  state: () => ({
    recentDatabases: [] as RecentDatabase[],
    loading: false,
    error: null as string | null
  }),

  getters: {
    hasRecent: (state) => state.recentDatabases.length > 0
  },

  actions: {
    async fetchRecent() {
      this.loading = true
      this.error = null
      try {
        this.recentDatabases = await recentApi.getRecentDatabases()
      } catch (e) {
        this.error = getErrorMessage(e)
        console.error('Failed to fetch recent databases:', e)
      } finally {
        this.loading = false
      }
    },

    async addRecent(path: string, companyName: string | null) {
      try {
        this.recentDatabases = await recentApi.addRecentDatabase(path, companyName)
      } catch (e) {
        this.error = getErrorMessage(e)
        console.error('Failed to add recent database:', e)
      }
    },

    async removeRecent(path: string) {
      try {
        this.recentDatabases = await recentApi.removeRecentDatabase(path)
      } catch (e) {
        this.error = getErrorMessage(e)
        console.error('Failed to remove recent database:', e)
      }
    },

    async updateCompany(path: string, companyName: string | null) {
      try {
        this.recentDatabases = await recentApi.updateRecentDatabaseCompany(path, companyName)
      } catch (e) {
        this.error = getErrorMessage(e)
        console.error('Failed to update recent database company:', e)
      }
    }
  }
})
