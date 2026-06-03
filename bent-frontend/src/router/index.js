import { createRouter, createWebHistory } from 'vue-router'

import FormView from '../views/FormView.vue'
import FormExpired from '../views/FormExpired.vue'
import FormUsed from '../views/FormUsed.vue'
import InspectorLogin from '../views/InspectorLogin.vue'
import InspectorForm from '../views/InspectorForm.vue'
import AdminLogin from '../views/AdminLogin.vue'
import AdminPanel from '../views/AdminPanel.vue'
import NotFound from '../views/NotFound.vue'

const routes = [
  { path: '/form', component: FormView },
  { path: '/form/expired', component: FormExpired },
  { path: '/form/used', component: FormUsed },
  { path: '/inspector/login', component: InspectorLogin },
  { path: '/inspector', component: InspectorForm },
  { path: '/admin/login', component: AdminLogin },
  { path: '/admin', component: AdminPanel },
  { path: '/:pathMatch(.*)*', component: NotFound },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

// Guard for /inspector
router.beforeEach((to) => {
  const inspectorToken = sessionStorage.getItem('inspector_token')
  const adminToken = sessionStorage.getItem('admin_token')

  if (to.path === '/inspector' && !inspectorToken) {
    return '/inspector/login'
  }
  if (to.path === '/admin' && !adminToken) {
    return '/admin/login'
  }
})

export default router
