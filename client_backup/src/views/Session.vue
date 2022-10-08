<template>
  <div class="session">
    <h1>Welcome to Session!</h1>
    <p>TODO: render session page according to {{ context }}</p>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
  name: 'Session',
  props: {
    context: String
  },
  components: {
  },
});
</script>
