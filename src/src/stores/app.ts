// App-level state management
import { defineStore } from 'pinia'
import { databaseApi } from '@/services/api'
import { getErrorMessage } from '@/utils/error'

export const useAppStore = defineStore('app', {
  state: () => ({
    databasePath: null as string | null,
    isDatabaseOpen: false,
    loading: false,
    error: null as string | null,
    sidebarOpen: true,
    devMode: false,
    notification: {
      show: false,
      message: '',
      type: 'info' as 'success' | 'error' | 'info' | 'warning'
    }
  }),
  
  getters: {
    hasDatabase: (state) => state.isDatabaseOpen && state.databasePath !== null
  },
  
  actions: {
    async checkDatabaseStatus() {
      try {
        this.isDatabaseOpen = await databaseApi.isDatabaseOpen()
        if (this.isDatabaseOpen) {
          this.databasePath = await databaseApi.getCurrentDatabasePath()
        }
      } catch (e) {
        this.error = getErrorMessage(e)
      }
    },
    
    async createDatabase(path: string) {
      this.loading = true
      this.error = null
      try {
        const result = await databaseApi.createDatabase(path)
        this.databasePath = result
        this.isDatabaseOpen = true
        this.showNotification('Database created successfully', 'success')
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        this.showNotification(`Failed to create database: ${errorMsg}`, 'error')
        throw e
      } finally {
        this.loading = false
      }
    },
    
    async openDatabase(path: string) {
      this.loading = true
      this.error = null
      try {
        const result = await databaseApi.openDatabase(path)
        this.databasePath = result
        this.isDatabaseOpen = true
        this.showNotification('Database opened successfully', 'success')
        return result
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        this.showNotification(`Failed to open database: ${errorMsg}`, 'error')
        throw e
      } finally {
        this.loading = false
      }
    },
    
    async closeDatabase() {
      this.loading = true
      this.error = null
      try {
        await databaseApi.closeDatabase()
        this.databasePath = null
        this.isDatabaseOpen = false
        this.showNotification('Database closed', 'info')
      } catch (e) {
        const errorMsg = getErrorMessage(e)
        this.error = errorMsg
        this.showNotification(`Failed to close database: ${errorMsg}`, 'error')
        throw e
      } finally {
        this.loading = false
      }
    },
    
    showNotification(message: string, type: 'success' | 'error' | 'info' | 'warning' = 'info') {
      this.notification = {
        show: true,
        message,
        type
      }
    },
    
    hideNotification() {
      this.notification.show = false
    },
    
    toggleSidebar() {
      this.sidebarOpen = !this.sidebarOpen
    },
    
    toggleDevMode() {
      this.devMode = !this.devMode
      this.showNotification(
        `Developer Mode ${this.devMode ? 'Enabled' : 'Disabled'}`,
        this.devMode ? 'warning' : 'info'
      )
    }
  }
})
