import { onBeforeUnmount, ref } from "vue";

export function useMediaQuery(query: string) {
  const media = window.matchMedia(query);
  const matches = ref(media.matches);
  const update = (event: MediaQueryListEvent) => (matches.value = event.matches);
  media.addEventListener("change", update);
  onBeforeUnmount(() => media.removeEventListener("change", update));
  return matches;
}
