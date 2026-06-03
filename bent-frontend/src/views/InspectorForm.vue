<template>
  <div>
    <Topbar>
      <div class="inspector-badge">
        🦺 {{ inspectorName || 'Инспектор ТБ' }}
        <button class="logout-btn" @click="logout">Выйти</button>
      </div>
    </Topbar>

    <div class="main">
      <div class="section-label">Фиксация нарушения ТБ</div>

      <!-- Step 1 -->
      <div v-if="step === 0">
        <div class="card">
          <StepIndicator :current="0" :steps="[1,2]" />

          <div class="field">
            <label>Подразделение / участок</label>
            <select v-model="form.department">
              <option value="">Выберите участок…</option>
              <optgroup label="БЕНТ">
                <option>АБК</option><option>БСЦ</option><option>РМЦ (БЕНТ)</option>
                <option>СБ</option><option>ШЦ</option><option>ЭРУ</option>
              </optgroup>
              <optgroup label="Автоклав">
                <option>Цех по газоблоку</option><option>Основное подразделение</option>
              </optgroup>
              <optgroup label="Fasteners Metel">
                <option>РМЦ (Fasteners)</option>
              </optgroup>
            </select>
          </div>

          <div class="field">
            <label>Категория нарушения</label>
            <TagSelector
              v-model="form.category"
              :tags="areas"
            />
          </div>

          <div v-if="error" class="error-msg">{{ error }}</div>

          <div class="btn-row">
            <button class="btn btn-primary" @click="nextStep">Далее →</button>
          </div>
        </div>
      </div>

      <!-- Step 2 -->
      <div v-else-if="step === 1">
        <div class="card">
          <StepIndicator :current="1" :steps="[1,2]" />

          <div class="field">
            <label>Фото или видео нарушения <span class="required">обязательно</span></label>
            <FileZone
              v-model:file="form.file"
              accept="image/*,video/*"
              icon="📸"
              label="Нажмите для загрузки фото/видео"
              hint="JPG, PNG, MP4 — до 50 МБ"
            />
          </div>

          <div class="field">
            <label>Комментарий к нарушению</label>
            <textarea
              v-model="form.text"
              placeholder="Опишите нарушение подробно (минимум 10 символов)…"
              rows="5"
            />
          </div>

          <div v-if="error" class="error-msg">{{ error }}</div>

          <div class="btn-row">
            <button class="btn btn-ghost" @click="step--">Назад</button>
            <button class="btn btn-danger" @click="submit" :disabled="submitting">
              {{ submitting ? 'Отправка…' : '⚠ Зафиксировать' }}
            </button>
          </div>
        </div>
      </div>

      <!-- Success -->
      <div v-else-if="step === 2">
        <div class="card">
          <div class="success-box">
            <div class="icon-circle red">🛡</div>
            <h2>Нарушение зафиксировано</h2>
            <p>Материалы переданы в чат руководства и инженеру ТБ.<br>Ожидайте реакцию в течение 24 часов.</p>
            <button class="btn btn-ghost" style="margin-top: 20px" @click="reset">Новая фиксация</button>
          </div>
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
import StepIndicator from '../components/StepIndicator.vue'
import TagSelector from '../components/TagSelector.vue'
import FileZone from '../components/FileZone.vue'

const router = useRouter()
const inspectorName = sessionStorage.getItem('inspector_name')
const inspectorToken = sessionStorage.getItem('inspector_token')

const step = ref(0)
const error = ref('')
const submitting = ref(false)

const areas = [
  'СИЗ', 'Работа на высоте', 'Электробезопасность', 'Пожарная безопасность',
  'ГПМ', 'Эксплуатация оборудования', 'Складирование и логистика',
  'Промсанитария и экология', 'Газовое хозяйство',
]

const form = ref({
  department: '',
  category: '',
  text: '',
  file: null,
})

function nextStep() {
  error.value = ''
  if (!form.value.department) { error.value = 'Выберите участок'; return }
  if (!form.value.category) { error.value = 'Выберите категорию нарушения'; return }
  step.value++
}

