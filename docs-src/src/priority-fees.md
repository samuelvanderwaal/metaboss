# Priority Fees

To specify priority fees on Metaboss transactions, use the `--priority/-p` flag. The current priority values are set at:


| Priority   | MicroLamport Units | 
|------------|--------------------|
| "none"     | 20                 |  
| "low"      | 20_000             |
| "medium"   | 200_000            |    
| "high"     | 1_000_000          | 
| "max"      | 2_000_000          |  
|------------|--------------------|

The default value if no priority is specified is `None`.

The total amount spennt on priority fees per transaction is the microlamports multiplied by the compute units used. The approximate value of each priority level for the `update` subcommands are given below with a hard-coded compute unit value of 50k:

| Priority   | MicroLamport Units | Approximate Value @ $150 |
|------------|-----------|-----------------------------------|
| "none"     | 20        |  1 lamport/update                 |
| "low"      | 20_000    | ~$1 for 10k updates               | 
| "medium"   | 200_000   | ~$10 for 10k updates              |   
| "high"     | 1_000_000 | ~$0.01/update @ $150 SOL          |
| "max"      | 2_000_000 | ~$0.02/update @ $150 SOL          |
|------------|-----------|-----------------------------------|  

Currently only `update` subcommands support priority fees.

**When running large batch updates be sure to consider the cost of priority fees for the level you set!!**


