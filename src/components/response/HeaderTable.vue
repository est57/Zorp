<script setup lang="ts">
defineProps<{ rows: string[][]; columns: string[]; empty: string }>();
</script>

<template>
  <p v-if="rows.length === 0" class="p-3 text-xs text-muted">{{ empty }}</p>
  <div v-else class="overflow-x-auto">
    <table class="w-full border-collapse text-xs select-text">
      <thead>
        <tr>
          <th
            v-for="column in columns"
            :key="column"
            class="label-caps h-7 bg-surface-2 px-3 text-left whitespace-nowrap"
          >
            {{ column }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, index) in rows" :key="index" class="border-b border-line">
          <td
            v-for="(cell, cellIndex) in row"
            :key="cellIndex"
            class="px-3 py-1.5 align-top font-mono break-all"
            :class="cellIndex === 0 ? 'w-1/3 text-muted' : 'text-fg'"
          >
            {{ cell }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
