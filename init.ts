import init from "./dist/wasm/client.js"
;(window as any).sleep = (delay: number) =>
  new Promise((resolve) => setTimeout(resolve, delay))

await init("/client_bg.wasm")
