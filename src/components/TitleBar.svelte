<script>
    import { onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { invoke } from '@tauri-apps/api/core'
    import { writable, get } from 'svelte/store';

    const appWindow = getCurrentWindow();

    function setupTitlebar() {
        const minimizeButton = document.getElementById("titlebar-minimize");
        const maximizeButton = document.getElementById("titlebar-maximize");
        const closeButton = document.getElementById("titlebar-close");

        if (minimizeButton) {
            minimizeButton.addEventListener("click", () => {
                appWindow.minimize();
            });
        }

        if (maximizeButton) {
            maximizeButton.addEventListener("click", async () => {
                const isMaximized = await appWindow.isMaximized();
                if (isMaximized) {
                appWindow.unmaximize();
                } else {
                appWindow.maximize();
                }
            });
        }

        if (closeButton) {
            closeButton.addEventListener("click", () => {
                appWindow.close();
            });
        }
    }

    // Store for dark mode
    export const darkMode = writable(false);

    // Toggle dark mode
    async function loadDarkMode() {
        const enabled = await invoke('get_dark_mode');
        darkMode.set(enabled);
        if (enabled) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    }

    onMount(() => {
        setupTitlebar();
        // Check system preferences for dark mode
        loadDarkMode();
    });
</script>



<div
  data-tauri-drag-region
  class="titlebar fixed top-0 w-full left-0 flex items-center justify-between bg-gray-800 text-white px-4 h-8 z-50"
>
  <!-- App Title -->
  <div class="flex items-center space-x-2">
    <span class="text-sm font-medium">Fortune's Notes</span>
  </div>

  <!-- Window Buttons -->
  <div class="flex items-center space-x-2">
    <div
      id="titlebar-minimize"
      class="titlebar-button w-8 h-8 flex items-center justify-center rounded-md hover:bg-gray-700 cursor-pointer"
    >
      <img
        src="https://api.iconify.design/mdi:window-minimize.svg?color=%23ffffff"
        alt="minimize"
        class="w-4 h-4"
      />
    </div>
    <div
      id="titlebar-maximize"
      class="titlebar-button w-8 h-8 flex items-center justify-center rounded-md hover:bg-gray-700 cursor-pointer"
    >
      <img
        src="https://api.iconify.design/mdi:window-maximize.svg?color=%23ffffff"
        alt="maximize"
        class="w-4 h-4"
      />
    </div>
    <div
      id="titlebar-close"
      class="titlebar-button w-8 h-8 flex items-center justify-center rounded-md hover:bg-red-600 cursor-pointer"
    >
      <img
        src="https://api.iconify.design/mdi:close.svg?color=%23ffffff"
        alt="close"
        class="w-4 h-4"
      />
    </div>
  </div>
</div>
