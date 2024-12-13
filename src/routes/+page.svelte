<script lang="ts">
    import { goto } from '$app/navigation';
    import { marked } from 'marked';
    import { onMount, tick } from "svelte";
    import { invoke } from '@tauri-apps/api/core'
    import { writable, get } from 'svelte/store';
    import { getCurrentWindow } from "@tauri-apps/api/window";

    const appWindow = getCurrentWindow();

    marked.setOptions({
        breaks: true, // Enable line breaks
        gfm: true,    // Enable GitHub Flavored Markdown
    });

    let isResizing = false;
    let startX = 0;
    let startWidth = 0;

    function startResizing(event: MouseEvent): void {
        isResizing = true;
        startX = event.clientX;

        const sidebar = document.querySelector("aside") as HTMLElement | null;
        if (sidebar) {
            startWidth = sidebar.offsetWidth;

            // Add listeners for continuous resizing
            document.addEventListener("mousemove", resize);
            document.addEventListener("mouseup", stopResizing);
        }
    }

    function resize(event: MouseEvent): void {
        if (isResizing) {
            const dx = event.clientX - startX;

            const sidebar = document.querySelector("aside") as HTMLElement | null;
            if (sidebar) {
                const newWidth = Math.min(
                Math.max(startWidth + dx, 200), // Minimum width
                window.innerWidth * 0.5 // Maximum width (50% of viewport)
                );

                document.documentElement.style.setProperty("--sidebar-width", `${newWidth}px`);
            }
        }
    }

    function stopResizing(): void {
        isResizing = false;

        // Remove listeners when resizing stops
        document.removeEventListener("mousemove", resize);
        document.removeEventListener("mouseup", stopResizing);
    }


    interface Note {
        id: number;
        title: string;
        content: string;
        markdown: boolean; // New flag to toggle between Markdown and plain text
        tags: string | string[];
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
        if (selectedNote && Array.isArray(selectedNote.tags)) {
            selectedNote.tags = selectedNote.tags.filter((t) => t.trim() !== tag.trim());
        }
    }

    function filteredNotes(): Note[] {
        if (!tagSearch.trim()) {
            return notes;
        }
        return notes.filter((note) => {
            const tags = Array.isArray(note.tags)
                ? note.tags // Use tags array directly
                : note.tags.split(",").map((tag) => tag.trim()); // Convert string to array
            return tags.some((tag) =>
                tag.toLowerCase().includes(tagSearch.trim().toLowerCase())
            );
        });
    }


    function selectNote(note: Note): void {
        selectedNote = {
            ...note,
            tags: typeof note.tags === "string"
                ? note.tags.split(",").map((tag) => tag.trim())
                : note.tags,
        };
    }

    async function addNewNote() {
        const newNote = {
            title: `New Note ${notes.length + 1}`,
            content: "",
            markdown: false,
            tags: [],
        };

        try {
            // Add the new note to the backend
            await invoke("add_note", newNote);

            // Fetch the updated notes list
            await fetchNotes();

            // Identify the newly added note (assuming it's the last one added)
            selectedNote = notes[notes.length - 1];
            selectedNote.markdown = false;
        } catch (error) {
            console.error("Failed to add note:", error);
        }
    }


    async function saveChanges() {
        if (selectedNote) {
            try {
                // Save the selected note's ID
                const currentNoteId = selectedNote.id;

                const validTags = Array.isArray(selectedNote.tags)
                    ? selectedNote.tags.filter((tag) => tag.trim() !== "") // Filter out empty tags
                    : selectedNote.tags
                    ? selectedNote.tags.split(",").map((tag) => tag.trim()).filter((tag) => tag !== "")
                    : [];
                    console.log("Saving tags:", validTags);

                await invoke("update_note", {
                    id: selectedNote.id,
                    title: selectedNote.title,
                    content: selectedNote.content,
                    markdown: selectedNote.markdown,
                    tags: validTags,
                });

                selectedNote.markdown = true; // Switch to markdown mode after saving
                await fetchNotes(); // Refresh the notes list

                // Re-select the current note based on its ID
                const updatedNote = notes.find((note) => note.id === currentNoteId);
                if (updatedNote) {
                    selectNote(updatedNote);
                }
            } catch (error) {
                console.error("Failed to save changes:", error);
            }
        }
    }

    let lastDeletedNote: Note | null = null;
    async function deleteNote() {
        if (!selectedNote) return;

        try {
            // Save the current note as the last deleted note
            lastDeletedNote = { ...selectedNote };

            // Send delete request to the backend
            await invoke("delete_note", { id: selectedNote.id });

            // Refresh the notes list
            await fetchNotes();

            // Clear the selected note if no notes remain
            if (notes.length === 0) {
                selectedNote = null;
            } else {
                // Select another note (e.g., the first one)
                selectedNote = notes[0];
            }
        } catch (error) {
            console.error("Failed to delete note:", error);
        }
    }

    async function undoDeleteNote() {
        if (!lastDeletedNote) return;

        try {
            // Add the last deleted note back to the backend
            await invoke("add_note", {
                title: lastDeletedNote.title,
                content: lastDeletedNote.content,
                markdown: lastDeletedNote.markdown,
                tags: lastDeletedNote.tags,
            });

            // Fetch the updated notes list
            await fetchNotes();

            // Select the restored note
            selectedNote = notes.find(
                note => note.title === lastDeletedNote!.title && note.content === lastDeletedNote!.content
            ) || null;

            // Clear the deleted note reference after restoration
            lastDeletedNote = null;
        } catch (error) {
            console.error("Failed to undo delete note:", error);
        }
    }

    let textareaElement: HTMLTextAreaElement | null = null;

    function autoResizeTextarea() {
        if (textareaElement) {
            // Save the current scroll position of the window
            const windowScrollY = window.scrollY;

            // Reset the textarea height and adjust it to fit content
            textareaElement.style.height = 'auto'; // Reset height
            textareaElement.style.height = `${textareaElement.scrollHeight + 5}px`; // Set height to fit content. Add +5 to ensure no overflow

            // Get the caret (cursor) position
            const selectionStart = textareaElement.selectionStart;

            // Create a temporary div to measure caret position
            const tempDiv = document.createElement('div');
            const textBeforeCaret = textareaElement.value.substring(0, selectionStart);
            const lineHeight = parseInt(window.getComputedStyle(textareaElement).lineHeight, 10) || 16;

            // Mirror textarea styles onto the temporary div
            const style = window.getComputedStyle(textareaElement);
            tempDiv.style.cssText = `
                position: absolute;
                visibility: hidden;
                white-space: pre-wrap;
                word-wrap: break-word;
                width: ${textareaElement.offsetWidth}px;
                font-size: ${style.fontSize};
                font-family: ${style.fontFamily};
                line-height: ${style.lineHeight};
                padding: ${style.paddingTop} ${style.paddingRight} ${style.paddingBottom} ${style.paddingLeft};
            `;
            tempDiv.textContent = textBeforeCaret.replace(/\n$/, '\n\n'); // Add newline for accurate line spacing

            // Append the temp div to the body
            document.body.appendChild(tempDiv);

            // Get the position of the caret in the textarea
            const caretRect = tempDiv.getBoundingClientRect();
            const caretTop = caretRect.top;

            // Clean up the temporary div
            document.body.removeChild(tempDiv);

            // Calculate the center position of the viewport
            const viewportCenter = window.innerHeight / 2;

            // Scroll the window to keep the caret in the center of the viewport
            if (caretTop > viewportCenter || caretTop < viewportCenter) {
                window.scrollTo(0, windowScrollY + (caretTop - viewportCenter));
            }
        }
    }


    async function toggleMarkdown(): Promise<void> {
        if (selectedNote) {
            selectedNote.markdown = !selectedNote.markdown;
            await tick();
            autoResizeTextarea();
        }
    }

    // Add the keybind listener
    export const maximizedWindow = writable(false);
    function handleKeydown(event: KeyboardEvent): void {
        if (event.ctrlKey && event.key === "e") {
            event.preventDefault();
            if (selectedNote) { 
                toggleMarkdown();
            }
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
        if (event.ctrlKey && event.key === "f") {
            event.preventDefault();
            appWindow.toggleMaximize();
        }
        if(event.ctrlKey && event.key == "m") {
            event.preventDefault();
            appWindow.minimize();
        }
        if(event.ctrlKey && event.key == "Escape") {
            event.preventDefault();
            appWindow.close()
        }
        if(event.ctrlKey && event.key == "t") {
            event.preventDefault();
            goto('/tags');
        }
        if(event.ctrlKey && event.key == "u") {
            event.preventDefault();
            undoDeleteNote();
        }
    }

    async function searchNotesByTag() {
        if (!tagSearch.trim()) {
            // Fetch all notes if search input is empty
            try {
                notes = (await invoke("get_notes")) as Note[];
                if (notes.length > 0) {
                    selectedNote = notes[0];
                } else {
                    selectedNote = null;
                }
            } catch (error) {
                console.error("Failed to fetch all notes:", error);
            }
            return;
        }

        try {
            const filtered = (await invoke("search_notes_by_tag", { tag: tagSearch.trim() })) as Note[];
            notes = filtered;
            if (notes.length > 0) {
                selectedNote = notes[0]; // Select the first filtered note
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
            const rawNotes = (await invoke("get_notes")) as Note[];
            notes = rawNotes.map((note) => ({
                ...note,
                tags: typeof note.tags === "string"
                    ? note.tags.split(",").map((tag) => tag.trim()) // Convert tags from string to array
                    : note.tags,
                markdown: true, // Set markdown mode to true by default after saving
            }));
            if (notes.length > 0) {
                selectedNote = notes[0]; // Select the first note by default
            }
        } catch (error) {
            console.error("Failed to fetch notes:", error);
        }
    }

    // Store for dark mode
    export const darkMode = writable(false);

    // Toggle dark mode
    async function loadDarkMode() {
        const enabled = await invoke<boolean>('get_dark_mode');
        darkMode.set(enabled);
        if (enabled) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    }

    async function toggleDarkMode() {
        const currentMode = get(darkMode); // Get the current mode value
        const newMode = !currentMode;

        darkMode.set(newMode); // Update the store immediately for UI responsiveness

        // Persist the new mode to the backend
        try {
            await invoke('set_dark_mode', { enabled: newMode });
            if (newMode) {
                document.documentElement.classList.add('dark');
            } else {
                document.documentElement.classList.remove('dark');
            }
        } catch (error) {
            console.error('Failed to update dark mode:', error);
            // Rollback if saving fails
            darkMode.set(currentMode);
        }
    }

    let selectedNoteId: number | null = null;
    onMount(() => {
        // Adjust the textarea height on mount
        autoResizeTextarea(); 

        // Check system preferences for dark mode
        loadDarkMode();

        // Wrap the async logic in a self-invoking function
        (async () => {
            await fetchNotes(); // Ensure notes are fetched before proceeding

            const params = new URLSearchParams(window.location.search);
            const noteIdParam = params.get('noteId');
            selectedNoteId = noteIdParam ? parseInt(noteIdParam) : null;

            if (selectedNoteId) {
                selectedNote = notes.find((note) => note.id === selectedNoteId) || null;
            }
        })();

        window.addEventListener("keydown", handleKeydown);
        return () => {
            window.removeEventListener("keydown", handleKeydown);
        };
    });

</script>

<div class="flex flex-grow bg-gray-100 dark:bg-gray-900">
    <!-- Sidebar -->
    <aside style="width: var(--sidebar-width, 25%);" class="w-1/4 bg-gray-800 text-white p-4 flex flex-col">
        <div class="flex items-center justify-between mb-6">
            <!-- Notes Header -->
            <h2 class="text-2xl font-bold">Notes</h2>
        
            <!-- Dark Mode Button -->
            <button
                class="flex items-center space-x-2 p-2 bg-gray-200 rounded-lg hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600"
                on:click={toggleDarkMode}
            >
                <!-- Moon Icon -->
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                    class="w-5 h-5 text-gray-600 dark:text-gray-300"
                >
                    <path
                        d="M21.753 15.904a9.005 9.005 0 01-10.796-10.797c.084-.336-.206-.63-.54-.547A10.001 10.001 0 1019.94 19.94c.084-.336-.211-.63-.547-.54z"
                    />
                </svg>
            </button>
        </div>        

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
        <button
            class="mt-4 p-2 bg-blue-500 text-white font-semibold rounded-lg hover:bg-blue-600"
            on:click={addNewNote}
        >
            + New Note
        </button>
        <button
            class="mt-4 p-2 bg-blue-500 text-white font-semibold rounded-lg hover:bg-blue-600"
            on:click={() => goto('/tags')}
        >
            View Tags
        </button>
    </aside>

    <!-- Resizable handle -->
    <button
        aria-label="Resize sidebar"
        class="w-1 bg-gray-600 cursor-col-resize focus:outline-none"
        on:mousedown={startResizing}
    ></button>


    <!-- Main Content -->
  <main class="flex-1 flex flex-col relative">
    <!-- Header -->
    <header class="shadow-md p-4 flex items-center justify-between bg-white dark:bg-gray-800 dark:text-white">
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
            on:click={autoResizeTextarea}
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
          {#if lastDeletedNote}
            <button
                class="p-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
                on:click={undoDeleteNote}
                >
                Undo Delete
            </button>
          {/if}
        </div>
      {/if}
      
    </header>
  
    <!-- Content -->
    <div class="p-4 flex-1 bg-gray-50 overflow-y-auto dark:bg-gray-900">
      {#if selectedNote}
        {#if selectedNote.markdown}
          <div class="prose max-w-none mb-16">
            {@html marked(selectedNote.content)}
          </div>
        {:else}
        <textarea
            class="w-full h-auto border rounded-lg p-4 focus:outline-none focus:ring-2 focus:ring-blue-400 resize-none bg-white dark:bg-gray-200 dark:text-gray-800"
            bind:value={selectedNote.content}
            placeholder="Start writing your note here..."
            on:input={autoResizeTextarea}
            bind:this={textareaElement}
        ></textarea>
        {/if}
      {/if}
    </div>
  
    <!-- Sticky Tags Section -->
    {#if selectedNote}
<div class={`sticky bottom-0 bg-gray-100 border-t flex flex-col ${!selectedNote.markdown ? 'h-28' : 'h-16'} px-2 pt-4`}>
    <div class="flex items-center">
        <h2 class="text-lg font-bold pr-4">Tags</h2>
        <div class="flex items-center space-x-2 flex-wrap">
            {#each (Array.isArray(selectedNote.tags)
                ? selectedNote.tags.filter((tag) => tag.trim() !== "")
                : selectedNote.tags
                ? selectedNote.tags.split(",").map((tag) => tag.trim()).filter((tag) => tag !== "")
                : []) as tag}
            <div
                class="tag flex items-center px-2 py-1 rounded"
                class:bg-blue-500={tag.toLowerCase().includes(tagSearch.trim().toLowerCase()) && tagSearch.trim() !== ""}
                class:bg-gray-200={!tagSearch || !tag.toLowerCase().includes(tagSearch.trim().toLowerCase())}
                class:text-white={tag.toLowerCase().includes(tagSearch.trim().toLowerCase()) && tagSearch.trim() !== ""}
                class:text-gray-800={!tagSearch || !tag.toLowerCase().includes(tagSearch.trim().toLowerCase())}
            >
                {tag}
                {#if !selectedNote.markdown}
                <button
                    class="ml-2 text-red-500 hover:text-red-700"
                    on:click={() => removeTag(tag)}
                >
                    &times;
                </button>
                {/if}
            </div>
            {/each}
        </div>
    </div>

    <!-- Add Tag Input (Only visible in Edit Mode) -->
    {#if !selectedNote.markdown}
    <div class="flex space-x-2 mt-4 justify-between items-center">
        <input
            type="text"
            class="flex-grow border rounded-lg px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-400"
            bind:value={newTag}
            placeholder="Add a tag..."
        />
        <button
            class="flex-shrink-0 px-4 py-1 bg-blue-500 text-white rounded-lg hover:bg-blue-600 whitespace-nowrap"
            on:click={addTag}
        >
            Add Tag
        </button>
    </div>
    {/if}
</div>
{/if}





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
.bg-blue-500 {
    background-color: #3b82f6;
}

.bg-gray-200 {
    background-color: #e5e7eb;
}

.text-white {
    color: white;
}

/* Add dark mode styles */
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

:global(html.dark p, html.dark h3, html.dark h4, html.dark h1, html.dark h5) {
    color: var(--text-color);
}

:global(html.dark button:hover) {
    background-color: #4b5563; /* Dark button hover */
}

:global(html.dark header) {
    background-color: #1f2937 !important; /* Equivalent to dark:bg-gray-800 */
    color: #f3f4f6 !important; /* Equivalent to dark:text-white */
}
:global(html.dark textarea, html.dark input, html.dark .tag, html.dark ul, html.dark li) {
    background-color: #2d3748 !important; /* Equivalent to dark:bg-gray-800 */
    color: #f3f4f6 !important; /* Equivalent to dark:text-white */
}

:root {
  --sidebar-width: 25%; /* Default sidebar width */
}

aside {
  transition: width 0.2s ease-in-out;
}

.w-1 {
  width: 4px; /* Handle width */
  background-color: rgb(75 85 99); /* Gray-600 */
  cursor: col-resize;
  user-select: none;
}

.w-1:hover {
  background-color: rgb(55 65 81); /* Gray-700 */
}
</style>

