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

const test = async () => {
  const response = await chrome.runtime.sendMessage({
    greeting: "hello from content",
  });
  console.log("response received in content.js: " + response);
};
test();
window.addEventListener("message", (event) => {
  console.log("inside window event listener in content.js");
  console.log(event);
  console.log("data: " + event.data);
  console.log("origin: " + event.origin);
  // console.log("source: " + event.source);
  console.log(event.source);
});

//import init from "./browser_rpass.js";
// async function run() {
//   await init();
// }
// console.log(document);
// document.body.style.backgroundColor = "orange";
// chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
//   console.log(tabs);
//   let url = tabs[0].url;
//   console.log(url);
//   // use `url` here inside the callback because it's asynchronous!
// });

// async function moveToFirstPosition(activeInfo) {
//   try {
//     await chrome.tabs.move(activeInfo.tabId, { index: 0 });
//     console.log("Success.");
//   } catch (error) {
//     if (
//       error ==
//       "Error: Tabs cannot be edited right now (user may be dragging a tab)."
//     ) {
//       setTimeout(() => moveToFirstPosition(activeInfo), 50);
//     } else {
//       console.error(error);
//     }
//   }
// }
// chrome.tabs.onActivated.addListener(moveToFirstPosition);
