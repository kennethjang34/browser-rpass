(async () => {
  const src = chrome.runtime.getURL("content.js");
  console.log("init wasm");
  const wasm = await import(src);
  console.log("wasm script imported");
  try {
    await wasm.default();
    console.log("wasm script executed successfully");
  } catch (err) {
    //wasm script execution failed. Most likely due to some security policy.
    //Do not throw error, just log.
    //Users will not be able to use suggestion dropdowns but popup will still work.
    console.log(err);
  }
})();
