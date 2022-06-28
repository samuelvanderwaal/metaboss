## Find

### Error

Look up Metaplex program error codes by hex or decimal values. 

E.g.:

```bash
metaboss find error 0x1770
```

or 

```bash
metaboss find error 6000
```

returns:

```bash
Auction House | PublicKeyMismatch: PublicKeyMismatch
Auctioneer |    BumpSeedNotInHashMap: Bump seed not in hash map
Candy Machine | IncorrectOwner: Account does not have correct owner!
```

Currently supported programs:

* Token Metadata 
* Auction House
* Auctioneer
* Candy Machine

It also decodes Anchor specific errors.

```bash
metaboss find 3000
```

```
Anchor Program |        AccountDiscriminatorAlreadySet: The account discriminator was already set on this account
```



### Missing Editions

Find any edition numbers in the sequence that have not been minted. See [editions][https://metaboss.rs/mint.html#editions] for more details on how to interact with editions with Metaboss.
