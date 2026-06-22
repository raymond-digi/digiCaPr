<template>
  <v-app>
    <AppBar />
    <Sidebar />

    <v-main>
      <v-container fluid class="pa-4">
        <slot />
      </v-container>
    </v-main>

    <Notification />
  </v-app>
</template>

<script setup lang="ts">
import AppBar from './AppBar.vue'
import Sidebar from './Sidebar.vue'
import Notification from './Notification.vue'
</script>
