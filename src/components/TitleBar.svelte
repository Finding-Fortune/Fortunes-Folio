<script>
    import { onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { invoke } from '@tauri-apps/api/core';
    import { writable } from 'svelte/store';
    import { marked } from 'marked';
  
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
  
    // State for showing/hiding the cheatsheet
    let showCheatsheet = false;
    // Raw markdown content
    let cheatsheetMarkdown = '';
    // The rendered HTML
    let cheatsheetHtml = '';
  
    async function openCheatsheet() {
      try {
        const response = await fetch('/cheatsheet.md');
        cheatsheetMarkdown = await response.text();
        // Convert the raw markdown to HTML
        // @ts-ignore
        cheatsheetHtml = marked(cheatsheetMarkdown);
  
        showCheatsheet = true;
      } catch (error) {
        console.error('Failed to load cheatsheet:', error);
      }
    }
  
    function closeCheatsheet() {
      showCheatsheet = false;
    }
  
    onMount(() => {
      setupTitlebar();
      loadDarkMode();
    });
  </script>
  
  <!-- Title Bar -->
  <div
    data-tauri-drag-region
    class="titlebar fixed top-0 w-full left-0 flex items-center justify-between bg-gray-800 text-white px-4 h-8 z-50"
  >
    <!-- App Title + Cheatsheet Button -->
    <div class="flex items-center space-x-2">
      <span class="text-sm font-medium">Fortune's Folio</span>
      <!-- Cheatsheet Button -->
      <button
        class="text-sm font-medium px-2 py-1 bg-gray-700 rounded hover:bg-gray-600 focus:outline-none"
        on:click={openCheatsheet}
      >
        Cheatsheet
      </button>
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
  
  <!-- Cheatsheet Modal -->
  {#if showCheatsheet}
  <div
    class="fixed inset-0 bg-black bg-opacity-60 z-50 flex items-center justify-center"
    on:click={closeCheatsheet}
  >
    <!-- Modal Content -->
    <!-- stopPropagation so clicking inside doesn't close it -->
    <div
      on:click|stopPropagation
      class="relative bg-white dark:bg-gray-900 w-11/12 md:w-3/4 lg:w-1/2 h-5/6 p-4 
             rounded-lg shadow-lg overflow-y-auto"
    >
      <!-- Pinned Close Button -->
      <!-- 'sticky top-0' keeps the button at top if user scrolls. 'flex justify-end' so it's on the right. -->
      <div class="flex justify-end sticky top-0 z-10">
        <button 
          class="bg-red-500 text-white px-4 py-2 text-xl rounded-full hover:bg-red-400
                 focus:outline-none shadow-md"
          on:click={closeCheatsheet}
        >
          ✕
        </button>
      </div>

      <!-- Render Markdown -->
      <div class="prose dark:prose-invert max-w-none">
        {@html cheatsheetHtml}
      </div>
    </div>
  </div>
{/if}


  

<style>
    :global(code)::before,
  :global(code)::after {
    content: none !important;
  }
  
  :global(.prose code)::before,
  :global(.prose code)::after {
    content: none !important;
  }
</style>