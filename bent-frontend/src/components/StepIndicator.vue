<template>
  <div class="step-row">
    <template v-for="(step, i) in steps" :key="i">
      <div
        class="step-dot"
        :class="{
          done: i < current,
          active: i === current,
          idle: i > current,
        }"
      >
        <span v-if="i < current">✓</span>
        <span v-else>{{ i + 1 }}</span>
      </div>
      <div v-if="i < steps.length - 1" class="step-line" />
    </template>
  </div>
</template>

<script setup>
defineProps({
  steps: { type: Array, default: () => [1, 2, 3] },
  current: { type: Number, default: 0 },
})
</script>

<style scoped>
.step-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 24px;
}
.step-dot {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 500;
  flex-shrink: 0;
  transition: all 0.2s;
}
.step-dot.done { background: var(--color-green-light); color: #3B6D11; }
.step-dot.active { background: #1B3A6B; color: #fff; }
.step-dot.idle { background: var(--color-background-secondary); color: var(--color-text-tertiary); }
.step-line {
  flex: 1;
  height: 0.5px;
  background: var(--color-border-tertiary);
}
</style>
