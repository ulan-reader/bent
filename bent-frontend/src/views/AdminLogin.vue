<template>
  <div>
    <Topbar />
    <div class="main">
      <div class="section-label">Панель управления</div>
      <div class="card">
        <h2 class="card-title">Вход</h2>

        <div class="field">
          <label>Email</label>
          <input v-model="email" type="email" placeholder="admin@bent.kz" @keydown.enter="login" />
        </div>
        <div class="field">
          <label>Пароль</label>
          <input v-model="password" type="password" placeholder="••••••••" @keydown.enter="login" />
        </div>

        <div v-if="error" class="error-msg">{{ error }}</div>

        <div class="btn-row">
          <button class="btn btn-primary" @click="login" :disabled="loading">
            {{ loading ? 'Входим…' : 'Войти' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '../api/index.js'
import Topbar from '../components/Topbar.vue'

const router = useRouter()
const email = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

async function login() {
  error.value = ''
  if (!email.value || !password.value) {
    error.value = 'Введите email и пароль'
    return
  }
  loading.value = true
  try {
    const res = await api.adminLogin(email.value, password.value)
    sessionStorage.setItem('admin_token', res.token)
    sessionStorage.setItem('admin_role', res.role)
    sessionStorage.setItem('admin_name', res.name || '')
    router.push('/admin')
  } catch (e) {
    error.value = e.status === 401 ? 'Неверный email или пароль' : 'Ошибка входа. Попробуйте снова.'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.main { max-width: 420px; margin: 0 auto; padding: 32px 16px; }
.section-label {
  font-size: 11px; font-weight: 500; letter-spacing: 0.08em;
  color: var(--color-text-tertiary); text-transform: uppercase; margin-bottom: 16px;
}
.card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg); padding: 28px;
}
.card-title { font-size: 16px; font-weight: 500; margin-bottom: 20px; }
.field { margin-bottom: 16px; }
label { display: block; font-size: 13px; font-weight: 500; color: var(--color-text-secondary); margin-bottom: 6px; }
input {
  width: 100%; font-family: var(--font-sans); font-size: 14px;
  color: var(--color-text-primary); background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md); padding: 8px 12px; outline: none; transition: border-color 0.15s;
}
input:focus { border-color: #378ADD; box-shadow: 0 0 0 3px rgba(55,138,221,.12); }
.btn-row { display: flex; justify-content: flex-end; margin-top: 20px; }
.btn {
  width: 100%; padding: 10px 20px; border-radius: var(--border-radius-md);
  font-size: 14px; font-weight: 500; border: none; transition: all 0.15s;
}
.btn-primary { background: #1B3A6B; color: #fff; }
.btn-primary:hover { background: #2E5FA3; }
.btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
.error-msg {
  color: var(--color-red); font-size: 13px; padding: 8px 12px;
  background: var(--color-red-light); border-radius: var(--border-radius-md); margin-bottom: 8px;
}
</style>
