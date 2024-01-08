# Summary

Password manager with browser extension that helps users manage their secrets by public key cryptography.

# Core features

- Securely store user-ids, passwords, notes, associated domains(urls) in a user-selected 'store'
- A store could have multiple pub-keys/user-ids for encryption, which is useful in team environment
- Multiple password store support
- Stateful browser extension for currently available stores/passwords
- Auto-suggestion for user-id/password input fields

## Installation

Currently, only works on OSX and Linux, firefox and chrome (chromium users should be able to use them but would have to do extra configuration for native host. [Google has docs for it.](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging)

### Required dependencies

- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [Trunk](https://trunkrs.dev/)
- GPG key that's capable of signing and encrypting. If you don't have GPG executables installed, take a look at [GPG official documentation](https://gnupg.org/documentation/index.html)
- Stand-alone launchable pin entry program like `pinentry-mac` on OSX

### Build script

Given you have the required dependencies available, you should be able to install through the provided build script. You can run `build.sh` included.


## Demo videos


https://github.com/kennethjang34/browser-rpass/assets/89117160/bdcc1e46-9464-4d11-acb7-048c4c6a0341

