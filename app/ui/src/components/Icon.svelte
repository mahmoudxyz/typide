<script lang="ts">
  import { icons } from "../lib/icons";
  let { name, size = 16, stroke = 1.6, fill = "none" }: {
    name: string;
    size?: number;
    stroke?: number;
    fill?: string;
  } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={fill}
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {@html icons[name] ?? icons.dot}
</svg>
