const runtime = chrome.runtime || browser.runtime;
//the following import doesn't work as content script is not a module
// import init from "./browser_rpass.js";
//async function run() {
//await wasm_bindgen(runtime.getURL('browser_rpass_bg.wasm'));
//}

//with the bundler:

const script = document.createElement("script");
script.setAttribute("type", "module");
script.setAttribute("src", runtime.getURL("run_wasm_content.js"));
const head =
  document.head ||
  document.getElementsByTagName("head")[0] ||
  document.documentElement;
head.insertBefore(script, head.lastChild);
