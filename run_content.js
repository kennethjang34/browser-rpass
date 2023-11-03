(async () => {
  const src = chrome.runtime.getURL("content.js");
  console.log("init wasm");
  const wasm = await import(src);
  await wasm.default();
})();
