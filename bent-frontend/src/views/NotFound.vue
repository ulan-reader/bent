<template>
  <div>
    <Topbar />
    <div class="main">
      <div class="card box">
        <div class="icon">🔍</div>
        <h2>Страница не найдена</h2>
        <p>Возможно, ссылка устарела или введена неверно.</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import Topbar from '../components/Topbar.vue'
</script>

<style scoped>
.main { max-width: 480px; margin: 0 auto; padding: 32px 16px; }
.card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg); padding: 24px;
}
.box { text-align: center; padding: 48px 24px; }
.icon { font-size: 40px; margin-bottom: 16px; }
h2 { font-size: 18px; font-weight: 500; margin-bottom: 8px; }
p { font-size: 14px; color: var(--color-text-secondary); }
</style>
