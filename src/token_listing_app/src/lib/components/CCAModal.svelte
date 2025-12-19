<script lang="ts">
  import { renderContent } from '$lib/utils/markdown'

  const md = `
### Overview
The **Continuous Clearing Auction (CCA)** is a next-generation mechanism for token liquidity bootstrapping. Originally conceptualized and designed by **Uniswap Labs**, CCA generalizes the traditional uniform-price auction into continuous time.

At **Tokenlist.ing**, we have implemented this groundbreaking model using the high-performance architecture of the **Internet Computer (ICP)**, bringing mathematically perfect price discovery to the Solana, EVM, and ICP ecosystems via Chain Fusion.

### Key Terminology

*   **Flow Rate:** The speed at which your currency is being converted into tokens per second.
*   **Clearing Price:** The current market price derived from Total Demand divided by Supply Release Rate.
*   **Decaying Max Price:** A protection mechanism. If the market price exceeds your defined \`Max Price\`, your bid automatically pauses (Outbid), protecting you from overpaying.

### References & Further Reading

We encourage users to explore the original research behind this mechanism to understand why it is the future of fair launches.

*   **Uniswap CCA Whitepaper:** [Read the full paper (PDF)](https://docs.uniswap.org/assets/files/whitepaper_cca-fc8b989c3a5b11f6fcd199f6c6837a77.pdf)
    *   *The foundational academic paper detailing the game-theory and math behind CCA.*
*   **Uniswap Documentation:** [Liquidity Launchpad / CCA](https://docs.uniswap.org/contracts/liquidity-launchpad/CCA)
    *   *Official documentation from the Uniswap team.*
*   **Tokenlist.ing CCA:** [github.com/ldclabs/token-listing](https://github.com/ldclabs/token-listing/blob/main/docs/cca.md)
    *   *Our open-source implementation verifying the logic and security.*

---

**Ready to participate?**
Enter the launchpad and experience the first true Continuous Clearing Auction on [Tokenlist.ing](https://tokenlist.ing).
  `
</script>

<div class="relative">
  <div class="md-content w-full text-pretty wrap-break-word"
    >{@html renderContent(md)}</div
  >
</div>
