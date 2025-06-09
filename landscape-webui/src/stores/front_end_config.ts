import { defineStore } from "pinia";
import { ref } from "vue";

export const useFrontEndStore = defineStore(
  "front_end",
  () => {
    const presentation_mode = ref(false);

    return {
      presentation_mode,
    };
  },
  {
    persist: {
      key: "front_end",
      storage: localStorage,
    },
  }
);
