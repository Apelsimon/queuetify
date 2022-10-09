<template>
  <div class="main">
    <h1>Welcome to Queuetify</h1>
    <button @click="create">Create</button>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import axios from 'axios'

export default defineComponent({
  name: 'Main',
  components: {
  },
  setup() {
    let create = async () => {
      try {
        let result = await axios.get("/create"); 
        window.location.href = result.data;
      } catch (error) {
        console.log("Error on create endpoint: ", error);
      }
    }

    return { create }
  },
});
</script>
