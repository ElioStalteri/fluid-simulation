<script lang="ts">
  import { addArray } from "vite-wasm-functions";

  let count: number = 0;
  let arr = [];
  let rustSum = 0
  const increment = () => {
    count += 1;
    arr.push(count);
    rustSum = addArray(arr)
  };
</script>

<button on:click={increment}>
  Sum of 0 to {count}: {rustSum}
</button>

<style>
  button {
    font-family: inherit;
    font-size: inherit;
    padding: 1em 2em;
    color: #ff3e00;
    background-color: rgba(255, 62, 0, 0.1);
    border-radius: 2em;
    border: 2px solid rgba(255, 62, 0, 0);
    outline: none;
    width: 300px;
    font-variant-numeric: tabular-nums;
    cursor: pointer;
  }

  button:focus {
    border: 2px solid #ff3e00;
  }

  button:active {
    background-color: rgba(255, 62, 0, 0.2);
  }
</style>
