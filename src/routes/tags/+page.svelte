<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';

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

    onMount(() => {
        fetchNotes();
        window.addEventListener("keydown", handleKeydown);
        return () => {
            window.removeEventListener("keydown", handleKeydown);
        };
    });
</script>

<div class="flex-grow bg-gray-100 p-6">
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
    <div class="mb-8">
        <!-- <h2 class="text-xl font-bold text-gray-700 mb-4">All Tags</h2> -->
        <div class="flex flex-wrap gap-2">
            {#each Array.from(tags).filter(tag => tag.trim() !== "") as tag}
                <button
                    class="px-3 py-1 bg-indigo-500 text-white rounded-full hover:bg-indigo-600 transition"
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
            <h2 class="text-xl font-bold text-gray-700 mb-4">
                Notes with tag: <span class="text-indigo-600 dark:text-indigo-300">{selectedTag}</span>
            </h2>
            <ul>
                {#each filteredNotes as note}
                    <li
                        class="mb-4 p-4 bg-white rounded-lg shadow hover:shadow-lg transition cursor-pointer"
                        on:click={() => handleNoteClick(note.id)}
                    >
                        <h3 class="text-lg font-semibold text-gray-800 mb-2">{note.title}</h3>
                        <p class="text-sm text-gray-600 truncate">{note.content}</p>
                    </li>
                {/each}
            </ul>
        </div>
    {:else}
        <p class="text-gray-500">Select a tag to view associated notes.</p>
    {/if}
</div>


<style>
    :global(html.dark .color-white-100) {
        color: #f3f4f6; /* Dark equivalent of gray-100 */
    }
</style>