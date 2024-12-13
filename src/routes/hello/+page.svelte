<script lang="ts">
    import { goto } from '$app/navigation';
    import { marked } from 'marked';

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
        },
        {
            id: 2,
            title: "Plain Text Note",
            content: "This is a simple plain text note.",
            markdown: false,
        },
    ];

    let selectedNote: Note = notes[0];

    function selectNote(note: Note): void {
        selectedNote = note;
    }

    function addNewNote(): void {
        const newNote: Note = {
            id: notes.length > 0 ? notes[notes.length - 1].id + 1 : 1,
            title: `New Note ${notes.length + 1}`,
            content: "Start writing your note here...",
            markdown: false, // Default to plain text
        };
        notes = [...notes, newNote];
        selectNote(newNote);
    }

    function saveChanges(): void {
        notes = notes.map((note) =>
            note.id === selectedNote.id ? { ...note, content: selectedNote.content } : note
        );
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
</script>

<div class="flex h-screen bg-gray-100">
    <!-- Sidebar -->
    <aside class="w-1/4 bg-gray-800 text-white p-4 flex flex-col">
      <h2 class="text-2xl font-bold mb-6">Notes</h2>
      <ul class="flex-1 overflow-y-auto">
        {#each notes as note}
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
        <h1 class="text-xl font-bold">{selectedNote.title}</h1>
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
            <button class="p-2 bg-red-500 text-white rounded-lg hover:bg-red-600"
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
      </div>
    </main>
  </div>

  
  
  <style>
  /* Add custom styles if needed */
  </style>


