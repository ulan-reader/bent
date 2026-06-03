<template>
  <div>
    <Topbar>
      <div class="admin-bar">
        <span class="admin-name">👔 {{ adminName }}</span>
        <button class="logout-btn" @click="logout">Выйти</button>
      </div>
    </Topbar>

    <div class="main">
      <div class="top-row">
        <div class="section-label">Обзор обращений</div>
        <button class="refresh-btn" @click="loadSubmissions" :disabled="loading">⟳ Обновить</button>
      </div>

      <!-- Stats -->
      <div class="dashboard">
        <div class="stat">
          <div class="num">{{ stats.total }}</div>
          <div class="lbl">Всего обращений</div>
        </div>
        <div class="stat">
          <div class="num red">{{ stats.new }}</div>
          <div class="lbl">Новых</div>
        </div>
        <div class="stat">
          <div class="num orange">{{ stats.in_progress }}</div>
          <div class="lbl">В работе</div>
        </div>
        <div class="stat">
          <div class="num green">{{ stats.resolved }}</div>
          <div class="lbl">Решено</div>
        </div>
      </div>

      <!-- Filters -->
      <div class="filters card">
        <div class="filter-group">
          <label>Тип</label>
          <div class="filter-tabs">
            <button
              v-for="t in typeOptions"
              :key="t.value"
              class="filter-tab"
              :class="{ active: filter.type === t.value }"
              @click="setFilter('type', t.value)"
            >{{ t.label }}</button>
          </div>
        </div>
        <div class="filter-group">
          <label>Статус</label>
          <div class="filter-tabs">
            <button
              v-for="s in statusOptions"
              :key="s.value"
              class="filter-tab"
              :class="{ active: filter.status === s.value }"
              @click="setFilter('status', s.value)"
            >{{ s.label }}</button>
          </div>
        </div>
      </div>

      <!-- List -->
      <div class="card submissions-card">
        <div v-if="loading" class="loading-state">Загружаем…</div>
        <div v-else-if="filtered.length === 0" class="empty-state">Нет обращений по выбранным фильтрам</div>
        <div
          v-for="item in filtered"
          :key="item.id"
          class="item-row"
          @click="openDetail(item)"
        >
          <div class="status-dot" :class="dotClass(item.status)" />
          <div class="item-meta">
            <div class="title">{{ item.category }} — {{ item.department }}</div>
            <div class="sub">
              {{ formatDate(item.created_at) }} ·
              {{ item.type === 'employee' ? 'Сотрудник' : 'Инспектор ТБ' }}
              <span class="badge" :class="badgeClass(item.status)">{{ statusLabel(item.status) }}</span>
            </div>
          </div>
          <span class="arrow">›</span>
        </div>
      </div>
    </div>

    <!-- Detail modal -->
    <div v-if="detail" class="modal-overlay" @click.self="detail = null">
      <div class="modal">
        <div class="modal-header">
          <h3>Обращение #{{ detail.id }}</h3>
          <button class="close-btn" @click="detail = null">✕</button>
        </div>
        <div class="modal-body">
          <div class="detail-row"><span>Тип:</span> {{ detail.type === 'employee' ? 'Сотрудник' : 'Инспектор ТБ' }}</div>
          <div class="detail-row"><span>Подразделение:</span> {{ detail.department }}</div>
          <div class="detail-row"><span>Категория:</span> {{ detail.category }}</div>
          <div class="detail-row"><span>Дата:</span> {{ formatDate(detail.created_at) }}</div>
          <div class="detail-row"><span>Статус:</span>
            <span class="badge" :class="badgeClass(detail.status)">{{ statusLabel(detail.status) }}</span>
          </div>
          <div class="detail-text">
            <span>Текст обращения:</span>
            <p>{{ detail.text }}</p>
          </div>
          <div v-if="detail.file_url" class="detail-row">
            <span>Файл:</span>
            <a :href="detail.file_url" target="_blank" class="file-link">Открыть файл →</a>
          </div>
          <div v-if="detail.reject_reason" class="detail-row">
            <span>Причина отклонения:</span> {{ detail.reject_reason }}
          </div>
        </div>
        <div class="modal-footer">
          <div class="status-actions">
            <button
              v-for="s in changeableStatuses"
              :key="s.value"
              class="btn"
              :class="s.btnClass"
              :disabled="detail.status === s.value || statusLoading"
              @click="changeStatus(s.value)"
            >{{ s.label }}</button>
          </div>
          <div v-if="statusError" class="error-msg">{{ statusError }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '../api/index.js'
import Topbar from '../components/Topbar.vue'

const router = useRouter()
const adminToken = sessionStorage.getItem('admin_token')
const adminName = sessionStorage.getItem('admin_name') || 'Администратор'
const adminRole = sessionStorage.getItem('admin_role') || 'admin'

const submissions = ref([])
const loading = ref(false)
const detail = ref(null)
const statusLoading = ref(false)
const statusError = ref('')

const filter = ref({ type: 'all', status: 'all' })

const typeOptions = [
  { value: 'all', label: 'Все' },
  { value: 'employee', label: 'Сотрудник' },
  { value: 'inspector', label: 'Инспектор ТБ' },
]

const statusOptions = [
  { value: 'all', label: 'Все' },
  { value: 'new', label: 'Новые' },
  { value: 'in_progress', label: 'В работе' },
  { value: 'resolved', label: 'Решено' },
  { value: 'rejected', label: 'Отклонено' },
]

const changeableStatuses = [
  { value: 'in_progress', label: '🔄 В работу', btnClass: 'btn-orange' },
  { value: 'resolved', label: '✓ Решено', btnClass: 'btn-green' },
  { value: 'rejected', label: '✕ Отклонить', btnClass: 'btn-danger' },
]

const filtered = computed(() => {
  return submissions.value.filter(s => {
    const typeOk = filter.value.type === 'all' || s.type === filter.value.type
    const statusOk = filter.value.status === 'all' || s.status === filter.value.status
    return typeOk && statusOk
  })
})

const stats = computed(() => ({
  total: submissions.value.length,
  new: submissions.value.filter(s => s.status === 'new').length,
  in_progress: submissions.value.filter(s => s.status === 'in_progress').length,
  resolved: submissions.value.filter(s => s.status === 'resolved').length,
}))

async function loadSubmissions() {
  loading.value = true
  try {
    const params = {}
    if (adminRole === 'hr') params.type = 'employee'
    if (adminRole === 'safety_engineer') params.type = 'inspector'
    const data = await api.getSubmissions(params, adminToken)
    submissions.value = data
  } catch (e) {
    if (e.status === 401) logout()
  } finally {
    loading.value = false
  }
}

function setFilter(key, val) {
  filter.value[key] = val
}

function openDetail(item) {
  detail.value = { ...item }
  statusError.value = ''
}

async function changeStatus(newStatus) {
  statusLoading.value = true
  statusError.value = ''
  try {
    await api.updateStatus(detail.value.id, newStatus, null, adminToken)
    const idx = submissions.value.findIndex(s => s.id === detail.value.id)
    if (idx !== -1) submissions.value[idx].status = newStatus
    detail.value.status = newStatus
  } catch (e) {
    statusError.value = 'Не удалось изменить статус'
  } finally {
    statusLoading.value = false
  }
}

function logout() {
  sessionStorage.removeItem('admin_token')
  sessionStorage.removeItem('admin_role')
  sessionStorage.removeItem('admin_name')
  router.push('/admin/login')
}

function dotClass(status) {
  return { 'dot-new': status === 'new', 'dot-work': status === 'in_progress', 'dot-done': status === 'resolved', 'dot-rejected': status === 'rejected' }
}

function badgeClass(status) {
  return { 'badge-new': status === 'new', 'badge-work': status === 'in_progress', 'badge-done': status === 'resolved', 'badge-rejected': status === 'rejected' }
}

function statusLabel(status) {
  const map = { new: 'Новое', in_progress: 'В работе', resolved: 'Решено', rejected: 'Отклонено' }
  return map[status] || status
}

function formatDate(dt) {
  if (!dt) return '—'
  const d = new Date(dt)
  return d.toLocaleString('ru-RU', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' })
}

onMounted(loadSubmissions)
</script>

<style scoped>
.main { max-width: 860px; margin: 0 auto; padding: 32px 16px; }
.top-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; }
.section-label {
  font-size: 11px; font-weight: 500; letter-spacing: 0.08em;
  color: var(--color-text-tertiary); text-transform: uppercase;
}
.refresh-btn {
  background: none; border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md); padding: 5px 12px;
  font-size: 12px; color: var(--color-text-secondary); cursor: pointer;
}
.refresh-btn:hover { background: var(--color-background-secondary); }
.admin-bar { display: flex; align-items: center; gap: 10px; font-size: 13px; color: var(--color-text-secondary); }
.logout-btn {
  background: none; border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md); padding: 4px 10px;
  font-size: 12px; color: var(--color-text-secondary); cursor: pointer;
}
.logout-btn:hover { background: var(--color-background-secondary); }
.dashboard { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; margin-bottom: 12px; }
.stat {
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md); padding: 14px 16px;
}
.stat .num { font-size: 24px; font-weight: 500; color: var(--color-text-primary); }
.stat .num.red { color: var(--color-red); }
.stat .num.orange { color: var(--color-orange); }
.stat .num.green { color: var(--color-green); }
.stat .lbl { font-size: 12px; color: var(--color-text-secondary); margin-top: 2px; }
.card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg); padding: 16px;
  margin-bottom: 12px;
}
.filters { display: flex; gap: 24px; }
.filter-group { display: flex; align-items: center; gap: 8px; }
.filter-group label { font-size: 12px; color: var(--color-text-tertiary); white-space: nowrap; }
.filter-tabs { display: flex; gap: 4px; }
.filter-tab {
  padding: 4px 10px; border-radius: 20px; font-size: 12px;
  border: 0.5px solid var(--color-border-tertiary);
  background: var(--color-background-secondary); color: var(--color-text-secondary);
  cursor: pointer; transition: all 0.15s; font-family: inherit;
}
.filter-tab.active { background: var(--color-blue-light); border-color: #185FA5; color: #185FA5; }
.submissions-card { padding: 0; overflow: hidden; }
.item-row {
  display: flex; align-items: flex-start; gap: 12px;
  padding: 14px 16px; border-bottom: 0.5px solid var(--color-border-tertiary);
  cursor: pointer; transition: background 0.1s;
}
.item-row:last-child { border-bottom: none; }
.item-row:hover { background: var(--color-background-secondary); }
.status-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; margin-top: 6px; }
.dot-new { background: var(--color-red); }
.dot-work { background: var(--color-orange); }
.dot-done { background: var(--color-green); }
.dot-rejected { background: #aaa; }
.item-meta { flex: 1; }
.item-meta .title { font-size: 14px; font-weight: 500; color: var(--color-text-primary); }
.item-meta .sub { font-size: 12px; color: var(--color-text-secondary); margin-top: 2px; }
.arrow { color: var(--color-text-tertiary); font-size: 18px; align-self: center; }
.badge {
  display: inline-block; padding: 2px 8px; border-radius: 4px;
  font-size: 11px; font-weight: 500; margin-left: 6px;
}
.badge-new { background: #FCEBEB; color: #A32D2D; }
.badge-work { background: #FAEEDA; color: #854F0B; }
.badge-done { background: var(--color-green-light); color: #3B6D11; }
.badge-rejected { background: #eee; color: #888; }
.loading-state, .empty-state { padding: 32px; text-align: center; color: var(--color-text-secondary); font-size: 14px; }
/* Modal */
.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.35);
  display: flex; align-items: center; justify-content: center; z-index: 200; padding: 16px;
}
.modal {
  background: var(--color-background-primary);
  border-radius: var(--border-radius-lg); width: 100%; max-width: 540px;
  max-height: 90vh; overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0,0,0,0.15);
}
.modal-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 20px 24px 16px; border-bottom: 0.5px solid var(--color-border-tertiary);
}
.modal-header h3 { font-size: 16px; font-weight: 500; }
.close-btn {
  background: none; border: none; font-size: 16px;
  color: var(--color-text-tertiary); cursor: pointer; padding: 4px;
}
.close-btn:hover { color: var(--color-text-primary); }
.modal-body { padding: 20px 24px; }
.detail-row {
  display: flex; gap: 8px; font-size: 13px;
  margin-bottom: 10px; color: var(--color-text-secondary);
}
.detail-row span:first-child { font-weight: 500; color: var(--color-text-primary); min-width: 120px; }
.detail-text { margin-bottom: 10px; }
.detail-text span { font-size: 13px; font-weight: 500; color: var(--color-text-primary); }
.detail-text p {
  font-size: 13px; color: var(--color-text-secondary); line-height: 1.6;
  margin-top: 6px; padding: 12px; background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
}
.file-link { color: #185FA5; font-size: 13px; }
.modal-footer {
  padding: 16px 24px 20px;
  border-top: 0.5px solid var(--color-border-tertiary);
}
.status-actions { display: flex; gap: 8px; flex-wrap: wrap; }
.btn {
  padding: 8px 16px; border-radius: var(--border-radius-md);
  font-size: 13px; font-weight: 500; border: none; cursor: pointer;
  transition: all 0.15s; font-family: inherit;
}
.btn:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-orange { background: #FEF3E0; color: #854F0B; border: 0.5px solid #EF9F27; }
.btn-orange:hover:not(:disabled) { background: #FAEEDA; }
.btn-green { background: var(--color-green-light); color: #3B6D11; border: 0.5px solid #639922; }
.btn-green:hover:not(:disabled) { background: #d4ebba; }
.btn-danger { background: var(--color-red-light); color: #A32D2D; border: 0.5px solid var(--color-red); }
.btn-danger:hover:not(:disabled) { background: #f7d5d5; }
.error-msg {
  color: var(--color-red); font-size: 13px; padding: 8px 12px;
  background: var(--color-red-light); border-radius: var(--border-radius-md); margin-top: 10px;
}
@media (max-width: 640px) {
  .dashboard { grid-template-columns: repeat(2, 1fr); }
  .filters { flex-direction: column; gap: 12px; }
}
</style>
