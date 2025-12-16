<script lang="ts">
  import { renderContent } from '$lib/utils/markdown'

  const md = `
Repository: [github.com/ldclabs/token-listing/blob/main/src/ic_auction](https://github.com/ldclabs/token-listing/blob/main/src/ic_auction)

The **Continuous Clearing Auction (CCA)** is a novel auction mechanism that generalizes the uniform-price auction into continuous time. It provides fair price discovery for bootstrapping initial liquidity while eliminating timing games and encouraging early participation.

The **ICP Continuous Clearing Auction** is a high-precision, continuous-time token distribution mechanism designed for the Internet Computer. Unlike EVM-based implementations that rely on discrete blocks and tick-based approximations, this implementation leverages the computational capacity of ICP to achieve **millisecond-level precision** and **O(1) settlement complexity** using an accumulator-based mathematical model.

## References

- Tokenlist.ing CCA Docs: [github.com/ldclabs/token-listing/blob/main/docs](https://github.com/ldclabs/token-listing/blob/main/docs/cca.md)
- Uniswap CCA Website: [cca.uniswap.org](https://cca.uniswap.org/)
- Uniswap CCA Docs: [docs.uniswap.org/contracts/liquidity-launchpad/CCA](https://docs.uniswap.org/contracts/liquidity-launchpad/CCA)
- [Uniswap CCA whitepaper](https://docs.uniswap.org/assets/files/whitepaper_cca-fc8b989c3a5b11f6fcd199f6c6837a77.pdf)
  `
</script>

<div class="relative">
  <div class="md-content w-full text-pretty wrap-break-word"
    >{@html renderContent(md)}</div
  >
</div>
