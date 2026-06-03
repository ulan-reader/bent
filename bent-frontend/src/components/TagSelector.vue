<template>
  <div class="tag-row">
    <button
      v-for="tag in tags"
      :key="tag"
      class="tag"
      :class="{ selected: modelValue === tag }"
      type="button"
      @click="$emit('update:modelValue', tag)"
    >
      {{ tag }}
    </button>
  </div>
</template>

<script setup>
defineProps({
  tags: Array,
  modelValue: String,
})
defineEmits(['update:modelValue'])
</script>

<style scoped>
.tag-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.tag {
  padding: 5px 12px;
  border-radius: 20px;
  font-size: 12px;
  cursor: pointer;
  border: 0.5px solid var(--color-border-tertiary);
  background: var(--color-background-secondary);
  color: var(--color-text-secondary);
  transition: all 0.15s;
  font-family: inherit;
}
.tag:hover {
  border-color: var(--color-border-secondary);
}
.tag.selected {
  background: var(--color-blue-light);
  border-color: #185FA5;
  color: #185FA5;
}
</style>
