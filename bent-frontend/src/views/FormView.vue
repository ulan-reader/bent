<template>
  <div>
    <Topbar />

    <!-- Token loading/error state -->
    <div class="main" v-if="tokenState !== 'valid'">
      <div class="card center-box">
        <div v-if="tokenState === 'loading'" class="loading-text">Проверяем ссылку…</div>
        <div v-else-if="tokenState === 'missing'" class="error-box">
          <div class="icon-circle red">⚠</div>
          <h2>Ссылка не указана</h2>
          <p>Перейдите по ссылке из Telegram-бота.</p>
        </div>
      </div>
    </div>

    <!-- Main form -->
    <div class="main" v-else>
      <div class="section-label">Обращение сотрудника</div>

      <!-- Step 1 -->
      <div v-if="step === 0">
        <div class="card">
          <StepIndicator :current="0" />

          <div class="field">
            <label>Подразделение</label>
            <select v-model="form.department">
              <option value="">Выберите подразделение…</option>
              <optgroup label="БЕНТ">
                <option>АБК</option><option>БСЦ</option><option>Отдел сбыта</option>
                <option>РМЦ (БЕНТ)</option><option>СБ</option><option>ШЦ</option><option>ЭРУ</option>
              </optgroup>
              <optgroup label="Автоклав">
                <option>Основное подразделение</option><option>Цех по газоблоку</option>
              </optgroup>
              <optgroup label="Fasteners Metel">
                <option>Основное подразделение</option><option>РМЦ (Fasteners)</option>
              </optgroup>
              <option>Standlab</option>
            </select>
          </div>

          <div class="field">
            <label>Категория обращения</label>
            <TagSelector
              v-model="form.category"
              :tags="['Условия труда', 'Бытовые вопросы', 'Предложение по улучшению', 'Жалоба на руководство', 'Конфликт в коллективе', 'Другое']"
            />
          </div>

          <div v-if="error" class="error-msg">{{ error }}</div>

          <div class="btn-row">
            <button class="btn btn-primary" @click="nextStep">Далее →</button>
          </div>
        </div>

        <div class="anon-note">
          <span>🔒</span>
          <p>Ваше обращение полностью анонимно. Имя и контактные данные не передаются и не сохраняются.</p>
        </div>
      </div>

      <!-- Step 2 -->
      <div v-else-if="step === 1">
        <div class="card">
          <StepIndicator :current="1" />

          <div class="field">
            <label>Суть обращения</label>
            <textarea v-model="form.text" placeholder="Опишите проблему или предложение подробно…" rows="5" />
          </div>

          <div class="field">
            <label>Прикрепить файл <span class="opt">необязательно</span></label>
            <FileZone v-model:file="form.file" icon="📷" label="Нажмите для загрузки фото" hint="JPG, PNG, PDF — до 20 МБ" />
          </div>

          <div v-if="error" class="error-msg">{{ error }}</div>

          <div class="btn-row">
            <button class="btn btn-ghost" @click="step--">Назад</button>
            <button class="btn btn-primary" @click="nextStep">Далее →</button>
          </div>
        </div>
      </div>

      <!-- Step 3: Confirm -->
      <div v-else-if="step === 2">
        <div class="card">
          <StepIndicator :current="2" />

          <div class="field">
            <label>Проверьте обращение</label>
            <div class="confirm-box">
              <div><strong>Подразделение:</strong> {{ form.department }}</div>
              <div><strong>Категория:</strong> {{ form.category }}</div>
              <div><strong>Текст:</strong> {{ form.text.length > 100 ? form.text.slice(0, 100) + '…' : form.text }}</div>
              <div v-if="form.file"><strong>Файл:</strong> {{ form.file.name }}</div>
            </div>
          </div>

          <div v-if="error" class="error-msg">{{ error }}</div>

          <div class="btn-row">
            <button class="btn btn-ghost" @click="step--" :disabled="submitting">Назад</button>
            <button class="btn btn-primary" @click="submit" :disabled="submitting">
              {{ submitting ? 'Отправка…' : '✉ Отправить' }}
            </button>
          </div>
        </div>
      </div>

      <!-- Success -->
      <div v-else-if="step === 3">
        <div class="card">
          <div class="success-box">
            <div class="icon-circle green">✓</div>
            <h2>Обращение отправлено</h2>
            <p>Ваше сообщение передано руководству анонимно.<br>Обычно реакция поступает в течение 48 часов.</p>
            <button class="btn btn-ghost" style="margin-top: 20px" @click="reset">Новое обращение</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api } from '../api/index.js'
