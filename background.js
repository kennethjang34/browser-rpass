import init from "./browser_rpass.js";
init();
console.log("inside background.js");
chrome.runtime.onMessage.addListener(function (request, sender, sendResponse) {
  console.log(
    sender.tab
      ? "from a content script:" + sender.tab.url
      : "from the extension"
  );
  if (request.greeting === "hello") sendResponse({ farewell: "goodbye" });
});
