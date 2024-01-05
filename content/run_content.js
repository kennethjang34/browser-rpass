(async () => {
  const src = chrome.runtime.getURL("content.js");
  const wasm = await import(src);
  try {
    await wasm.default();
  } catch (err) {
    //wasm script execution failed. Most likely due to some security policy.
    //Do not throw error, just log.
    //Users will not be able to use suggestion dropdowns but popup will still work.
    console.error(err);
  }
})();