async function submit() {
  error.value = ''
  if (!form.value.file) { error.value = 'Прикрепите фото или видео нарушения'; return }
  if (form.value.text.trim().length < 10) { error.value = 'Комментарий должен быть не менее 10 символов'; return }

  submitting.value = true
  try {
    let file_url = null
    if (form.value.file) {
      file_url = await api.uploadFile(form.value.file, inspectorToken)
    }
    await api.submitInspector({
      type: 'inspector',
      department: form.value.department,
      category: form.value.category,
      text: form.value.text,
      file_url,
      channel: 'web',
    }, inspectorToken)
    step.value = 2
  } catch (e) {
    error.value = e.message || 'Ошибка при отправке. Попробуйте снова.'
  } finally {
    submitting.value = false
  }
}

function reset() {
  form.value = { department: '', category: '', text: '', file: null }
  step.value = 0
  error.value = ''
}

function logout() {
  sessionStorage.removeItem('inspector_token')
  sessionStorage.removeItem('inspector_name')
  router.push('/inspector/login')
}
</script>

<style scoped>
.main { max-width: 680px; margin: 0 auto; padding: 32px 16px; }
.section-label {
  font-size: 11px; font-weight: 500; letter-spacing: 0.08em;
  color: var(--color-text-tertiary); text-transform: uppercase; margin-bottom: 16px;
}
.inspector-badge {
  display: flex; align-items: center; gap: 10px;
  font-size: 13px; color: var(--color-text-secondary);
}
.logout-btn {
  background: none; border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md); padding: 4px 10px;
  font-size: 12px; color: var(--color-text-secondary); cursor: pointer;
}
.logout-btn:hover { background: var(--color-background-secondary); }
.card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 24px;
}
.field { margin-bottom: 20px; }
.field:last-child { margin-bottom: 0; }
label { display: block; font-size: 13px; font-weight: 500; color: var(--color-text-secondary); margin-bottom: 6px; }
.required { font-weight: 400; color: var(--color-red); margin-left: 4px; }
select, textarea {
  width: 100%; font-family: var(--font-sans); font-size: 14px;
  color: var(--color-text-primary); background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md); padding: 8px 12px; outline: none;
  transition: border-color 0.15s;
}
select {
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat; background-position: right 12px center; padding-right: 32px;
}
select:focus, textarea:focus { border-color: #378ADD; box-shadow: 0 0 0 3px rgba(55,138,221,.12); }
textarea { resize: vertical; line-height: 1.6; }
.btn-row { display: flex; justify-content: flex-end; gap: 8px; margin-top: 24px; }
.btn {
  padding: 9px 20px; border-radius: var(--border-radius-md);
  font-size: 13px; font-weight: 500; border: none; transition: all 0.15s;
}
.btn-ghost { background: none; border: 0.5px solid var(--color-border-secondary); color: var(--color-text-secondary); }
.btn-ghost:hover { background: var(--color-background-secondary); }
.btn-primary { background: #1B3A6B; color: #fff; }
.btn-primary:hover { background: #2E5FA3; }
.btn-danger { background: var(--color-red); color: #fff; }
.btn-danger:hover { background: #A32D2D; }
.btn-danger:disabled, .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
.success-box { text-align: center; padding: 40px 24px; }
.success-box h2 { font-size: 18px; font-weight: 500; margin-bottom: 8px; }
.success-box p { font-size: 14px; color: var(--color-text-secondary); line-height: 1.6; }
.icon-circle {
  width: 56px; height: 56px; border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  margin: 0 auto 16px; font-size: 26px;
}
.icon-circle.red { background: var(--color-red-light); color: #A32D2D; }
.error-msg {
  color: var(--color-red); font-size: 13px; padding: 8px 12px;
  background: var(--color-red-light); border-radius: var(--border-radius-md); margin-top: 8px;
}
</style>
