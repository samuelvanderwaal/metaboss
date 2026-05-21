## Derive

### Derive Candy Machine V2 Creator

Derive the candy machine creator PDA from the candy machine id.

#### Usage

```bash
metaboss derive cmv2-creator <candy_machine_id>
```

### Derive Edition

Derive the edition PDA from the mint account.

```bash
metaboss derive edition <mint_account>
```

### Derive Metadata

Derive the metadata PDA from the mint account.

```bash
metaboss derive metadata <mint_account>
```

### Derive PDA

Derive a generic PDA from a list of seeds and a program id.

By default, the canonical bump is found automatically (equivalent to Solana's `find_program_address`). Pass `--bump` to derive the PDA with a specific bump byte (equivalent to `create_program_address`); the command will fail if that bump does not produce a valid off-curve address for the given seeds and program.

#### Usage

```bash
metaboss derive pda <seed1>,<seed2>,<seed3> <program_id>
```

With an explicit bump:

```bash
metaboss derive pda <seed1>,<seed2>,<seed3> <program_id> --bump <bump>
```

#### Examples

Derive a PDA letting metaboss find the canonical bump:

```bash
metaboss derive pda emit_event_authority B6rD3jUm4nh5irXAApmkyuv2TAmM1PpZrc6xXsnCExXx
```

Derive the same PDA with an explicit bump (useful when you already know the bump stored on-chain and want to verify it):

```bash
metaboss derive pda emit_event_authority B6rD3jUm4nh5irXAApmkyuv2TAmM1PpZrc6xXsnCExXx --bump 252
```

Note: the canonical bump is not always 255. `find_program_address` walks bumps down from 255 and returns the first one that yields an off-curve address, so for some seed/program combinations the canonical bump can be lower.