import Topbar from '../components/Topbar.vue'
import StepIndicator from '../components/StepIndicator.vue'
import TagSelector from '../components/TagSelector.vue'
import FileZone from '../components/FileZone.vue'

const route = useRoute()
const router = useRouter()

const tokenState = ref('loading') // loading | valid | missing
const token = ref('')
const step = ref(0)
const error = ref('')
const submitting = ref(false)

const form = ref({
  department: '',
  category: '',
  text: '',
  file: null,
})

onMounted(async () => {
  const t = route.query.token
  if (!t) {
    tokenState.value = 'missing'
    return
  }
  token.value = t
  try {
    await api.validateToken(t)
    tokenState.value = 'valid'
  } catch (e) {
    if (e.status === 400) {
      const msg = e.message || ''
      if (msg.includes('used') || msg.includes('already')) router.replace('/form/used')
      else router.replace('/form/expired')
    } else {
      router.replace('/form/expired')
    }
  }
})

function nextStep() {
  error.value = ''
  if (step.value === 0) {
    if (!form.value.department) { error.value = 'Выберите подразделение'; return }
    if (!form.value.category) { error.value = 'Выберите категорию обращения'; return }
  }
  if (step.value === 1) {
    if (form.value.text.trim().length < 10) { error.value = 'Опишите проблему подробнее (минимум 10 символов)'; return }
  }
  step.value++
}

async function submit() {
  error.value = ''
  submitting.value = true
  try {
    let file_url = null
    if (form.value.file) {
      file_url = await api.uploadFile(form.value.file)
    }
    await api.submitEmployee({
      type: 'employee',
      token: token.value,
      department: form.value.department,
      category: form.value.category,
      text: form.value.text,
      file_url,
      channel: 'web',
    })
    step.value = 3
  } catch (e) {
    error.value = e.message || 'Ошибка при отправке. Попробуйте ещё раз.'
  } finally {
    submitting.value = false
  }
}

function reset() {
  form.value = { department: '', category: '', text: '', file: null }
  step.value = 0
  error.value = ''
}
</script>

<style scoped>
.main {
  max-width: 680px;
  margin: 0 auto;
  padding: 32px 16px;
}
.section-label {
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.08em;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  margin-bottom: 16px;
}
.card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 24px;
}
.field { margin-bottom: 20px; }
.field:last-child { margin-bottom: 0; }
label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 6px;
}
.opt { font-weight: 400; color: var(--color-text-tertiary); margin-left: 4px; }
select, textarea {
  width: 100%;
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  padding: 8px 12px;
  outline: none;
  transition: border-color 0.15s;
}
select {
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 32px;
}
select:focus, textarea:focus { border-color: #378ADD; box-shadow: 0 0 0 3px rgba(55,138,221,.12); }
textarea { resize: vertical; line-height: 1.6; }
.btn-row {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
}
.btn {
  padding: 9px 20px;
  border-radius: var(--border-radius-md);
  font-size: 13px;
  font-weight: 500;
  border: none;
  transition: all 0.15s;
}
.btn-ghost {
  background: none;
  border: 0.5px solid var(--color-border-secondary);
  color: var(--color-text-secondary);
}
.btn-ghost:hover { background: var(--color-background-secondary); }
.btn-primary { background: #1B3A6B; color: #fff; }
.btn-primary:hover { background: #2E5FA3; }
.btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
.anon-note {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
  padding: 12px 14px;
  margin-top: 16px;
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}
.confirm-box {
  background: var(--color-background-secondary);
  border-radius: var(--border-radius-md);
  padding: 16px;
  font-size: 13px;
  line-height: 1.8;
  color: var(--color-text-secondary);
}
.confirm-box strong { color: var(--color-text-primary); }
.success-box {
  text-align: center;
  padding: 40px 24px;
}
.success-box h2 { font-size: 18px; font-weight: 500; margin-bottom: 8px; }
.success-box p { font-size: 14px; color: var(--color-text-secondary); line-height: 1.6; }
.icon-circle {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 16px;
  font-size: 26px;
}
.icon-circle.green { background: var(--color-green-light); color: #3B6D11; }
.icon-circle.red { background: var(--color-red-light); color: #A32D2D; }
.error-msg {
  color: var(--color-red);
  font-size: 13px;
  margin-top: 8px;
  padding: 8px 12px;
  background: var(--color-red-light);
  border-radius: var(--border-radius-md);
}
.center-box { text-align: center; padding: 40px; }
.loading-text { color: var(--color-text-secondary); font-size: 14px; }
.error-box h2 { margin: 12px 0 8px; }
.error-box p { color: var(--color-text-secondary); font-size: 14px; }
</style>
