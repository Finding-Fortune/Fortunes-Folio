<script lang="ts">
    import { onMount, tick } from "svelte";
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';
    import { writable } from 'svelte/store';

    interface Note {
        id: number;
        title: string;
        content: string;
        markdown: boolean;
        tags: string | string[]; // Allow tags to be a string or an array of strings
    }

    let tags: Set<string> = new Set();
    let notes: Note[] = [];
    let filteredNotes: Note[] = [];
    let selectedTag: string | null = null;

    // Fetch all notes from the backend
    async function fetchNotes() {
        try {
            const rawNotes = (await invoke("get_notes")) as Note[];
            notes = rawNotes.map((note) => ({
                ...note,
                tags: typeof note.tags === "string" ? note.tags.split(",").map(tag => tag.trim()) : note.tags,
            }));
            extractTags();
        } catch (error) {
            console.error("Failed to fetch notes:", error);
        }
    }

    // Extract unique tags from all notes
    function extractTags() {
        tags = new Set();
        notes.forEach((note) => {
            if (typeof note.tags === "string") {
                // Split the string and add each tag to the set
                note.tags.split(",").forEach((tag) => tags.add(tag.trim()));
            } else if (Array.isArray(note.tags)) {
                // Iterate over the array and add each tag to the set
                note.tags.forEach((tag) => tags.add(tag));
            }
        });
    }

    // Filter notes by the selected tag
    function filterNotesByTag(tag: string) {
        selectedTag = tag;
        filteredNotes = notes.filter((note) => note.tags.includes(tag));
    }

    // Handle note click to navigate back to the main page with the selected note
    function handleNoteClick(noteId: number) {
        goto(`/?noteId=${noteId}`);
    }

    function handleKeydown(event: KeyboardEvent): void {
        if(event.ctrlKey && event.key == "n") {
            event.preventDefault();
            goto('/');
        }
    }

    // Store for dark mode
    export const darkMode = writable(true);

    // Toggle dark mode
    async function loadDarkMode() {
        // Check localStorage for dark mode state immediately
        const localStorageMode = localStorage.getItem('darkMode') === 'true';
        darkMode.set(localStorageMode);

        // Apply the `dark` or `light` class immediately based on localStorage
        document.documentElement.classList.toggle('dark', localStorageMode);
        document.documentElement.classList.toggle('light', !localStorageMode);

        // Fetch the dark mode state from the backend asynchronously
        const enabled = await invoke<boolean>('get_dark_mode');
        darkMode.set(enabled);

        // Update localStorage and apply the class again if needed
        localStorage.setItem('darkMode', enabled ? 'true' : 'false');
        document.documentElement.classList.toggle('dark', enabled);
        document.documentElement.classList.toggle('light', !enabled);

        // Force a tick to ensure the DOM re-renders
        await tick();
    }
    loadDarkMode()

    onMount(() => {
        loadDarkMode()
        fetchNotes();
        window.addEventListener("keydown", handleKeydown);
        return () => {
            window.removeEventListener("keydown", handleKeydown);
        };
    });
</script>

<div style="max-width: 100vw;" class="flex-grow bg-gray-100 p-6">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-3xl font-bold text-gray-800">Tags</h1>
        <button
            class="px-4 py-2 bg-blue-500 text-white font-semibold rounded-lg hover:bg-blue-600"
            on:click={() => goto('/')}
        >
            Back to Notes
        </button>
    </div>

    <!-- Tags Section -->
    <div class="mb-4">
        <div
          class="flex flex-wrap gap-2 overflow-auto max-h-64 p-2 rounded-lg"
        >
          {#each Array.from(tags).filter(tag => tag.trim() !== "") as tag}
            <button
              class="px-3 py-1 bg-indigo-500 text-white rounded-full hover:bg-indigo-600 transition truncate max-w-[150px]"
              title={tag} 
              on:click={() => filterNotesByTag(tag)}
            >
              {tag}
            </button>
          {/each}
        </div>
      </div>
      

    <!-- Notes List -->
    {#if selectedTag}
        <div>
            <h2 class="flex space-around-2 text-xl font-bold text-gray-700 mb-4">
                <span style="min-width: fit-content;">Notes with tag:</span> <span
                style="display: inline-block; max-width: 100%; overflow: auto; white-space: nowrap;"
                class="ml-2 text-indigo-600 dark:text-indigo-300"
                title={selectedTag} 
              >
                {selectedTag}
              </span>
              
              
            </h2>
            <ul>
                {#each filteredNotes as note}
                    <li class="mb-4" style="background-color: rgb(229 231 235 / var(--tw-bg-opacity, 1)); max-height: 100px; overflow: auto;">
                        <button
                            type="button"
                            class="w-full p-4 rounded-lg shadow hover:shadow-lg transition cursor-pointer text-left"
                            on:click={() => handleNoteClick(note.id)}
                        >
                            <h3 class="text-lg font-semibold text-gray-800 mb-2">{note.title}</h3>
                            <p class="text-sm text-gray-600 truncate">{note.content}</p>
                        </button>
                    </li>                
                {/each}
            </ul>
        </div>
    {:else}
        <p class="text-gray-500">Select a tag to view associated notes.</p>
    {/if}
</div>


<style>
    :global(html.dark) {
        --bg-color: #1f2937; /* Dark background color */
        --text-color: #f3f4f6; /* Light text color */
        --border-color: #374151; /* Dark border color */
    }

    :global(html.dark body) {
        background-color: var(--bg-color) !important;
        color: var(--text-color) !important;
    }

    :global(html.dark .color-white-100) {
        color: #f3f4f6; /* Dark equivalent of gray-100 */
    }

    :global(html.dark .bg-gray-100) {
        background-color: #1f2937; /* Dark equivalent of gray-100 */
    }

    :global(html.dark .bg-gray-50) {
        background-color: #2d3748; /* Dark equivalent of gray-50 */
    }

    :global(html.dark .text-gray-900) {
        color: #f3f4f6; /* Light equivalent of gray-900 */
    }

    :global(html.dark .text-gray-800) {
        color: #e2e8f0; /* Light equivalent of gray-800 */
    }

    :global(html.dark .border) {
        border-color: var(--border-color);
    }

    :global(html.dark button) {
        background-color: #374151; /* Dark button background */
        color: var(--text-color);
    }

    :global(html.dark p, html.dark h2, html.dark h3, html.dark h4, html.dark h1, html.dark h5) {
        color: var(--text-color);
    }

    :global(html.dark button:hover) {
        background-color: #4b5563; /* Dark button hover */
    }

    :global(html.dark header) {
        background-color: #1f2937 !important; /* Equivalent to dark:bg-gray-800 */
        color: #f3f4f6 !important; /* Equivalent to dark:text-white */
    }
    :global(html.dark textarea, html.dark input, html.dark .tag, html.dark li) {
        background-color: #2d3748 !important; /* Equivalent to dark:bg-gray-800 */
        color: #f3f4f6 !important; /* Equivalent to dark:text-white */
    }
</style>