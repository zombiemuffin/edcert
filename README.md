Hi and welcome on the git page of my crate "edcert".

Edcert is a simple library for certification and authentication of data.

# How it works

1. You create a master keypair. This will be used to sign the highest certificate.
2. You create a root certificate. Sign this with the master key.
3. You can now create other certificates and use certificates to sign each other.
4. Transmit your certificates in a json-encoded format over the network.
5. Sign and verify data with the certificates using the ".sign" and ".verify" methods.

The design uses the "super-secure, super-fast" elliptic curve [Ed25519],
which you can learn more about here

For cryptography it uses the [sodiumoxide] library, which is based on [NaCl],
the well known cryptography libraray by Dan Bernstein et al.

# License

MIT

[Ed25519]: https://ed25519.cr.yp.to/
[sodiumoxide]: http://dnaq.github.io/sodiumoxide/sodiumoxide/index.html
[NaCl]: https://nacl.cr.yp.to/