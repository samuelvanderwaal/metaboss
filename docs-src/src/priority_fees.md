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

The total amount spennt on priority fees per transaction is the microlamports multiplied by the compute units used. Metaboss simulates each tranasaction to determine the compute units required, and then uses that value or a default.

Setting higher levels of priority fees are unlikely to make a significant difference in the getting transactions confirmed, so it's recommended to use "none" or "low" until Solana network performance improves. However, higher levels are included to give users options.

**When running large batch updates be sure to consider the cost of priority fees for the level you set!! Medium, High and Max could cost significant amounts of SOL when updating thousands of NFTs.**


