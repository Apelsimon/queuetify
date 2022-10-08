<template>
  <component :is='useComponent' :context="context" />
</template>

<script lang="ts">
import { computed, defineComponent, ref } from 'vue';
import Main from './Main.vue'
import Session from './Session.vue'
import axios from 'axios'

export default defineComponent({
  name: 'Home',
  components: {
    Main,
    Session
  },
  setup() {
    const context = ref('none');
    const useComponent = computed(() => {
      return context.value != 'none' ? 'Session' : 'Main';
    });

    let get_context = async () => {
      try {
        const result = await axios.get("/context"); 
        console.log("got context: ", result.data);
        context.value = result.data;
      } catch (error) {
        console.log("Error on verify endpoint: ", error);
      }
    }

    return { context, get_context, useComponent }
  },
  mounted() {
    this.get_context();
  }
});
</script>
