<template>
  <header class="topbar">
    <div class="logo">
      <div class="logo-mark">БТ</div>
      Цифровой контроль БЕНТ
    </div>
    <slot />
  </header>
</template>

<script setup></script>

<style scoped>
.topbar {
  background: var(--color-background-primary);
  border-bottom: 0.5px solid var(--color-border-tertiary);
  padding: 0 32px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 100;
}
.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  font-weight: 500;
  color: var(--color-text-primary);
}
.logo-mark {
  width: 28px;
  height: 28px;
  background: #1B3A6B;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
}
</style>
