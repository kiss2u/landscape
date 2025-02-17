import { defineStore } from "pinia";
import { ref } from "vue";

export const useUiStore = defineStore("ui", () => {
  const username = ref<string>("");

  async function INSERT_USERNAME(name: string) {
    username.value = name;
  }

  return {
    username,
    INSERT_USERNAME,
  };
});
