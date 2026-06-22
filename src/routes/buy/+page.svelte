<script lang="ts">
    // CosmoBond — Hackathon Project
    // Svelte 5 Runes + Skeleton + GSAP + CosmWasm ready
    import { onMount } from 'svelte';
    import gsap from 'gsap';
    import { Slider } from '@skeletonlabs/skeleton-svelte';
    import CurveChart from '$lib/components/CurveChart.svelte';

    // Note: Using @skeletonlabs/skeleton-svelte (Svelte 5)
    // Button, Card, Input are built with Tailwind + semantic HTML (recommended by Skeleton)

    let isSpendInputFocused = $state(false);
    let isBuying = $state(false);

    const isChartZoomed = $derived(isSpendInputFocused || isBuying);

    // === STATE (Runes) ===
    let spend = $state(5);
    let currentSupply = $state(245_678);
    let maxBalance = $state(1250);
    let displayedReceive = $state(0);

    // === CONSTANTS (bonding curve params from contract) ===
    const BASE_PRICE = 0.001;   // ATOM
    const SLOPE = 0.0000005;

    // === DERIVED (Runes) — super clean reactivity ===
    const tokensOut = $derived(
    calculateTokensOut(spend, currentSupply, BASE_PRICE, SLOPE)
    );
    const newSupply = $derived(currentSupply + tokensOut);
    const maxSupply = $derived(newSupply * 1.6);

    const newPrice = $derived(BASE_PRICE + SLOPE * newSupply);
    const currentPrice = $derived(BASE_PRICE + SLOPE * currentSupply);
    const priceImpact = $derived(
    ((newPrice - currentPrice) / currentPrice) * 100
    );

    // === GSAP number tween (runs whenever tokensOut changes) ===
    $effect(() => {
    if (tokensOut > 0) {
        gsap.to({ val: displayedReceive }, {
        val: tokensOut,
        duration: 0.65,
        ease: 'power2.out',
        overwrite: true,
        onUpdate: function () {
            displayedReceive = this.targets()[0].val;
        }
        });
    }
    });

    // === BONDING CURVE MATH (same as contract) ===
    function calculateTokensOut(
        spendAmount: number,
        S0: number,
        P0: number,
        m: number
        ): number {
        if (spendAmount <= 0) return 0;
        const a = m / 2;
        const b = P0 + m * S0;
        const c = -spendAmount;
        const discriminant = b * b - 4 * a * c;
        if (discriminant < 0) return 0;
        const T = (-b + Math.sqrt(discriminant)) / (2 * a);
        return Math.max(0, T);
    }



    // Re-animate preview area when spend changes (subtle GSAP)
    $effect(() => {
        // You can add gsap.to on a preview polygon here for extra flair
        console.log('Preview updated →', tokensOut.toFixed(2), 'BOND');
    });

    // === COSMWASM BUY ===
    async function buyOnCurve() {
        if (!spend || spend <= 0) return;

        isBuying = true;
        try {
            // === CosmJS + Keplr integration (real) ===
            // const { getSigningCosmWasmClient } = await import('@cosmjs/cosmwasm-stargate');
            // const client = await getSigningCosmWasmClient();
            // await client.execute(
            //   yourWalletAddress,
            //   CONTRACT_ADDRESS,
            //   { buy: {} },
            //   'auto',
            //   undefined,
            //   [{ denom: 'uatom', amount: Math.floor(spend * 1_000_000).toString() }]
            // );

            // === DEMO SUCCESS (remove in prod) ===
            await new Promise((r) => setTimeout(r, 1200));

            // GSAP success burst
            gsap.to('.buy-button', {
                scale: 1.05,
                duration: 0.15,
                yoyo: true,
                repeat: 1,
                ease: 'power2.inOut'
            });

            // Update local state (in real app → refetch from contract)
            currentSupply += tokensOut;
            spend = 0;

            // Toast / modal success
            alert('Purchase successful! Welcome to the curve 🚀');
        } catch (err) {
            console.error(err);
            alert('Transaction failed. Check console.');
        } finally {
            isBuying = false;
            isSpendInputFocused = false;
        }
    }

    function setMax() {
    spend = maxBalance;
    }
</script>

