import App from './App.svelte'
import init, { start } from 'vite-wasm-functions'

const load = async () => {
  const startTime = performance.now()
  await init()
  start()
  const endTime = performance.now()
  console.log(`Call to wasm init took ${endTime - startTime} milliseconds`)


  const app = new App({
    target: document.getElementById('app')
  })
}

load()
