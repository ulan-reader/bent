<template>
  <div>
    <div class="file-zone" @click="inputRef.click()">
      <span class="icon">{{ icon }}</span>
      <p>{{ label }}</p>
      <small>{{ hint }}</small>
    </div>
    <input
      ref="inputRef"
      type="file"
      :accept="accept"
      style="display: none"
      @change="handleChange"
    />
    <p v-if="fileName" class="file-name">📎 {{ fileName }}</p>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const props = defineProps({
  accept: { type: String, default: 'image/*,.pdf' },
  label: { type: String, default: 'Нажмите для загрузки файла' },
  hint: { type: String, default: 'JPG, PNG, PDF — до 20 МБ' },
  icon: { type: String, default: '📁' },
})

const emit = defineEmits(['update:file'])
const inputRef = ref(null)
const fileName = ref('')

function handleChange(e) {
  const file = e.target.files[0]
  if (file) {
    fileName.value = file.name
    emit('update:file', file)
  }
}
</script>

<style scoped>
.file-zone {
  border: 0.5px dashed var(--color-border-secondary);
  border-radius: var(--border-radius-md);
  padding: 24px;
  text-align: center;
  cursor: pointer;
  transition: all 0.15s;
}
.file-zone:hover {
  border-color: #378ADD;
  background: var(--color-background-secondary);
}
.icon {
  font-size: 24px;
  display: block;
  margin-bottom: 8px;
}
.file-zone p {
  font-size: 13px;
  color: var(--color-text-secondary);
}
.file-zone small {
  font-size: 11px;
  color: var(--color-text-tertiary);
}
.file-name {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 6px;
}
</style>
