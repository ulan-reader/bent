<template>
  <div>
    <Topbar />
    <div class="main">
      <div class="card">
        <div class="box">
          <div class="icon-circle">⏰</div>
          <h2>Ссылка устарела</h2>
          <p>Одноразовые ссылки действуют 24 часа.<br>Попросите новую ссылку у бота в Telegram.</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import Topbar from '../components/Topbar.vue'
</script>

<style scoped>
.main { max-width: 680px; margin: 0 auto; padding: 32px 16px; }
.card {
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-tertiary);
  border-radius: var(--border-radius-lg);
  padding: 24px;
}
.box { text-align: center; padding: 40px 24px; }
.icon-circle {
  width: 56px; height: 56px; border-radius: 50%;
  background: #FEF3E0; color: #854F0B;
  display: flex; align-items: center; justify-content: center;
  margin: 0 auto 16px; font-size: 26px;
}
h2 { font-size: 18px; font-weight: 500; margin-bottom: 8px; }
p { font-size: 14px; color: var(--color-text-secondary); line-height: 1.6; }
</style>
