<script lang="ts">
    import { goto } from '$app/navigation';
    import { marked } from 'marked';
    import { onMount } from "svelte";

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

    let notes: Note[] = [
        {
            id: 1,
            title: "Markdown Example",
            content: `
# Markdown Syntax Test

## Subheading

### Sub-subheading

**Bold Text**

_Italic Text_

~~Strikethrough~~

1. Ordered List Item 1
2. Ordered List Item 2
   - Nested Bullet 1
   - Nested Bullet 2

- Unordered List Item 1
- Unordered List Item 2

\`Inline code\`

\`\`\`javascript
// Code block example
function helloWorld() {
  console.log("Hello, World!");
}
\`\`\`

> Blockquote Example

[Link to Google](https://www.google.com)

![Image Example](https://via.placeholder.com/150)
    `,
            markdown: true,
            tags: ["markdown", "example"],
        },
        {
            id: 2,
            title: "Plain Text Note",
            content: "This is a simple plain text note.",
            markdown: false,
            tags: ["plain", "text"],
        },
    ];

    let selectedNote: Note = notes[0];

    let newTag = "";

    function addTag(): void {
        if (newTag.trim() && !selectedNote.tags.includes(newTag.trim())) {
            selectedNote.tags = [...selectedNote.tags, newTag.trim()];
            newTag = "";
        }
    }

    function removeTag(tag: string): void {
        selectedNote.tags = selectedNote.tags.filter((t) => t !== tag);
    }

    let tagSearch = "";

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

    function addNewNote(): void {
        const newNote: Note = {
            id: notes.length > 0 ? notes[notes.length - 1].id + 1 : 1,
            title: `New Note ${notes.length + 1}`,
            content: "Start writing your note here...",
            markdown: false, // Default to plain text
            tags: [],
        };
        notes = [...notes, newNote];
        selectNote(newNote);
    }

    function saveChanges(): void {
        notes = notes.map((note) =>
            note.id === selectedNote.id
                ? { ...note, title: selectedNote.title, content: selectedNote.content, tags: selectedNote.tags }
                : note
        );
        // If we save changes go out of edit mode back to markdown preview mode
        selectedNote.markdown = true;
    }

    function deleteNote(): void {
        if (notes.length > 1) {
            notes = notes.filter((note) => note.id !== selectedNote.id);
            selectedNote = notes[0]; // Set the first note as the selected note
        } else {
            alert("Cannot delete the last note!");
        }
    }

    function toggleMarkdown(): void {
        selectedNote.markdown = !selectedNote.markdown;
    }

    // Add the keybind listener
  function handleKeydown(event: KeyboardEvent): void {
    if (event.ctrlKey && event.key === "e") {
      event.preventDefault(); // Prevent default browser behavior
      toggleMarkdown();
    }
    if (event.ctrlKey && event.key === "s") {
      event.preventDefault(); // Prevent default browser behavior
      saveChanges();
    }
    if (event.ctrlKey && event.key === "d") {
      event.preventDefault(); // Prevent default browser behavior
      deleteNote();
    }
    if (event.ctrlKey && event.key === "n") {
      event.preventDefault(); // Prevent default browser behavior
      addNewNote();
    }
  }

  // Lifecycle to attach and detach the listener
  onMount(() => {
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
        />
    </div>

    <!-- Notes List -->

      <ul class="flex-1 overflow-y-auto">
        {#each filteredNotes() as note}
            <li>
                <button
                class="w-full text-left p-3 rounded-lg mb-2 cursor-pointer bg-gray-700 hover:bg-gray-600"
                on:click={() => selectNote(note)}
                >
                <h3 class="text-lg font-semibold">{note.title}</h3>
                <p class="text-sm text-gray-300 truncate">{note.content}</p>
            </button>
          </li>
        {/each}
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
        </div>
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
      </header>        
  
      <!-- Content -->
      <div class="p-4 flex-1 bg-gray-50 overflow-y-auto">
        {#if selectedNote.markdown}
          <div class="prose max-w-none">
            {@html marked(selectedNote.content)}
          </div>
        {:else}
          <textarea
            class="w-full h-full border rounded-lg p-4 focus:outline-none focus:ring-2 focus:ring-blue-400 resize-none"
            bind:value={selectedNote.content}
          ></textarea>
        {/if}

        <!-- Tags Section -->
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

