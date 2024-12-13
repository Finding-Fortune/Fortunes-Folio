<script lang="ts">
    import { goto } from '$app/navigation';

    function navigateBack() {
        goto('/hello');
    }
</script>

<p>hi</p>
<button>
    <a href="/">Go back to Home</a>
</button>

<div class="flex items-center justify-center h-screen bg-gray-100">
    <h1 class="text-4xl font-bold text-blue-500">Hello, Tailwind CSS!</h1>
  </div>

