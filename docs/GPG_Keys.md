

# Primary(Master) key 
## for both encryption and signature

### encryption subkey
* for encryption/decryption only. No passphrase neeeded when encrypting, but needed when decrypting
* can be specified in {store}/.gpg-id. required for operations involving encryption. e.g. creating and editing an entry file
### signature subkey
* for signatures only, passphrase needed when signing, but not needed when verifying the signature status
* currently, different signature keys are used for user auth (login request) and git commit signature. 
	* ~~for user auth, gpg will search its keyring for keys specified in the `.gpg-id` file. Although `gpg-id.sig` is not checked currently, to make sure if the `.gpg-id` is signed by valid users/admins, it should be checked in the future.~~
	* if the user is in the store's .gpg-id file and the file is valid, then that means they can decrypt the store.
	* ~~git will check `user.signingkey` field in {store}/.git/config first and if the field not found, search global git config: ~/.gitconfig. Either one of the config files must contain `user.signingkey` field. Otherwise, outputs error + cannot login to the store~~

Currently, signing key is used for user login that's located by the passed user-id. 

##### ex. of how to sign a file
```
alice% gpg --output doc.sig --sign doc

You need a passphrase to unlock the private key for
user: "Alice (Judge) <alice@cyb.org>"
1024-bit DSA key, ID BB7576AC, created 1999-06-04

Enter passphrase:
```
* gpg --sign creates a new file with sig extension that contains compressed version of the original file content and signature
* doc
  > A signature is created using the private key of the signer.
  > The signature is verified using the corresponding public key.
  > For example, Alice would use her own private key to digitally sign her latest submission to the Journal of Inorganic Chemistry.
  > The associate editor handling her submission would use Alice's public key to check the signature to verify that the submission indeed came from Alice and that it had not been modified since Alice sent it. 
  > A consequence of using digital signatures is that it is difficult to deny that you made a digital signature since that would imply your private key had been compromised.
  > The command-line option [--sign](https://www.gnupg.org/gph/en/manual/r606.html) is used to make a digital signature.
  > The document to sign is input, and the signed document is output.

##### ex. of how to clear-sign a file
```
alice% gpg --clearsign doc

You need a passphrase to unlock the secret key for
user: "Alice (Judge) <alice@cyb.org>"
1024-bit DSA key, ID BB7576AC, created 1999-06-04

-----BEGIN PGP SIGNED MESSAGE-----
Hash: SHA1

[...]
-----BEGIN PGP SIGNATURE-----
Version: GnuPG v0.9.7 (GNU/Linux)
Comment: For info see http://www.gnupg.org

iEYEARECAAYFAjdYCQoACgkQJ9S6ULt1dqz6IwCfQ7wP6i/i8HhbcOSKF4ELyQB1
oCoAoOuqpRqEzr4kOkQqHRLE/b8/Rw2k
=y6kj
-----END PGP SIGNATURE-----
```

* gpg --clearsign creates a new file with asc extension that contains the original original file content and ASCII armored signature
* <cite>https://www.gnupg.org/gph/en/manual/x135.html</cite>
  > A common use of digital signatures is to sign usenet postings or email messages.
  > In such situations it is undesirable to compress the document while signing it.
  > The option [--clearsign](https://www.gnupg.org/gph/en/manual/r684.html) causes the document to be wrapped in an ASCII-armored signature but otherwise does not modify the document.

##### ex. of how to verify/decrypt the signed file
```
blake% gpg --output doc --decrypt doc.sig
gpg: Signature made Fri Jun  4 12:02:38 1999 CDT using DSA key ID BB7576AC
gpg: Good signature from "Alice (Judge) <alice@cyb.org>"
```
* <cite>https://www.gnupg.org/gph/en/manual/x135.html</cite>
  > The document is compressed before signed, and the output is in binary format.
  > Given a signed document, you can either check the signature or check the signature and recover the original document.
  > To check the signature use the [--verify](https://www.gnupg.org/gph/en/manual/r697.html) option.
  > To verify the signature and extract the document use the --decrypt option.
  > The signed document to verify and recover is input and the recovered document is output.

##### ex. of detachd signature
```
alice% gpg --output doc.sig --detach-sig doc

You need a passphrase to unlock the secret key for
user: "Alice (Judge) <alice@cyb.org>"
1024-bit DSA key, ID BB7576AC, created 1999-06-04

Enter passphrase:
```
* <cite>https://www.gnupg.org/gph/en/manual/x135.html</cite>
  > A signed document has limited usefulness.
  > Other users must recover the original document from the signed version, and even with clearsigned documents, the signed document must be edited to recover the original.
  > Therefore, there is a third method for signing a document that creates a detached signature.
  > A detached signature is created using the [--detach-sig](https://www.gnupg.org/gph/en/manual/r622.html) option.
* To verify both the original file and signature file must be present.
- with both files
  ```
  blake% gpg --verify doc.sig doc
  gpg: Signature made Fri Jun  4 12:38:46 1999 CDT using DSA key ID BB7576AC
  gpg: Good signature from "Alice (Judge) <alice@cyb.org>"
- with signature file only
  ```
  blake% gpg --verify doc.sig
  gpg: no signed data
  gpg: can't hash datafile: No data
  ```
  

### Use case in a team in shraed file system
1. Each one has their own gpg private key ring and public key ring for other people's public keys.
2. To access a store, the user can specify the store name and their public key id must be specified in the store's .gpg-id file, which must have been signed by valid signers specified in store config.
3. When they make changes to the store, their public key should bei in the list of the valid signing keys for the store. The store will create a new commit with their signature. This requires access to their own sprivate key, which would be on each individual's private key ring and other people's public keys as it requires reencryption.

~~### Decryption with a particular secret
When given a passphrase, gpg does not check whether the passphrase belongs to the current user's key but check all the keys in the keyring. 
To make sure the passphrase belongs to the given user-id in the backend of Rpass, temp keyring can be created upon login.
~~1. each user should have their own keyring, which only has their own key on it. If the user is eligible for signing git commits, then add their email to .write-gpg-id file.~~~~
~~2. upon receiving subsequent requests, only use the keyring with the single secret for encryption/decryption/signature.~~
~~2. upon receiving subsequent requests, check .gpg-id file for user-ids and their privilege level
~~4. At the end, there should be 3 groups of users. 1. with read-only access, 2. with previlige to  make changes to store entries, 3. with previlige to add new person to .gpg-id and .admin-gpg-id files. The last group should have their emails in .super-gpg-id and the file should have been signed by everyone~~o
