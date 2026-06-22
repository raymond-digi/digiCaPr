import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from '@/views/Dashboard.vue'
import WelcomeView from '@/views/WelcomeView.vue'
import { useAppStore } from '@/stores/app'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: Dashboard
    },
    {
      path: '/welcome',
      name: 'welcome',
      component: WelcomeView
    },
    {
      path: '/employees',
      name: 'employees',
      component: () => import('@/views/Employee.vue')
    },
    {
      path: '/payroll',
      name: 'payroll',
      component: () => import('@/views/Payroll.vue')
    },
    {
      path: '/history-period',
      name: 'history-period',
      component: () => import('@/views/HistoryPeriod.vue')
    },
    {
      path: '/history-employee',
      name: 'history-employee',
      component: () => import('@/views/HistoryEmployee.vue')
    },
    {
      path: '/remittance',
      name: 'remittance',
      component: () => import('@/views/Remittance.vue')
    },
    {
      path: '/vacation',
      name: 'vacation',
      component: () => import('@/views/Vacation.vue')
    },
    {
      path: '/t4',
      name: 't4',
      component: () => import('@/views/T4.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue')
    }
  ]
})

// Navigation guard to check database status
router.beforeEach(async (to, from, next) => {
  const appStore = useAppStore()
  
  // Check database status on first load
  if (!appStore.isDatabaseOpen && from.name === undefined) {
    await appStore.checkDatabaseStatus()
  }
  
  // Allow navigation to settings and welcome without database
  if (to.name === 'settings' || to.name === 'welcome') {
    next()
    return
  }
  
  // Redirect to welcome if no database is open
  if (!appStore.isDatabaseOpen && to.name !== 'welcome') {
    next({ name: 'welcome' })
    return
  }
  
  next()
})

export default router
