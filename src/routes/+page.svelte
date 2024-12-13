<script lang="ts">
    import { goto } from '$app/navigation';
    import { marked } from 'marked';
    import { onMount } from "svelte";
    import { invoke } from '@tauri-apps/api/core'

    marked.setOptions({
        breaks: true, // Enable line breaks
        gfm: true,    // Enable GitHub Flavored Markdown
    });

    function navigateBack() {
        goto('/hello');
    }

    interface Note {
        id: number;
        title: string;
        content: string;
        markdown: boolean; // New flag to toggle between Markdown and plain text
        tags: string[];
    }

    let notes: Note[] = [];
    let selectedNote: Note | null = null;
    let tagSearch = "";
    let newTag = "";

    function addTag(): void {
        if (selectedNote) {
            if (newTag.trim() && !selectedNote.tags.includes(newTag.trim())) {
                selectedNote.tags = [...selectedNote.tags, newTag.trim()];
                newTag = "";
            }
        }
    }

    function removeTag(tag: string): void {
        if (selectedNote) {
            selectedNote.tags = selectedNote.tags.filter((t) => t !== tag);
        }
    }

    function filteredNotes(): Note[] {
        if (!tagSearch.trim()) {
            return notes;
        }
        return notes.filter((note) =>
            note.tags.some((tag) =>
                tag.toLowerCase().includes(tagSearch.trim().toLowerCase())
            )
        );
    }

    function selectNote(note: Note): void {
        selectedNote = note;
    }

    async function addNewNote() {
        const newNote = {
            title: `New Note ${notes.length + 1}`,
            content: "",
            markdown: false,
            tags: [],
        };
        try {
            await invoke("add_note", newNote);
            await fetchNotes(); // Refresh notes list
        } catch (error) {
            console.error("Failed to add note:", error);
        }
    }

    async function saveChanges() {
        if (selectedNote) {
            try {
                await invoke("update_note", {
                    id: selectedNote.id,
                    title: selectedNote.title,
                    content: selectedNote.content,
                    markdown: selectedNote.markdown,
                    tags: Array.isArray(selectedNote.tags) ? selectedNote.tags : [],
                });
                await fetchNotes(); // Refresh the notes list
            } catch (error) {
                console.error("Failed to save changes:", error);
            }
        }
    }

    async function deleteNote() {
        if (selectedNote) {
            try {
                await invoke("delete_note", { id: selectedNote.id });
                await fetchNotes(); // Refresh notes list
            } catch (error) {
                console.error("Failed to delete note:", error);
            }
        }
    }

    function toggleMarkdown(): void {
        if (selectedNote) {
            selectedNote.markdown = !selectedNote.markdown;
        }
    }

    // Add the keybind listener
    function handleKeydown(event: KeyboardEvent): void {
    if (event.ctrlKey && event.key === "e") {
        event.preventDefault();
        if (selectedNote) toggleMarkdown();
    }
    if (event.ctrlKey && event.key === "s") {
        event.preventDefault();
        saveChanges();
    }
    if (event.ctrlKey && event.key === "d") {
        event.preventDefault();
        deleteNote();
    }
    if (event.ctrlKey && event.key === "n") {
        event.preventDefault();
        addNewNote();
    }
}

  async function searchNotesByTag() {
    try {
        const filtered = (await invoke("search_notes_by_tag", { tag: tagSearch })) as Note[];
        notes = filtered;
        if (notes.length > 0) {
            selectedNote = notes[0]; // Select the first note in the filtered list
        } else {
            selectedNote = null;
        }
    } catch (error) {
        console.error("Failed to search notes by tag:", error);
    }
}

  // Fetch notes from the backend
    async function fetchNotes() {
        try {
            const currentNoteId = selectedNote?.id; // Save the current note's ID
            notes = (await invoke("get_notes")) as Note[];
            // Restore the selected note
            selectedNote = notes.find((note) => note.id === currentNoteId) || notes[0];
        } catch (error) {
            console.error("Failed to fetch notes:", error);
        }
    }

  // Lifecycle to attach and detach the listener
  onMount(() => {
    fetchNotes();
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<div class="flex h-screen bg-gray-100">
    <!-- Sidebar -->
    <aside class="w-1/4 bg-gray-800 text-white p-4 flex flex-col">
    <h2 class="text-2xl font-bold mb-6">Notes</h2>

    <!-- Search by Tag -->
    <div class="mb-4">
        <input
            type="text"
            class="w-full p-2 rounded-lg text-gray-900"
            bind:value={tagSearch}
            placeholder="Search by tag..."
            on:input={searchNotesByTag}
        />
    </div>

    <!-- Notes List -->

        <ul class="flex-1 overflow-y-auto">
            {#if notes.length > 0}
                {#each notes as note (note.id)}
                    <li>
                        <button
                            class="w-full text-left p-3 rounded-lg mb-2 cursor-pointer bg-gray-700 hover:bg-gray-600"
                            on:click={() => (selectedNote = note)}
                        >
                            <h3 class="text-lg font-semibold">{note.title}</h3>
                            <p class="text-sm text-gray-300 truncate">{note.content}</p>
                        </button>
                    </li>
                {/each}
            {:else}
                <p class="text-gray-400">No notes found.</p>
            {/if}
        </ul>
      <button class="mt-4 p-2 bg-blue-500 text-white font-semibold rounded-lg hover:bg-blue-600"
      on:click={addNewNote}>
        + New Note
      </button>
      <button class="mt-4 p-2 bg-blue-500 text-white font-semibold rounded-lg hover:bg-blue-600">
            <a href="/">Go back to Home</a>
        </button>
    </aside>
  
    <!-- Main Content -->
    <main class="flex-1 flex flex-col">
      <!-- Header -->
      <header class="bg-white shadow-md p-4 flex items-center justify-between">
        <div class="flex-1">
          {#if selectedNote}
            {#if selectedNote.markdown}
                <h1 class="text-xl font-bold truncate">{selectedNote.title}</h1>
            {:else}
                <input
                type="text"
                class="text-xl font-bold border-b-2 focus:outline-none focus:border-blue-500 p-1 w-full max-w-lg truncate"
                bind:value={selectedNote.title}
                placeholder="Enter title here"
                />
            {/if}
          {/if}
        </div>
        {#if selectedNote}
            <div class="flex space-x-4">
            <button
                class="p-2 bg-yellow-500 text-white rounded-lg hover:bg-yellow-600"
                on:click={toggleMarkdown}
            >
                {selectedNote.markdown ? "Edit as Text" : "Preview Markdown"}
            </button>
            <button
                class="p-2 bg-green-500 text-white rounded-lg hover:bg-green-600"
                on:click={saveChanges}
            >
                Save Changes
            </button>
            <button
                class="p-2 bg-red-500 text-white rounded-lg hover:bg-red-600"
                on:click={deleteNote}
            >
                Delete Note
            </button>
            </div>
        {/if}
      </header>        
  
      <!-- Content -->
      <div class="p-4 flex-1 bg-gray-50 overflow-y-auto">
        {#if selectedNote}
            {#if selectedNote.markdown}
            <div class="prose max-w-none">
                {@html marked(selectedNote.content)}
            </div>
            {:else}
            <textarea
                class="w-full h-full border rounded-lg p-4 focus:outline-none focus:ring-2 focus:ring-blue-400 resize-none"
                bind:value={selectedNote.content}
                placeholder="Start writing your note here..."
            ></textarea>
            {/if}
        {/if}

        <!-- Tags Section -->
        {#if selectedNote}
            <div class="mt-4">
                <h2 class="text-lg font-bold">Tags</h2>
                <div class="flex items-center space-x-2 mb-4">
                    {#each selectedNote.tags as tag}
                        <div class="flex items-center bg-gray-200 text-gray-800 px-2 py-1 rounded">
                            {tag}
                            <button
                                class="ml-2 text-red-500 hover:text-red-700"
                                on:click={() => removeTag(tag)}
                            >
                                &times;
                            </button>
                        </div>
                    {/each}
                </div>
                <div class="flex space-x-2">
                    <input
                        type="text"
                        class="border rounded-lg p-2 w-full focus:outline-none focus:ring-2 focus:ring-blue-400"
                        bind:value={newTag}
                        placeholder="Add a tag..."
                    />
                    <button
                        class="p-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
                        on:click={addTag}
                    >
                        Add Tag
                    </button>
                </div>
            </div>
        {/if}
      </div>
    </main>
  </div>

  
  
<style>
.tag {
    display: inline-flex;
    align-items: center;
    background-color: #f3f4f6;
    color: #1f2937;
    padding: 0.25rem 0.5rem;
    border-radius: 0.375rem;
    margin-right: 0.5rem;
}
.tag button {
    margin-left: 0.5rem;
    background: none;
    border: none;
    color: #ef4444;
    cursor: pointer;
}
.tag button:hover {
    color: #b91c1c;
}
</style>

