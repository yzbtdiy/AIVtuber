<script setup lang="ts">


import LiveRoom from './views/LiveRoom.vue';

</script>

<template>
  <LiveRoom />
</template>

<style>
body{
  margin: 0;
}

#app {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  /* background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); */
  background-image: url("/background.jpg");
  background-size: cover;
}
</style>
