import { mask_string } from "@/lib/common";
import { defineStore } from "pinia";
import { ref } from "vue";

export const useFrontEndStore = defineStore(
  "front_end",
  () => {
    const presentation_mode = ref(false);
    const username = ref<string>("");

    async function INSERT_USERNAME(name: string) {
      username.value = name;
    }

    function MASK_INFO(
      value: string | undefined | null,
    ): string | undefined | null {
      if (value) {
        return presentation_mode.value ? mask_string(value) : value;
      } else {
        return value;
      }
    }

    function MASK_PORT(
      value: string | number | undefined | null,
    ): string | number | undefined | null {
      if (value) {
        return presentation_mode.value ? "****" : value;
      } else {
        return value;
      }
    }

    return {
      presentation_mode,
      username,
      INSERT_USERNAME,
      MASK_INFO,
      MASK_PORT,
    };
  },
  {
    persist: {
      key: "front_end",
      storage: localStorage,
    },
  },
);