<div class="min-h-screen cosmic-bg text-white">
  <!-- Navbar -->
  <nav class="glass border-b border-white/10">
    <div class="max-w-7xl mx-auto px-8 py-5 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 rounded-full bg-linear-to-br from-primary-500 to-secondary-500 flex items-center justify-center">
          <span class="text-xl">⚛︎</span>
        </div>
        <div>
          <span class="font-bold text-2xl tracking-tighter">CosmoBond</span>
          <span class="ml-2 text-xs px-2 py-0.5 rounded bg-white/10">HACKATHON</span>
        </div>
      </div>

      <div class="flex items-center gap-4">
        <div class="px-4 py-1.5 rounded-full glass text-sm flex items-center gap-2">
          <div class="w-2 h-2 bg-emerald-400 rounded-full animate-pulse"></div>
          Cosmos Hub Testnet
        </div>
        
        <button class="btn btn-sm variant-glass">
          cosmos1...xyz
        </button>
        <button 
          class="btn btn-sm variant-ghost"
          onclick={() => alert('Disconnected (demo)')}
        >
          Disconnect
        </button>
      </div>
    </div>
  </nav>

  <!-- Hero -->
  <div class="max-w-5xl mx-auto pt-12 pb-6 px-8 text-center">
    <h1 class="text-6xl font-bold tracking-tighter mb-3">
      Buy <span class="text-transparent bg-clip-text bg-linear-to-r from-primary-400 to-secondary-400">$BOND</span><br>
      on the Bonding Curve
    </h1>
    <p class="text-xl text-white/70 max-w-md mx-auto">
      Fair launch powered by CosmWasm.<br>
      Price rises with demand. Be early.
    </p>
  </div>

  <!-- Main Content -->
  <div class="max-w-7xl mx-auto px-8 pb-16">
    <div class="grid grid-cols-1 lg:grid-cols-5 gap-8">
      
    

    <!-- LEFT: THE CURVE -->
    <div class="lg:col-span-3">
    <div class="glass p-8 rounded-3xl h-full">
        <div class="flex items-center justify-between mb-6">
        <h2 class="h2">The Bonding Curve</h2>
        <div class="badge variant-soft">Live • CosmWasm • √ curve</div>
        </div>

        <div class="relative bg-black/40 rounded-3xl p-5 border border-white/10">
        <CurveChart 
            bind:currentSupply 
            previewSupply={newSupply}
            isZoomed={isChartZoomed}
            {maxSupply}
            height={420}
            a={0.03}
        />
        </div>
    </div>
    </div>

      <!-- RIGHT: PURCHASE PANEL -->
      <div class="lg:col-span-2">
        <div class="glass p-8 rounded-3xl sticky top-8">
          <h2 class="h2 mb-1">Purchase $BOND Tokens</h2>
          <p class="text-sm text-white/60 mb-8">Instant settlement on Cosmos Hub</p>

          <!-- Spend -->
          <div class="mb-6">
            <label for="spend" class="label text-sm mb-2 block">You Spend</label>
            <div class="flex gap-3">
              <input 
                id="spend"
                type="number" 
                min="1"
                max={maxBalance}
                class="input text-3xl font-mono flex-1 bg-transparent border border-white/20 focus:border-primary-500 rounded-2xl px-6 py-4 outline-none"
                onfocus={() => isSpendInputFocused = true}
                onblur={() => isSpendInputFocused = false}
                bind:value={spend}
              />
              <div class="px-5 flex items-center justify-center rounded-xl glass font-medium">ATOM</div>
              <button 
                class="btn variant-ghost px-6 rounded-2xl"
                onclick={setMax}
              >
                Max
              </button>
            </div>
            
            <div class="mt-4">
              <Slider 
                value={[spend]} 
                onValueChange={(details) => { spend = details.value[0] }}
                max={maxBalance} 
                step={0.1}
                class="accent-primary-500"
                />
              <div class="flex justify-between text-[10px] text-white/50 mt-1">
                <div>0</div>
                <div>{maxBalance} ATOM</div>
              </div>
            </div>
          </div>

          <!-- Receive -->
          <div class="mb-8">
            <div class="text-sm text-white/60 mb-1">You Receive (estimated)</div>
            <div class="text-6xl font-bold tabular-nums tracking-tighter text-primary-400">
              {displayedReceive.toFixed(2)}
            </div>
            <div class="text-2xl text-primary-300 -mt-2">BOND</div>
          </div>

          <!-- Buy Button -->
          <button 
            class="buy-button w-full h-14 text-lg font-semibold bg-linear-to-r from-primary-500 via-secondary-500 to-primary-500 hover:brightness-110 active:scale-[0.985] transition-all rounded-3xl disabled:opacity-60 disabled:cursor-not-allowed flex items-center justify-center"
            disabled={isBuying || spend <= 0}
            onclick={buyOnCurve}
          >
            {#if isBuying}
              <span class="animate-pulse">Processing on-chain...</span>
            {:else}
              BUY ON THE BONDING CURVE
            {/if}
          </button>

                 <!-- Stats -->
          <div class="grid grid-cols-3 gap-3 text-sm mt-8">
            <div class="glass p-3 rounded-xl">
              <div class="text-white/60 text-xs">Current Price</div>
              <div class="font-mono text-lg">{currentPrice.toFixed(4)}</div>
              <div class="text-[10px] text-white/50">ATOM</div>
            </div>
            <div class="glass p-3 rounded-xl ring-1 ring-primary-500/40">
              <div class="text-white/60 text-xs">Price After Buy</div>
              <div class="font-mono text-lg text-primary-400">{newPrice.toFixed(4)}</div>
              <div class="text-[10px] text-white/50">ATOM</div>
            </div>
            <div class="glass p-3 rounded-xl">
              <div class="text-white/60 text-xs">Price Impact</div>
              <div class="font-mono text-lg {priceImpact > 10 ? 'text-warning-400' : 'text-emerald-400'}">
                +{priceImpact.toFixed(1)}%
              </div>
            </div>
          </div>


          <div class="text-center text-[10px] text-white/50 mt-4">
            Secured by CosmWasm • ~0.01 ATOM fee • Instant settlement
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .cosmic-bg {
    background: 
      radial-gradient(circle at 20% 30%, rgba(103, 232, 249, 0.06) 0%, transparent 50%),
      radial-gradient(circle at 80% 70%, rgba(192, 38, 255, 0.06) 0%, transparent 60%),
      #0a0a0f;
  }
  .glass {
    background: rgba(255,255,255,0.04);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255,255,255,0.08);
  }
</style>
